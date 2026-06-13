use axum::http::{HeaderMap, StatusCode};
use bytes::Bytes;
use futures::{
    stream::{self, BoxStream},
    Stream, StreamExt,
};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{borrow::Cow, pin::Pin, sync::Arc, time::Duration};
use warpsock::{HttpVersion, Message};

use crate::{
    auth::codex::{load_codex_auth, refresh_codex_auth, CODEX_OPENAI_BETA, CODEX_ORIGINATOR},
    codex_catalog::CodexCatalog,
    model_alias::{self, Provider, ResolvedModel},
    rate_limit,
    sse::splice::splice_completed_event_filtered,
    state::CodexTransport,
    upstream_response::{collect_warpsock_body, observe_warpsock_response, warpsock_body_stream},
    AppError, AppResult, AppState,
};

pub const CODEX_RESPONSES_WSS_URL: &str = "wss://chatgpt.com/backend-api/codex/responses";
pub const CODEX_RESPONSES_HTTP_URL: &str = "https://chatgpt.com/backend-api/codex/responses";
pub const CODEX_PROVIDER: &str = "codex";
const CODEX_REMOTE_COMPACTION_V2_FEATURE: &str = "remote_compaction_v2";
const CODEX_RESPONSES_ALLOWED_FIELDS: &[&str] = &[
    "model",
    "instructions",
    "input",
    "context_management",
    "tools",
    "tool_choice",
    "parallel_tool_calls",
    "previous_response_id",
    "reasoning",
    "store",
    "stream",
    "include",
    "service_tier",
    "prompt_cache_key",
    "text",
    "generate",
    "client_metadata",
];

pub type CodexResponseStream = Pin<Box<dyn Stream<Item = AppResult<Bytes>> + Send>>;

pub async fn refresh_codex_model_catalog(state: &AppState) -> AppResult<Arc<CodexCatalog>> {
    let headers = codex_headers(state)?;
    state
        .codex_catalog
        .refresh_from_endpoint(&state.warpsock, &headers, &state.runtime.codex_models_url)
        .await
}

pub async fn warm_codex_model_catalog_with_timeout(state: &AppState, timeout: Duration) {
    match tokio::time::timeout(timeout, refresh_codex_model_catalog(state)).await {
        Ok(Ok(_)) => {}
        Ok(Err(err)) => tracing::warn!(%err, "Codex model catalog startup refresh failed"),
        Err(_) => tracing::warn!(?timeout, "Codex model catalog startup refresh timed out"),
    }
}

pub struct CodexCatalogRefresher {
    handle: tokio::task::JoinHandle<()>,
}

impl CodexCatalogRefresher {
    pub fn abort(&self) {
        self.handle.abort();
    }
}

pub fn spawn_codex_model_catalog_refresher(state: AppState) -> CodexCatalogRefresher {
    let handle = tokio::spawn(async move {
        let interval = state
            .runtime
            .codex_catalog_ttl
            .max(Duration::from_millis(1));
        loop {
            if let Err(err) = refresh_codex_model_catalog(&state).await {
                tracing::warn!(%err, "Codex model catalog background refresh failed");
            }
            tokio::time::sleep(interval).await;
        }
    });
    CodexCatalogRefresher { handle }
}

pub fn codex_headers(state: &AppState) -> AppResult<HeaderMap> {
    let auth = load_codex_auth(state)?;
    let mut headers = HeaderMap::new();
    headers.insert(
        axum::http::header::AUTHORIZATION,
        format!("Bearer {}", auth.access_token)
            .parse()
            .map_err(|_| AppError::BadRequest("invalid Codex access token".into()))?,
    );
    headers.insert("originator", CODEX_ORIGINATOR.parse().unwrap());
    headers.insert("OpenAI-Beta", CODEX_OPENAI_BETA.parse().unwrap());
    headers.insert(
        "x-codex-beta-features",
        CODEX_REMOTE_COMPACTION_V2_FEATURE.parse().unwrap(),
    );
    if let Some(account_id) = auth.account_id.filter(|value| !value.trim().is_empty()) {
        headers.insert(
            "ChatGPT-Account-Id",
            account_id
                .parse()
                .map_err(|_| AppError::BadRequest("invalid Codex account id".into()))?,
        );
    }
    Ok(headers)
}

fn codex_headers_for_body(state: &AppState, body: &serde_json::Value) -> AppResult<HeaderMap> {
    let mut headers = codex_headers(state)?;
    if has_remote_compaction_v2_trigger(body) {
        headers.insert(
            "x-codex-beta-features",
            CODEX_REMOTE_COMPACTION_V2_FEATURE.parse().unwrap(),
        );
    }
    Ok(headers)
}

pub fn prepare_responses_body(body: serde_json::Value) -> AppResult<serde_json::Value> {
    prepare_responses_body_with_resolver(body, model_alias::resolve_model_required)
}

