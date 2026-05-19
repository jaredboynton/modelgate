use axum::http::{HeaderMap, StatusCode};
use bytes::Bytes;
use serde_json::{json, Value};
use specter::Message;

use crate::{
    auth::codex::{load_codex_auth, refresh_codex_auth, CODEX_OPENAI_BETA, CODEX_ORIGINATOR},
    model_alias::{self, Provider, ResolvedModel},
    rate_limit,
    sse::{filter::filter_codex_events, splice::splice_completed_event},
    state::CodexTransport,
    AppError, AppResult, AppState,
};

pub const CODEX_RESPONSES_WSS_URL: &str = "wss://chatgpt.com/backend-api/codex/responses";
pub const CODEX_RESPONSES_HTTP_URL: &str = "https://chatgpt.com/backend-api/codex/responses";
const CODEX_REMOTE_COMPACTION_V2_FEATURE: &str = "remote_compaction_v2";

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
        object.remove("prompt_cache_retention");
        reject_lossy_request_fields(object)?;
        reject_unsupported_input_items(object.get("input"))?;
        reject_unsupported_tools(object.get("tools"))?;

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

fn reject_lossy_request_fields(
    object: &serde_json::Map<String, serde_json::Value>,
) -> AppResult<()> {
    for field in [
        "background",
        "context_management",
        "conversation",
        "max_tool_calls",
        "max_output_tokens",
        "max_tokens",
        "prompt",
        "top_logprobs",
        "truncation",
    ] {
        if object.contains_key(field) {
            return Err(AppError::BadRequest(format!(
                "{field} is not supported for Codex responses"
            )));
        }
    }
    if object.get("store").and_then(Value::as_bool) == Some(true) {
        return Err(AppError::BadRequest(
            "store:true is not supported for Codex responses".into(),
        ));
    }
    Ok(())
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
    for item_type in [
        "input_audio",
        "input_file",
        "file",
        "localImage",
        "local_image",
        "image_asset_pointer",
        "image_asset_pointer_citation",
    ] {
        if contains_typed_item(input, item_type) {
            return Err(AppError::BadRequest(format!(
                "{item_type} is not supported for Codex responses"
            )));
        }
    }
    if contains_input_image_file_id(input) {
        return Err(AppError::BadRequest(
            "input_image.file_id is not supported for Codex responses".into(),
        ));
    }
    Ok(())
}

fn reject_unsupported_tools(tools: Option<&serde_json::Value>) -> AppResult<()> {
    for tool_type in [
        "apply_patch",
        "file_search",
        "code_interpreter",
        "mcp",
        "shell",
        "local_shell",
        "computer",
    ] {
        if contains_typed_item(tools, tool_type) {
            return Err(AppError::BadRequest(format!(
                "{tool_type} tool is not supported for Codex responses"
            )));
        }
    }
    Ok(())
}

fn contains_typed_item(value: Option<&serde_json::Value>, item_type: &str) -> bool {
    match value {
        Some(Value::Object(object)) => {
            object
                .get("type")
                .and_then(Value::as_str)
                .is_some_and(|value| value == item_type)
                || object
                    .values()
                    .any(|value| contains_typed_item(Some(value), item_type))
        }
        Some(Value::Array(values)) => values
            .iter()
            .any(|value| contains_typed_item(Some(value), item_type)),
        _ => false,
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

    let object = body
        .as_object_mut()
        .ok_or_else(|| AppError::BadRequest("response.create must be an object".into()))?;
    object.remove("type");
    let prepared = prepare_responses_body_with_resolver(Value::Object(object.clone()), resolve)?;
    Ok(flat_response_create_event(prepared))
}

fn flat_response_create_event(prepared: serde_json::Value) -> serde_json::Value {
    let mut event = serde_json::Map::new();
    event.insert(
        "type".into(),
        serde_json::Value::String("response.create".into()),
    );
    if let Some(response) = prepared.as_object() {
        event.extend(response.clone());
    }
    serde_json::Value::Object(event)
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

async fn send_wss(state: &AppState, wss_url: &str, body: &serde_json::Value) -> AppResult<Bytes> {
    let mut ws = connect_responses_wss_for_body(state, wss_url, body).await?;
    ws.send_text(flat_response_create_event(body.clone()).to_string())
        .await
        .map_err(|err| AppError::Upstream(format!("Codex WSS send failed: {err}")))?;

    let mut stream = String::new();
    while let Some(message) = ws
        .next()
        .await
        .map_err(|err| AppError::Upstream(format!("Codex WSS read failed: {err}")))?
    {
        match message {
            Message::Text(text) => {
                let terminal = contains_terminal_response_event(&text);
                append_response_stream_chunk(&mut stream, &text);
                if terminal {
                    break;
                }
            }
            Message::Binary(bytes) => {
                let text = String::from_utf8_lossy(&bytes);
                let terminal = contains_terminal_response_event(&text);
                append_response_stream_chunk(&mut stream, &text);
                if terminal {
                    break;
                }
            }
            Message::Ping(_) | Message::Pong(_) | Message::Close(_) => {}
        }
    }
    Ok(normalize_sse(stream))
}

pub async fn connect_responses_wss(
    state: &AppState,
    wss_url: &str,
) -> AppResult<specter::WebSocket> {
    match connect_responses_wss_once(state, wss_url, None).await {
        Err(err) if maybe_auth_failure(&err) => {
            refresh_codex_auth(state).await?;
            connect_responses_wss_once(state, wss_url, None).await
        }
        result => result,
    }
}

async fn connect_responses_wss_for_body(
    state: &AppState,
    wss_url: &str,
    body: &serde_json::Value,
) -> AppResult<specter::WebSocket> {
    match connect_responses_wss_once(state, wss_url, Some(body)).await {
        Err(err) if maybe_auth_failure(&err) => {
            refresh_codex_auth(state).await?;
            connect_responses_wss_once(state, wss_url, Some(body)).await
        }
        result => result,
    }
}

async fn connect_responses_wss_once(
    state: &AppState,
    wss_url: &str,
    body: Option<&serde_json::Value>,
) -> AppResult<specter::WebSocket> {
    rate_limit::parse_codex_ws_protocol(Some("rfc6455")).map_err(AppError::BadRequest)?;
    let headers = match body {
        Some(body) => codex_headers_for_body(state, body)?,
        None => codex_headers(state)?,
    };
    let mut builder = state
        .specter
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

async fn send_http(
    state: &AppState,
    http_url: &str,
    body: &serde_json::Value,
) -> AppResult<CodexHttpResponse> {
    let headers = codex_headers_for_body(state, body)?;
    let response = state
        .specter
        .post(http_url)
        .headers(specter::Headers::from(headers))
        .json(body)
        .send()
        .await
        .map_err(|err| AppError::Upstream(format!("Codex HTTP request failed: {err}")))?;
    let status = response.status();
    let bytes = response.into_body();
    Ok(CodexHttpResponse { status, bytes })
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
    Bytes::from(filter_codex_events(&splice_completed_event(&input)))
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

fn response_event_type(text: &str) -> Option<String> {
    serde_json::from_str::<Value>(text).ok().and_then(|value| {
        value
            .get("type")
            .or_else(|| value.get("event"))
            .and_then(Value::as_str)
            .map(ToOwned::to_owned)
    })
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
}