pub fn prepare_responses_body_with_resolver<F>(
    mut body: serde_json::Value,
    mut resolve: F,
) -> AppResult<serde_json::Value>
where
    F: FnMut(&str) -> AppResult<ResolvedModel>,
{
    let model = body
        .get("model")
        .and_then(|value| value.as_str())
        .ok_or_else(|| AppError::BadRequest("missing model".into()))?;
    let alias = resolve(model)?;
    if alias.provider != Provider::Codex {
        return Err(AppError::ModelNotSupported(model.to_string()));
    }
    body["model"] = serde_json::Value::String(alias.upstream_model);
    if let Some(object) = body.as_object_mut() {
        reject_unsupported_input_items(object.get("input"))?;
        reject_unsupported_tools(object.get("tools"))?;
        retain_codex_responses_allowed_fields(object);
        normalize_codex_service_tier(object);

        object.insert("stream".into(), serde_json::Value::Bool(true));
        object.insert("store".into(), serde_json::Value::Bool(false));
        if object.get("instructions").is_none_or(Value::is_null) {
            object.insert(
                "instructions".into(),
                serde_json::Value::String("You are a helpful assistant.".into()),
            );
        }
        normalize_input(object)?;
        match object.get_mut("reasoning") {
            Some(reasoning) => {
                let reasoning = reasoning
                    .as_object_mut()
                    .ok_or_else(|| AppError::BadRequest("reasoning must be an object".into()))?;
                reasoning
                    .entry("summary")
                    .or_insert(serde_json::Value::String("auto".into()));
            }
            None => {
                object.insert(
                    "reasoning".into(),
                    json!({
                        "effort": "medium",
                        "summary": "auto"
                    }),
                );
            }
        }
        let include = object.entry("include").or_insert_with(|| json!([]));
        if let Some(values) = include.as_array_mut() {
            if !values
                .iter()
                .any(|value| value.as_str() == Some("reasoning.encrypted_content"))
            {
                values.push(serde_json::Value::String(
                    "reasoning.encrypted_content".into(),
                ));
            }
        } else {
            object.insert("include".into(), json!(["reasoning.encrypted_content"]));
        }
    }
    Ok(body)
}

fn retain_codex_responses_allowed_fields(object: &mut serde_json::Map<String, serde_json::Value>) {
    object.retain(|key, _| CODEX_RESPONSES_ALLOWED_FIELDS.contains(&key.as_str()));
}

fn normalize_codex_service_tier(object: &mut serde_json::Map<String, serde_json::Value>) {
    object.insert("service_tier".into(), Value::String("priority".into()));
}

fn normalize_input(object: &mut serde_json::Map<String, serde_json::Value>) -> AppResult<()> {
    let Some(input) = object.get_mut("input") else {
        return Ok(());
    };
    if let Some(text) = input.as_str() {
        *input = json!([{
            "role": "user",
            "content": [{
                "type": "input_text",
                "text": text
            }]
        }]);
    }
    Ok(())
}

fn reject_unsupported_input_items(input: Option<&serde_json::Value>) -> AppResult<()> {
    if let Some(item_type) = contains_any_typed_item(
        input,
        &[
            "input_audio",
            "input_file",
            "file",
            "localImage",
            "local_image",
            "image_asset_pointer",
            "image_asset_pointer_citation",
        ],
    ) {
        return Err(AppError::BadRequest(format!(
            "{item_type} is not supported for Codex responses"
        )));
    }
    if contains_input_image_file_id(input) {
        return Err(AppError::BadRequest(
            "input_image.file_id is not supported for Codex responses".into(),
        ));
    }
    Ok(())
}

fn reject_unsupported_tools(tools: Option<&serde_json::Value>) -> AppResult<()> {
    if let Some(tool_type) = contains_any_typed_item(
        tools,
        &[
            "apply_patch",
            "file_search",
            "code_interpreter",
            "mcp",
            "shell",
            "local_shell",
            "computer",
        ],
    ) {
        return Err(AppError::BadRequest(format!(
            "{tool_type} tool is not supported for Codex responses"
        )));
    }
    Ok(())
}

fn contains_any_typed_item<'a>(
    value: Option<&serde_json::Value>,
    item_types: &'a [&'a str],
) -> Option<&'a str> {
    match value {
        Some(Value::Object(object)) => {
            if let Some(found) = object
                .get("type")
                .and_then(Value::as_str)
                .and_then(|value| {
                    item_types
                        .iter()
                        .copied()
                        .find(|item_type| value == *item_type)
                })
            {
                return Some(found);
            }
            object
                .values()
                .find_map(|value| contains_any_typed_item(Some(value), item_types))
        }
        Some(Value::Array(values)) => values
            .iter()
            .find_map(|value| contains_any_typed_item(Some(value), item_types)),
        _ => None,
    }
}

fn contains_input_image_file_id(value: Option<&serde_json::Value>) -> bool {
    match value {
        Some(Value::Object(object)) => {
            let is_input_image = object
                .get("type")
                .and_then(Value::as_str)
                .is_some_and(|value| value == "input_image");
            if is_input_image && object.contains_key("file_id") {
                return true;
            }
            object
                .values()
                .any(|value| contains_input_image_file_id(Some(value)))
        }
        Some(Value::Array(values)) => values
            .iter()
            .any(|value| contains_input_image_file_id(Some(value))),
        _ => false,
    }
}

pub fn prepare_response_create_payload(body: serde_json::Value) -> AppResult<serde_json::Value> {
    prepare_response_create_payload_with_resolver(body, model_alias::resolve_model_required)
}

pub fn prepare_response_create_payload_with_resolver<F>(
    body: serde_json::Value,
    resolve: F,
) -> AppResult<serde_json::Value>
where
    F: FnMut(&str) -> AppResult<ResolvedModel>,
{
    prepare_responses_body_with_resolver(body, resolve)
}

pub fn prepare_nested_response_create_payload_with_resolver<F>(
    body: serde_json::Value,
    resolve: F,
) -> AppResult<serde_json::Value>
where
    F: FnMut(&str) -> AppResult<ResolvedModel>,
{
    Ok(json!({
        "type": "response.create",
        "response": prepare_responses_body_with_resolver(body, resolve)?,
    }))
}

pub fn prepare_response_create_event_payload_with_resolver<F>(
    mut body: serde_json::Value,
    resolve: F,
) -> AppResult<serde_json::Value>
where
    F: FnMut(&str) -> AppResult<ResolvedModel>,
{
    if body.get("type").and_then(|value| value.as_str()) != Some("response.create") {
        let prepared = prepare_responses_body_with_resolver(body, resolve)?;
        return Ok(flat_response_create_event(prepared));
    }

    if let Some(response) = body.get_mut("response") {
        if !response.is_object() {
            return Err(AppError::BadRequest("invalid response".into()));
        }
        let prepared = prepare_responses_body_with_resolver(response.take(), resolve)?;
        return Ok(flat_response_create_event(prepared));
    }

    let mut object = match body {
        Value::Object(object) => object,
        _ => {
            return Err(AppError::BadRequest(
                "response.create must be an object".into(),
            ))
        }
    };
    object.remove("type");
    let prepared = prepare_responses_body_with_resolver(Value::Object(object), resolve)?;
    Ok(flat_response_create_event(prepared))
}

fn flat_response_create_event(prepared: serde_json::Value) -> serde_json::Value {
    let mut event = serde_json::Map::new();
    event.insert(
        "type".into(),
        serde_json::Value::String("response.create".into()),
    );
    if let Value::Object(response) = prepared {
        event.extend(response);
    }
    serde_json::Value::Object(event)
}

/// Serialize `body` as the flat `response.create` event without producing
/// an intermediate `Value` clone. Mirrors [`flat_response_create_event`]
/// when `body` is already a JSON object: the `"type"` key is emitted
/// first, followed by every entry of the object.
fn serialize_flat_response_create_event(body: &Value) -> String {
    use serde::{
        ser::{SerializeMap, Serializer},
        Serialize,
    };

    struct Wrap<'a>(&'a Value);

    impl<'a> Serialize for Wrap<'a> {
        fn serialize<S: Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
            match self.0 {
                Value::Object(map) => {
                    let mut out = ser.serialize_map(Some(map.len() + 1))?;
                    out.serialize_entry("type", "response.create")?;
                    for (key, value) in map {
                        out.serialize_entry(key, value)?;
                    }
                    out.end()
                }
                _ => {
                    let mut out = ser.serialize_map(Some(1))?;
                    out.serialize_entry("type", "response.create")?;
                    out.end()
                }
            }
        }
    }

    serde_json::to_string(&Wrap(body)).expect("serde_json::Value is infallibly serializable")
}

pub async fn responses(state: &AppState, body: serde_json::Value) -> AppResult<Bytes> {
    responses_with_endpoints(
        state,
        body,
        &state.runtime.codex_responses_wss_url,
        &state.runtime.codex_responses_http_url,
    )
    .await
}

pub async fn responses_with_endpoints(
    state: &AppState,
    body: serde_json::Value,
    wss_url: &str,
    http_url: &str,
) -> AppResult<Bytes> {
    let body = prepare_responses_body_with_resolver(body, |model| {
        state.resolve_model_for_format(model, "responses")
    })?;
    responses_prepared_with_endpoints(state, body, wss_url, http_url).await
}

pub async fn responses_prepared(state: &AppState, body: serde_json::Value) -> AppResult<Bytes> {
    responses_prepared_with_endpoints(
        state,
        body,
        &state.runtime.codex_responses_wss_url,
        &state.runtime.codex_responses_http_url,
    )
    .await
}

pub async fn responses_prepared_stream(
    state: &AppState,
    body: serde_json::Value,
) -> AppResult<CodexResponseStream> {
    responses_prepared_stream_with_endpoints(
        state,
        body,
        &state.runtime.codex_responses_wss_url,
        &state.runtime.codex_responses_http_url,
    )
    .await
}

async fn responses_prepared_with_endpoints(
    state: &AppState,
    body: serde_json::Value,
    wss_url: &str,
    http_url: &str,
) -> AppResult<Bytes> {
    match state.runtime.codex_transport {
        CodexTransport::Wss => send_wss_with_refresh(state, wss_url, &body).await,
        CodexTransport::Http => send_http_with_refresh(state, http_url, &body).await,
        CodexTransport::WssThenHttp if state.codex_wss_latched() => {
            send_http_with_refresh(state, http_url, &body).await
        }
        CodexTransport::WssThenHttp => match send_wss_with_refresh(state, wss_url, &body).await {
            Ok(bytes) => Ok(bytes),
            Err(wss_error) => {
                state.record_codex_wss_failure();
                tracing::warn!(error = %wss_error, "Codex WSS failed; using HTTP fallback");
                send_http_with_refresh(state, http_url, &body).await
            }
        },
    }
}

async fn responses_prepared_stream_with_endpoints(
    state: &AppState,
    body: serde_json::Value,
    wss_url: &str,
    http_url: &str,
) -> AppResult<CodexResponseStream> {
    match state.runtime.codex_transport {
        CodexTransport::Wss => send_wss_stream_with_refresh(state, wss_url, &body).await,
        CodexTransport::Http => send_http_stream_with_refresh(state, http_url, &body).await,
        CodexTransport::WssThenHttp if state.codex_wss_latched() => {
            send_http_stream_with_refresh(state, http_url, &body).await
        }
        CodexTransport::WssThenHttp => {
            match send_wss_stream_with_refresh(state, wss_url, &body).await {
                Ok(stream) => Ok(stream),
                Err(wss_error) => {
                    state.record_codex_wss_failure();
                    tracing::warn!(error = %wss_error, "Codex WSS failed; using HTTP fallback");
                    send_http_stream_with_refresh(state, http_url, &body).await
                }
            }
        }
    }
}

async fn send_wss_with_refresh(
    state: &AppState,
    wss_url: &str,
    body: &serde_json::Value,
) -> AppResult<Bytes> {
    match send_wss(state, wss_url, body).await {
        Err(err) if maybe_auth_failure(&err) => {
            refresh_codex_auth(state).await?;
            send_wss(state, wss_url, body).await
        }
        result => result,
    }
}

async fn send_wss_stream_with_refresh(
    state: &AppState,
    wss_url: &str,
    body: &serde_json::Value,
) -> AppResult<CodexResponseStream> {
    match send_wss_stream(state, wss_url, body).await {
        Err(err) if maybe_auth_failure(&err) => {
            refresh_codex_auth(state).await?;
            send_wss_stream(state, wss_url, body).await
        }
        result => result,
    }
}

async fn send_wss(state: &AppState, wss_url: &str, body: &serde_json::Value) -> AppResult<Bytes> {
    let CodexWsCheckout {
        mut websocket,
        pool_key,
        reused: _,
    } = checkout_responses_wss_for_body(state, wss_url, body).await?;
    websocket
        .send_text(serialize_flat_response_create_event(body))
        .await
        .map_err(|err| AppError::Upstream(format!("Codex WSS send failed: {err}")))?;

    let mut stream = String::new();
    let mut reusable = false;
    while let Some(message) = websocket
        .next()
        .await
        .map_err(|err| AppError::Upstream(format!("Codex WSS read failed: {err}")))?
    {
        match message {
            Message::Text(text) => {
                let terminal = contains_terminal_response_event(&text);
                append_response_stream_chunk(&mut stream, &text);
                if terminal {
                    reusable = true;
                    break;
                }
            }
            Message::Binary(bytes) => {
                let text = String::from_utf8_lossy(&bytes);
                let terminal = contains_terminal_response_event(&text);
                append_response_stream_chunk(&mut stream, &text);
                if terminal {
                    reusable = true;
                    break;
                }
            }
            Message::Ping(_) | Message::Pong(_) | Message::Close(_) => {}
        }
    }
    if reusable {
        state.store_codex_ws(pool_key, websocket).await;
    }
    Ok(normalize_sse(stream))
}

async fn send_wss_stream(
    state: &AppState,
    wss_url: &str,
    body: &serde_json::Value,
) -> AppResult<CodexResponseStream> {
    let CodexWsCheckout {
        mut websocket,
        mut pool_key,
        mut reused,
    } = checkout_responses_wss_for_body(state, wss_url, body).await?;
    let request_payload = serialize_flat_response_create_event(body);
    if let Err(err) = websocket.send_text(request_payload.clone()).await {
        if !reused {
            return Err(AppError::Upstream(format!("Codex WSS send failed: {err}")));
        }
        let checkout = checkout_responses_wss_for_body(state, wss_url, body).await?;
        websocket = checkout.websocket;
        pool_key = checkout.pool_key;
        reused = false;
        websocket
            .send_text(request_payload.clone())
            .await
            .map_err(|err| AppError::Upstream(format!("Codex WSS send failed: {err}")))?;
    }

    Ok(Box::pin(stream::unfold(
        CodexWssStreamState {
            websocket: Some(websocket),
            normalizer: CodexSseNormalizer::default(),
            done: false,
            state: state.clone(),
            pool_key,
            wss_url: wss_url.to_string(),
            body: body.clone(),
            request_payload,
            retry_stale_pool: reused,
            emitted: false,
        },
        |mut stream_state| async move {
            if stream_state.done {
                return None;
            }
            let mut websocket = stream_state.websocket.take()?;
            loop {
                let message = match websocket.next().await {
                    Ok(Some(message)) => message,
                    Ok(None) => {
                        let bytes = stream_state.normalizer.finish();
                        if bytes.is_empty()
                            && stream_state.retry_stale_pool
                            && !stream_state.emitted
                        {
                            match reconnect_stale_codex_wss(&mut stream_state).await {
                                Ok(reconnected) => {
                                    websocket = reconnected;
                                    continue;
                                }
                                Err(err) => {
                                    stream_state.done = true;
                                    return Some((Err(err), stream_state));
                                }
                            }
                        }
                        stream_state.done = true;
                        if bytes.is_empty() {
                            return None;
                        }
                        stream_state.emitted = true;
                        return Some((Ok(bytes), stream_state));
                    }
                    Err(err) => {
                        if stream_state.retry_stale_pool && !stream_state.emitted {
                            match reconnect_stale_codex_wss(&mut stream_state).await {
                                Ok(reconnected) => {
                                    websocket = reconnected;
                                    continue;
                                }
                                Err(retry_err) => {
                                    stream_state.done = true;
                                    return Some((Err(retry_err), stream_state));
                                }
                            }
                        }
                        stream_state.done = true;
                        return Some((
                            Err(AppError::Upstream(format!("Codex WSS read failed: {err}"))),
                            stream_state,
                        ));
                    }
                };
                match message {
                    Message::Text(text) => {
                        let terminal = contains_terminal_response_event(&text);
                        let bytes = stream_state.normalizer.push_response_text(&text);
                        if terminal {
                            stream_state
                                .state
                                .store_codex_ws(stream_state.pool_key.clone(), websocket)
                                .await;
                            stream_state.done = true;
                            return Some((
                                Ok(join_chunks(bytes, stream_state.normalizer.finish())),
                                stream_state,
                            ));
                        }
                        if !bytes.is_empty() {
                            stream_state.websocket = Some(websocket);
                            stream_state.emitted = true;
                            return Some((Ok(bytes), stream_state));
                        }
                    }
                    Message::Binary(bytes) => {
                        let text = String::from_utf8_lossy(&bytes);
                        let terminal = contains_terminal_response_event(&text);
                        let bytes = stream_state.normalizer.push_response_text(&text);
                        if terminal {
                            stream_state
                                .state
                                .store_codex_ws(stream_state.pool_key.clone(), websocket)
                                .await;
                            stream_state.done = true;
                            return Some((
                                Ok(join_chunks(bytes, stream_state.normalizer.finish())),
                                stream_state,
                            ));
                        }
                        if !bytes.is_empty() {
                            stream_state.websocket = Some(websocket);
                            stream_state.emitted = true;
                            return Some((Ok(bytes), stream_state));
                        }
                    }
                    Message::Ping(_) | Message::Pong(_) | Message::Close(_) => {}
                }
            }
        },
    )))
}

pub async fn connect_responses_wss(
    state: &AppState,
    wss_url: &str,
) -> AppResult<warpsock::WebSocket> {
    match connect_responses_wss_once(state, wss_url, None).await {
        Err(err) if maybe_auth_failure(&err) => {
            refresh_codex_auth(state).await?;
            connect_responses_wss_once(state, wss_url, None).await
        }
        result => result,
    }
}

struct CodexWsCheckout {
    websocket: warpsock::WebSocket,
    pool_key: String,
    reused: bool,
}

async fn checkout_responses_wss_for_body(
    state: &AppState,
    wss_url: &str,
    body: &serde_json::Value,
) -> AppResult<CodexWsCheckout> {
    let headers = codex_headers_for_body(state, body)?;
    let pool_key = codex_ws_pool_key(wss_url, &headers)?;
    if let Some(websocket) = state.take_codex_ws(&pool_key).await {
        return Ok(CodexWsCheckout {
            websocket,
            pool_key,
            reused: true,
        });
    }

    match connect_responses_wss_once_with_headers(state, wss_url, &headers).await {
        Err(err) if maybe_auth_failure(&err) => {
            refresh_codex_auth(state).await?;
            let headers = codex_headers_for_body(state, body)?;
            let pool_key = codex_ws_pool_key(wss_url, &headers)?;
            let websocket =
                connect_responses_wss_once_with_headers(state, wss_url, &headers).await?;
            Ok(CodexWsCheckout {
                websocket,
                pool_key,
                reused: false,
            })
        }
        Ok(websocket) => Ok(CodexWsCheckout {
            websocket,
            pool_key,
            reused: false,
        }),
        Err(err) => Err(err),
    }
}

struct CodexWssStreamState {
    websocket: Option<warpsock::WebSocket>,
    normalizer: CodexSseNormalizer,
    done: bool,
    state: AppState,
    pool_key: String,
    wss_url: String,
    body: serde_json::Value,
    request_payload: String,
    retry_stale_pool: bool,
    emitted: bool,
}

async fn reconnect_stale_codex_wss(
    stream_state: &mut CodexWssStreamState,
) -> AppResult<warpsock::WebSocket> {
    stream_state.retry_stale_pool = false;
    let checkout = checkout_responses_wss_for_body(
        &stream_state.state,
        &stream_state.wss_url,
        &stream_state.body,
    )
    .await?;
    let mut websocket = checkout.websocket;
    stream_state.pool_key = checkout.pool_key;
    websocket
        .send_text(stream_state.request_payload.clone())
        .await
        .map_err(|err| AppError::Upstream(format!("Codex WSS send failed: {err}")))?;
    Ok(websocket)
}

async fn connect_responses_wss_once(
    state: &AppState,
    wss_url: &str,
    body: Option<&serde_json::Value>,
) -> AppResult<warpsock::WebSocket> {
    let headers = match body {
        Some(body) => codex_headers_for_body(state, body)?,
        None => codex_headers(state)?,
    };
    connect_responses_wss_once_with_headers(state, wss_url, &headers).await
}

async fn connect_responses_wss_once_with_headers(
    state: &AppState,
    wss_url: &str,
    headers: &HeaderMap,
) -> AppResult<warpsock::WebSocket> {
    // Hold the permit across the WSS handshake so the concurrency limiter
    // bounds real in-flight connection starts rather than just preflight checks.
    let _permit = state.codex_acquire_handshake().await?;

    rate_limit::parse_codex_ws_protocol(Some("rfc6455")).map_err(AppError::BadRequest)?;
    let mut builder = state
        .warpsock
        .websocket(wss_url)
        .connect_timeout(state.runtime.codex_wss_connect_timeout);
    for (name, value) in headers.iter() {
        builder = builder.header(
            name.as_str(),
            value
                .to_str()
                .map_err(|_| AppError::BadRequest(format!("invalid Codex header: {name}")))?,
        );
    }

    builder
        .connect()
        .await
        .map_err(|err| AppError::Upstream(format!("Codex WSS handshake failed: {err}")))
}

fn codex_ws_pool_key(wss_url: &str, headers: &HeaderMap) -> AppResult<String> {
    let mut hasher = Sha256::new();
    hasher.update(wss_url.as_bytes());
    let mut header_pairs = Vec::with_capacity(headers.len());
    for (name, value) in headers.iter() {
        header_pairs.push((
            name.as_str(),
            value
                .to_str()
                .map_err(|_| AppError::BadRequest(format!("invalid Codex header: {name}")))?,
        ));
    }
    header_pairs.sort_unstable_by(|left, right| left.0.cmp(right.0));
    for (name, value) in header_pairs {
        hasher.update(name.as_bytes());
        hasher.update([0]);
        hasher.update(value.as_bytes());
        hasher.update([0xff]);
    }
    Ok(hex::encode(hasher.finalize()))
}

async fn send_http_with_refresh(
    state: &AppState,
    http_url: &str,
    body: &serde_json::Value,
) -> AppResult<Bytes> {
    let first = send_http(state, http_url, body).await?;
    if first.status == StatusCode::UNAUTHORIZED {
        refresh_codex_auth(state).await?;
        return send_http(state, http_url, body).await?.into_result_bytes();
    }
    first.into_result_bytes()
}

async fn send_http_stream_with_refresh(
    state: &AppState,
    http_url: &str,
    body: &serde_json::Value,
) -> AppResult<CodexResponseStream> {
    let first = send_http_stream(state, http_url, body).await?;
    if first.status() == StatusCode::UNAUTHORIZED {
        refresh_codex_auth(state).await?;
        let second = send_http_stream(state, http_url, body).await?;
        return response_to_stream(second).await;
    }
    response_to_stream(first).await
}

async fn send_http(
    state: &AppState,
    http_url: &str,
    body: &serde_json::Value,
) -> AppResult<CodexHttpResponse> {
    // Hold the permit across the HTTP call start.
    let _permit = state.codex_acquire_handshake().await?;
    let headers = codex_headers_for_body(state, body)?;
    let client = state.warpsock.clone();
    let request_url = http_url.to_string();
    let request_body = body.clone();
    let response = client
        .post(request_url)
        .headers(headers)
        .json(&request_body)
        .send()
        .await
        .map_err(|err| AppError::Upstream(format!("Codex HTTP request failed: {err}")))?;
    observe_warpsock_response(CODEX_PROVIDER, &response);
    let status = response.status();
    let bytes = response
        .bytes()
        .map_err(|err| AppError::Upstream(format!("Codex HTTP response failed: {err}")))?;
    Ok(CodexHttpResponse { status, bytes })
}

async fn send_http_stream(
    state: &AppState,
    http_url: &str,
    body: &serde_json::Value,
) -> AppResult<CodexStreamResponse> {
    // Hold the permit across the HTTP streaming request start.
    let _permit = state.codex_acquire_handshake().await?;
    let headers = codex_headers_for_body(state, body)?;
    let stream_client = state.warpsock.clone();
    let stream_url = http_url.to_string();
    let stream_body = body.clone();
    let request = stream_client
        .post(stream_url)
        .headers(headers)
        .json(&stream_body)
        .version(HttpVersion::Http2);
    match Box::pin(request.send_streaming()).await {
        Ok(response) => Ok(CodexStreamResponse::Streaming(Box::new(response))),
        Err(error) if is_non_h2_streaming_error(&error) => {
            let fallback_client = state.warpsock.clone();
            let fallback_url = http_url.to_string();
            let headers = codex_headers_for_body(state, body)?;
            let fallback_body = body.clone();
            let request = fallback_client
                .post(fallback_url)
                .headers(headers)
                .json(&fallback_body);
            let response = Box::pin(request.send())
                .await
                .map_err(|err| AppError::Upstream(format!("Codex HTTP request failed: {err}")))?;
            observe_warpsock_response(CODEX_PROVIDER, &response);
            Ok(CodexStreamResponse::Buffered(Box::new(response)))
        }
        Err(error) => Err(AppError::Upstream(format!(
            "Codex HTTP request failed: {error}"
        ))),
    }
}

async fn response_to_stream(response: CodexStreamResponse) -> AppResult<CodexResponseStream> {
    match response {
        CodexStreamResponse::Buffered(response) => {
            let status = response.status();
            let body = response
                .bytes()
                .map_err(|err| AppError::Upstream(format!("Codex HTTP response failed: {err}")))?;
            if !status.is_success() {
                return Err(AppError::Upstream(format!(
                    "Codex HTTP returned {status}: {}",
                    String::from_utf8_lossy(&body)
                )));
            }
            let normalized = normalize_sse(String::from_utf8_lossy(&body).into_owned());
            Ok(Box::pin(stream::once(async move { Ok(normalized) })))
        }
        CodexStreamResponse::Streaming(response) => {
            let status = response.status();
            let body = response.into_body();
            if !status.is_success() {
                let body = collect_warpsock_body(body, "Codex HTTP stream failed").await?;
                return Err(AppError::Upstream(format!(
                    "Codex HTTP returned {status}: {}",
                    String::from_utf8_lossy(&body)
                )));
            }
            Ok(normalize_sse_stream(warpsock_body_stream(
                body,
                "Codex HTTP stream failed",
            )))
        }
    }
}

fn has_remote_compaction_v2_trigger(value: &serde_json::Value) -> bool {
    match value {
        Value::Object(object) => {
            if object.get("type").and_then(Value::as_str) == Some("context_compaction")
                && object.get("encrypted_content").is_none_or(Value::is_null)
            {
                return true;
            }
            object.values().any(has_remote_compaction_v2_trigger)
        }
        Value::Array(values) => values.iter().any(has_remote_compaction_v2_trigger),
        _ => false,
    }
}

struct CodexHttpResponse {
    status: StatusCode,
    bytes: Bytes,
}

impl CodexHttpResponse {
    fn into_result_bytes(self) -> AppResult<Bytes> {
        if !self.status.is_success() {
            return Err(AppError::Upstream(format!(
                "Codex HTTP returned {}",
                self.status
            )));
        }
        Ok(normalize_sse(
            String::from_utf8_lossy(&self.bytes).into_owned(),
        ))
    }
}

fn normalize_sse(input: String) -> Bytes {
    Bytes::from(splice_completed_event_filtered(&input))
}

fn normalize_sse_stream(input: BoxStream<'static, AppResult<Bytes>>) -> CodexResponseStream {
    Box::pin(stream::unfold(
        (input, CodexSseNormalizer::default(), false),
        |(mut input, mut normalizer, done)| async move {
            if done {
                return None;
            }
            loop {
                match input.next().await {
                    Some(Ok(bytes)) => {
                        let text = String::from_utf8_lossy(&bytes);
                        let bytes = normalizer.push_sse_text(&text);
                        if bytes.is_empty() {
                            continue;
                        }
                        return Some((Ok(bytes), (input, normalizer, false)));
                    }
                    Some(Err(err)) => {
                        return Some((Err(err), (input, normalizer, true)));
                    }
                    None => {
                        let bytes = normalizer.finish();
                        if bytes.is_empty() {
                            return None;
                        }
                        return Some((Ok(bytes), (input, normalizer, true)));
                    }
                }
            }
        },
    ))
}

enum CodexStreamResponse {
    Buffered(Box<warpsock::Response>),
    Streaming(Box<warpsock::Response>),
}

impl CodexStreamResponse {
    fn status(&self) -> StatusCode {
        match self {
            Self::Buffered(response) | Self::Streaming(response) => response.status(),
        }
    }
}

fn is_non_h2_streaming_error(error: &warpsock::Error) -> bool {
    matches!(error, warpsock::Error::HttpProtocol(message) if message.contains("Expected h2 ALPN"))
}

#[derive(Default)]
struct CodexSseNormalizer {
    pending: String,
    output_items: Vec<serde_json::Value>,
}

impl CodexSseNormalizer {
    fn push_response_text(&mut self, text: &str) -> Bytes {
        let mut chunk = String::new();
        append_response_stream_chunk(&mut chunk, text);
        self.push_sse_text(&chunk)
    }

    fn push_sse_text(&mut self, text: &str) -> Bytes {
        self.pending.push_str(text);
        let mut output = String::new();
        while let Some(index) = self.pending.find("\n\n") {
            let consumed = index + 2;
            output.push_str(&process_sse_block(
                &self.pending[..consumed],
                &mut self.output_items,
            ));
            let _ = self.pending.drain(..consumed);
        }
        Bytes::from(output)
    }

    fn finish(&mut self) -> Bytes {
        if self.pending.is_empty() {
            return Bytes::new();
        }
        let block = std::mem::take(&mut self.pending);
        Bytes::from(process_sse_block(&block, &mut self.output_items))
    }
}

fn process_sse_block(block: &str, output_items: &mut Vec<serde_json::Value>) -> String {
    let Some(event) = sse_event_name(block) else {
        return block.to_string();
    };
    if event.starts_with("codex.") {
        return String::new();
    }
    match event {
        "response.output_item.done" => {
            if let Some(item) = sse_event_data_json(block).and_then(extract_output_item) {
                output_items.push(item);
            }
            block.to_string()
        }
        "response.completed" if !output_items.is_empty() => {
            let Some(mut data) = sse_event_data_json(block) else {
                return block.to_string();
            };
            splice_output_items(&mut data, output_items);
            rewrite_sse_data(block, &data)
        }
        _ => block.to_string(),
    }
}

fn join_chunks(first: Bytes, second: Bytes) -> Bytes {
    if first.is_empty() {
        return second;
    }
    if second.is_empty() {
        return first;
    }
    let mut output = Vec::with_capacity(first.len() + second.len());
    output.extend_from_slice(&first);
    output.extend_from_slice(&second);
    Bytes::from(output)
}

fn sse_event_name(block: &str) -> Option<&str> {
    block
        .lines()
        .find_map(|line| line.strip_prefix("event:").map(str::trim_start))
}

fn sse_event_data_json(block: &str) -> Option<serde_json::Value> {
    let mut data = String::new();
    for line in block.lines() {
        if let Some(value) = line.strip_prefix("data:") {
            if !data.is_empty() {
                data.push('\n');
            }
            data.push_str(value.trim_start());
        }
    }
    serde_json::from_str(&data).ok()
}

fn extract_output_item(mut data: serde_json::Value) -> Option<serde_json::Value> {
    data.get_mut("item").map(std::mem::take).or_else(|| {
        if data.get("type").is_some() || data.get("id").is_some() {
            Some(data)
        } else {
            None
        }
    })
}

fn splice_output_items(data: &mut serde_json::Value, output_items: &[serde_json::Value]) {
    let items = serde_json::Value::Array(output_items.to_vec());
    if let Some(response) = data
        .get_mut("response")
        .and_then(|value| value.as_object_mut())
    {
        response.insert("output".into(), items);
    } else if let Some(response) = data.as_object_mut() {
        response.insert("output".into(), items);
    }
}

fn rewrite_sse_data(block: &str, data: &serde_json::Value) -> String {
    let mut rewritten = String::new();
    let mut wrote_data = false;
    for line in block.lines() {
        if line.strip_prefix("data:").is_some() {
            if !wrote_data {
                rewritten.push_str("data: ");
                rewritten.push_str(&data.to_string());
                rewritten.push('\n');
                wrote_data = true;
            }
        } else {
            rewritten.push_str(line);
            rewritten.push('\n');
        }
    }
    rewritten
}

fn append_response_stream_chunk(stream: &mut String, text: &str) {
    if let Some(event_type) = response_event_type(text) {
        stream.push_str("event: ");
        stream.push_str(&event_type);
        stream.push('\n');
        stream.push_str("data: ");
        stream.push_str(text);
        stream.push_str("\n\n");
    } else {
        stream.push_str(text);
    }
}

fn contains_terminal_response_event(text: &str) -> bool {
    let terminal = |event_type: &str| {
        matches!(
            event_type,
            "response.completed" | "response.failed" | "response.incomplete"
        )
    };
    if text.lines().any(|line| {
        line.strip_prefix("event:")
            .map(str::trim)
            .is_some_and(terminal)
    }) {
        return true;
    }
    response_event_type(text).as_deref().is_some_and(terminal)
}

fn response_event_type(text: &str) -> Option<Cow<'_, str>> {
    #[derive(serde::Deserialize)]
    struct EventType<'a> {
        #[serde(borrow)]
        #[serde(default)]
        r#type: Option<Cow<'a, str>>,
        #[serde(borrow)]
        #[serde(default)]
        event: Option<Cow<'a, str>>,
    }

    serde_json::from_str::<EventType<'_>>(text)
        .ok()
        .and_then(|value| value.r#type.or(value.event))
}

fn maybe_auth_failure(err: &AppError) -> bool {
    err.to_string().contains("401") || err.to_string().contains("Unauthorized")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn resolve_test_model(model: &str) -> AppResult<ResolvedModel> {
        if model == "client-model" {
            Ok(ResolvedModel {
                provider: Provider::Codex,
                upstream_model: "upstream-model".to_string(),
            })
        } else {
            Err(AppError::ModelNotSupported(model.to_string()))
        }
    }

    #[test]
    fn flat_response_create_frame_keeps_flat_shape() {
        let payload = prepare_response_create_event_payload_with_resolver(
            json!({
                "type": "response.create",
                "model": "client-model",
                "input": "hello",
                "stream": true
            }),
            resolve_test_model,
        )
        .unwrap();

        assert_eq!(payload["type"], "response.create");
        assert_eq!(payload["model"], "upstream-model");
        assert_eq!(payload["input"][0]["content"][0]["text"], "hello");
        assert!(payload.get("response").is_none());
        assert_eq!(payload["stream"], true);
        assert_eq!(payload["store"], false);
    }

    #[test]
    fn nested_response_create_frame_is_flattened() {
        let payload = prepare_response_create_event_payload_with_resolver(
            json!({
                "type": "response.create",
                "response": {
                    "model": "client-model",
                    "input": "hello",
                    "stream": true
                }
            }),
            resolve_test_model,
        )
        .unwrap();

        assert_eq!(payload["type"], "response.create");
        assert_eq!(payload["model"], "upstream-model");
        assert_eq!(payload["input"][0]["content"][0]["text"], "hello");
        assert_eq!(payload["stream"], true);
        assert_eq!(payload["store"], false);
        assert!(payload.get("response").is_none());
    }

    #[test]
    fn raw_responses_body_becomes_flat_response_create_frame() {
        let payload = prepare_response_create_event_payload_with_resolver(
            json!({
                "model": "client-model",
                "input": "hello"
            }),
            resolve_test_model,
        )
        .unwrap();

        assert_eq!(payload["type"], "response.create");
        assert_eq!(payload["model"], "upstream-model");
        assert_eq!(payload["input"][0]["content"][0]["text"], "hello");
        assert!(payload.get("response").is_none());
    }

    #[test]
    fn nested_responses_body_wraps_only_through_explicit_compatibility_helper() {
        let payload = prepare_nested_response_create_payload_with_resolver(
            json!({
                "model": "client-model",
                "input": "hello"
            }),
            resolve_test_model,
        )
        .unwrap();

        assert_eq!(payload["type"], "response.create");
        assert_eq!(payload["response"]["model"], "upstream-model");
        assert_eq!(
            payload["response"]["input"][0]["content"][0]["text"],
            "hello"
        );
    }

    #[test]
    fn detects_terminal_response_events_from_json_and_sse() {
        assert!(contains_terminal_response_event(
            r#"{"type":"response.completed","response":{"id":"resp_1"}}"#
        ));
        assert!(contains_terminal_response_event(
            "event: response.failed\ndata: {\"response\":{\"id\":\"resp_1\"}}\n\n"
        ));
        assert!(!contains_terminal_response_event(
            r#"{"type":"response.output_text.delta","delta":"hi"}"#
        ));
    }

    #[test]
    fn projects_wss_json_events_to_sse_chunks() {
        let mut stream = String::new();
        append_response_stream_chunk(
            &mut stream,
            r#"{"type":"response.output_text.delta","delta":"OK"}"#,
        );
        assert_eq!(
            stream,
            "event: response.output_text.delta\n\
data: {\"type\":\"response.output_text.delta\",\"delta\":\"OK\"}\n\n"
        );
    }

    #[test]
    fn serialize_flat_response_create_event_matches_value_round_trip() {
        let body = json!({
            "model": "upstream-model",
            "input": [
                {"role": "user", "content": [{"type": "input_text", "text": "hi"}]}
            ],
            "stream": true,
            "store": false
        });

        let from_value = flat_response_create_event(body.clone()).to_string();
        let direct = serialize_flat_response_create_event(&body);

        let parsed_value: Value = serde_json::from_str(&from_value).unwrap();
        let parsed_direct: Value = serde_json::from_str(&direct).unwrap();
        assert_eq!(parsed_value, parsed_direct);
        assert_eq!(parsed_direct["type"], "response.create");
        assert_eq!(parsed_direct["model"], "upstream-model");
        assert_eq!(parsed_direct["stream"], true);
    }
}
