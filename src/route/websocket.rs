use std::{borrow::Cow, collections::VecDeque, time::Duration};

use axum::{
    body::to_bytes,
    extract::{
        ws::{close_code, CloseFrame, Message, WebSocket, WebSocketUpgrade},
        State,
    },
    http::{header, HeaderMap, Method, StatusCode, Uri},
    response::IntoResponse,
};
use futures::{SinkExt, StreamExt};
use serde_json::{json, Value};
use specter::Message as UpstreamMessage;
use tokio::{sync::mpsc, time::Instant};

use crate::{
    adapter::responses_sse::ResponsesSseParser,
    compaction::{
        find_compaction_carriers, prepare_responses_input_for_target, CompactionHttpError,
        CompactionLimits, CompactionPackContext, RemoteCompactionPolicy,
    },
    model_alias::{Provider, ResolvedTarget, TargetFormat},
    route::{
        dispatch::{plan_with_state, DispatchAction, RequestFormat},
        models::validate_codex_catalog_websocket_request,
        responses::ResponsesRoute,
        responses_compaction::{
            context_compaction_unavailable_frame, is_v2_context_compaction_trigger,
            proxy_visible_context_compaction_item,
        },
        responses_executor::{execute_responses_request, ExecuteResponsesOptions},
    },
    upstream, AppError, AppResult, AppState, UpstreamResponse,
};

pub const REALTIME_WS_HANDSHAKE_TIMEOUT_MS: u64 = 5000;
pub const REALTIME_WS_HEARTBEAT_TIMEOUT_MS: u64 = 30000;
pub const REALTIME_WS_MAX_FRAME_BYTES: usize = 16 * 1024 * 1024;
pub const REALTIME_WS_QUEUE_CAPACITY: usize = 32;
pub const REALTIME_WS_QUEUE_SATURATION_TIMEOUT_MS: u64 = 2000;

const OPENAI_PUBLIC_REALTIME_MODEL: &str = "gpt-realtime-2";
const MISSING_CODEX_AUTH_MESSAGE: &str = "Missing Codex OAuth credentials at ~/.codex/auth.json.";

pub async fn responses_ws(
    State(state): State<AppState>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| async move {
        proxy_responses_websocket(state, socket).await;
    })
}

pub async fn realtime_ws(
    State(state): State<AppState>,
    uri: Uri,
    headers: HeaderMap,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    let query_result = validate_realtime_model_query(uri.query());
    ws.on_upgrade(move |socket| async move {
        proxy_realtime_websocket(state, headers, socket, query_result).await;
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RealtimeRoute {
    Realtime,
    Responses,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RealtimeEventPolicy {
    Accept,
    Reject,
    Ignore,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CodexBearerPublicRoutePolicy {
    Allow,
    Reject,
    RouteAway,
}

pub fn normalize_gpt_realtime_2_response_create(mut value: Value) -> AppResult<Value> {
    if value.get("type").and_then(Value::as_str) != Some("response.create") {
        return Ok(value);
    }

    let Some(response) = value.get_mut("response") else {
        return Ok(value);
    };
    let Some(response) = response.as_object_mut() else {
        return Err(AppError::BadRequest("invalid realtime response".into()));
    };

    if response.contains_key("modalities") && response.contains_key("output_modalities") {
        return Err(AppError::BadRequest(
            "response.modalities conflicts with response.output_modalities".into(),
        ));
    }

    if let Some(modalities) = response.remove("modalities") {
        response.insert("output_modalities".into(), modalities);
    }

    Ok(value)
}

pub fn realtime_headers_for_model(model: &str, headers: &mut HeaderMap) -> AppResult<()> {
    if model == "gpt-realtime-2"
        && headers
            .get("OpenAI-Beta")
            .and_then(|value| value.to_str().ok())
            .is_some_and(|value| value == "realtime=v1")
    {
        headers.remove("OpenAI-Beta");
    }
    Ok(())
}

pub fn realtime_event_policy(
    route: RealtimeRoute,
    event: &Value,
) -> AppResult<RealtimeEventPolicy> {
    let Some(event_type) = event.get("type").and_then(Value::as_str) else {
        return Err(AppError::BadRequest("missing realtime event type".into()));
    };

    match (route, event_type) {
        (
            RealtimeRoute::Realtime,
            "response.output_text.delta" | "response.output_text.done" | "response.done",
        ) => Ok(RealtimeEventPolicy::Accept),
        (RealtimeRoute::Responses, "response.done") => Ok(RealtimeEventPolicy::Reject),
        _ => Ok(RealtimeEventPolicy::Ignore),
    }
}

pub fn codex_bearer_public_route_policy(
    method: &Method,
    path: &str,
) -> CodexBearerPublicRoutePolicy {
    match (method, path) {
        (&Method::GET, "/v1/models") => CodexBearerPublicRoutePolicy::RouteAway,
        (&Method::POST, "/v1/responses") | (&Method::POST, "/v1/audio/speech") => {
            CodexBearerPublicRoutePolicy::Reject
        }
        (&Method::GET, "/v1/realtime") | (&Method::POST, "/v1/realtime/transcription_sessions") => {
            CodexBearerPublicRoutePolicy::Allow
        }
        _ => CodexBearerPublicRoutePolicy::Reject,
    }
}

async fn proxy_responses_websocket(state: AppState, mut client: WebSocket) {
    if let Err(error) = dispatch_responses_websocket(&state, &mut client).await {
        tracing::warn!(error = %error, "Responses WebSocket passthrough failed");
        let _ = client
            .send(Message::Text(
                websocket_request_error(StatusCode::BAD_REQUEST, "websocket_proxy_error", &error)
                    .to_string(),
            ))
            .await;
        let _ = client
            .send(Message::Close(Some(CloseFrame {
                code: close_code_for_error(&error),
                reason: Cow::Owned("Responses WebSocket passthrough failed".to_string()),
            })))
            .await;
    }
}

async fn proxy_realtime_websocket(
    state: AppState,
    inbound_headers: HeaderMap,
    client: WebSocket,
    query_result: Result<&'static str, RealtimeWsLocalError>,
) {
    if let Err(error) =
        dispatch_realtime_websocket(&state, &inbound_headers, client, query_result).await
    {
        tracing::warn!(error = %error, "Realtime WebSocket passthrough failed");
    }
}

async fn dispatch_realtime_websocket(
    state: &AppState,
    inbound_headers: &HeaderMap,
    client: WebSocket,
    query_result: Result<&'static str, RealtimeWsLocalError>,
) -> AppResult<()> {
    let model = match query_result {
        Ok(model) => model,
        Err(error) => return send_realtime_error_and_close(client, error).await,
    };
    let upstream = match connect_public_openai_realtime_ws(state, inbound_headers, model).await {
        Ok(upstream) => upstream,
        Err(error) if matches!(error, AppError::MissingCredential(_)) => {
            return send_realtime_error_and_close(client, realtime_auth_error(error)).await;
        }
        Err(_error) => {
            return send_realtime_error_and_close(
                client,
                RealtimeWsLocalError::new(
                    StatusCode::BAD_GATEWAY,
                    "upstream_error",
                    "upstream_realtime_websocket_handshake_failed",
                    "Public OpenAI Realtime WebSocket handshake failed.",
                    close_code::ERROR,
                    "upstream realtime handshake failed",
                ),
            )
            .await;
        }
    };

    bridge_public_openai_realtime(client, upstream).await
}

async fn connect_public_openai_realtime_ws(
    state: &AppState,
    inbound_headers: &HeaderMap,
    model: &str,
) -> AppResult<specter::WebSocket> {
    debug_assert_eq!(model, OPENAI_PUBLIC_REALTIME_MODEL);
    let request =
        upstream::openai_realtime::build_realtime_ws_request(state, inbound_headers, model)?;
    let mut builder = state
        .specter
        .websocket(request.url)
        .connect_timeout(Duration::from_millis(REALTIME_WS_HANDSHAKE_TIMEOUT_MS))
        .handshake_timeout(Duration::from_millis(REALTIME_WS_HANDSHAKE_TIMEOUT_MS))
        .max_frame_size(REALTIME_WS_MAX_FRAME_BYTES)
        .max_message_size(REALTIME_WS_MAX_FRAME_BYTES);
    for (name, value) in request.headers.iter() {
        builder = builder.header(
            name.as_str(),
            value.to_str().map_err(|_| {
                AppError::BadRequest(format!("invalid public OpenAI realtime header: {name}"))
            })?,
        );
    }
    builder.connect().await.map_err(|error| {
        AppError::Upstream(format!(
            "Public OpenAI Realtime WebSocket handshake failed: {error}"
        ))
    })
}

async fn bridge_public_openai_realtime(
    client: WebSocket,
    mut upstream: specter::WebSocket,
) -> AppResult<()> {
    let (mut client_sender, mut client_receiver) = client.split();
    let heartbeat = tokio::time::sleep(Duration::from_millis(REALTIME_WS_HEARTBEAT_TIMEOUT_MS));
    tokio::pin!(heartbeat);

    loop {
        tokio::select! {
            _ = &mut heartbeat => {
                let error = RealtimeWsLocalError::new(
                    StatusCode::BAD_GATEWAY,
                    "upstream_error",
                    "realtime_heartbeat_timeout",
                    "Public OpenAI Realtime WebSocket heartbeat timed out.",
                    close_code::ERROR,
                    "upstream realtime heartbeat timeout",
                );
                send_realtime_error_to_client_sender(&mut client_sender, &error).await?;
                close_client_sender(&mut client_sender, error.close_code, error.close_reason).await?;
                let _ = upstream.close(Some(specter::CloseFrame {
                    code: specter::CloseCode::Error,
                    reason: "heartbeat timeout".to_string(),
                })).await;
                return Ok(());
            }
            client_message = client_receiver.next() => {
                let Some(client_message) = client_message else {
                    let _ = upstream.close(None).await;
                    return Ok(());
                };
                let client_message = client_message.map_err(|error| {
                    AppError::Upstream(format!("client Realtime WebSocket read failed: {error}"))
                })?;
                if forward_realtime_client_message(client_message, &mut upstream, &mut client_sender).await? {
                    return Ok(());
                }
            }
            upstream_message = upstream.next() => {
                heartbeat.as_mut().reset(Instant::now() + Duration::from_millis(REALTIME_WS_HEARTBEAT_TIMEOUT_MS));
                match upstream_message {
                    Ok(Some(upstream_message)) => {
                        if forward_realtime_upstream_message(upstream_message, &mut client_sender).await? {
                            return Ok(());
                        }
                    }
                    Ok(None) => {
                        close_client_sender(&mut client_sender, close_code::NORMAL, "upstream realtime closed").await?;
                        return Ok(());
                    }
                    Err(_error) => {
                        let error = RealtimeWsLocalError::new(
                            StatusCode::BAD_GATEWAY,
                            "upstream_error",
                            "upstream_realtime_websocket_read_failed",
                            "Public OpenAI Realtime WebSocket read failed.",
                            close_code::ERROR,
                            "upstream realtime read failed",
                        );
                        send_realtime_error_to_client_sender(&mut client_sender, &error).await?;
                        close_client_sender(&mut client_sender, error.close_code, error.close_reason).await?;
                        return Ok(());
                    }
                }
            }
        }
    }
}

async fn forward_realtime_client_message(
    message: Message,
    upstream: &mut specter::WebSocket,
    client_sender: &mut futures::stream::SplitSink<WebSocket, Message>,
) -> AppResult<bool> {
    let upstream_message = match message {
        Message::Text(text) => {
            if text.len() > REALTIME_WS_MAX_FRAME_BYTES {
                send_realtime_oversize_and_close(client_sender, upstream).await?;
                return Ok(true);
            }
            UpstreamMessage::Text(text)
        }
        Message::Binary(bytes) => {
            if bytes.len() > REALTIME_WS_MAX_FRAME_BYTES {
                send_realtime_oversize_and_close(client_sender, upstream).await?;
                return Ok(true);
            }
            UpstreamMessage::Binary(bytes.into())
        }
        Message::Ping(bytes) => UpstreamMessage::Ping(bytes.into()),
        Message::Pong(bytes) => UpstreamMessage::Pong(bytes.into()),
        Message::Close(frame) => {
            let _ = upstream.close(to_upstream_close_frame(frame)).await;
            return Ok(true);
        }
    };

    send_upstream_with_saturation(upstream, upstream_message, client_sender).await?;
    Ok(false)
}

async fn forward_realtime_upstream_message(
    message: UpstreamMessage,
    client_sender: &mut futures::stream::SplitSink<WebSocket, Message>,
) -> AppResult<bool> {
    let client_message = match message {
        UpstreamMessage::Text(text) => Message::Text(text),
        UpstreamMessage::Binary(bytes) => Message::Binary(bytes.to_vec()),
        UpstreamMessage::Ping(bytes) => Message::Ping(bytes.to_vec()),
        UpstreamMessage::Pong(bytes) => Message::Pong(bytes.to_vec()),
        UpstreamMessage::Close(frame) => {
            close_client_sender(
                client_sender,
                close_code_from_upstream(frame.as_ref()),
                close_reason_from_upstream(frame.as_ref()),
            )
            .await?;
            return Ok(true);
        }
    };
    send_client_with_saturation(client_sender, client_message).await?;
    Ok(false)
}

async fn send_upstream_with_saturation(
    upstream: &mut specter::WebSocket,
    message: UpstreamMessage,
    client_sender: &mut futures::stream::SplitSink<WebSocket, Message>,
) -> AppResult<()> {
    match tokio::time::timeout(
        Duration::from_millis(REALTIME_WS_QUEUE_SATURATION_TIMEOUT_MS),
        upstream.send(message),
    )
    .await
    {
        Ok(Ok(())) => Ok(()),
        Ok(Err(error)) => Err(AppError::Upstream(format!(
            "Public OpenAI Realtime WebSocket send failed: {error}"
        ))),
        Err(_) => {
            let error = realtime_queue_saturated_error();
            send_realtime_error_to_client_sender(client_sender, &error).await?;
            close_client_sender(client_sender, error.close_code, error.close_reason).await
        }
    }
}

async fn send_client_with_saturation(
    client_sender: &mut futures::stream::SplitSink<WebSocket, Message>,
    message: Message,
) -> AppResult<()> {
    match tokio::time::timeout(
        Duration::from_millis(REALTIME_WS_QUEUE_SATURATION_TIMEOUT_MS),
        client_sender.send(message),
    )
    .await
    {
        Ok(Ok(())) => Ok(()),
        Ok(Err(error)) => Err(AppError::Upstream(format!(
            "client Realtime WebSocket send failed: {error}"
        ))),
        Err(_) => Err(AppError::Upstream(
            "client Realtime WebSocket send saturated".into(),
        )),
    }
}

async fn send_realtime_oversize_and_close(
    client_sender: &mut futures::stream::SplitSink<WebSocket, Message>,
    upstream: &mut specter::WebSocket,
) -> AppResult<()> {
    let error = RealtimeWsLocalError::new(
        StatusCode::PAYLOAD_TOO_LARGE,
        "invalid_request_error",
        "realtime_frame_too_large",
        "Realtime WebSocket frame exceeds the supported size limit.",
        close_code::SIZE,
        "realtime frame too large",
    );
    send_realtime_error_to_client_sender(client_sender, &error).await?;
    close_client_sender(client_sender, error.close_code, error.close_reason).await?;
    let _ = upstream
        .close(Some(specter::CloseFrame {
            code: specter::CloseCode::Size,
            reason: "frame too large".to_string(),
        }))
        .await;
    Ok(())
}

async fn send_realtime_error_and_close(
    client: WebSocket,
    error: RealtimeWsLocalError,
) -> AppResult<()> {
    let (mut client_sender, _client_receiver) = client.split();
    send_realtime_error_to_client_sender(&mut client_sender, &error).await?;
    close_client_sender(&mut client_sender, error.close_code, error.close_reason).await
}

async fn send_realtime_error_to_client_sender(
    client_sender: &mut futures::stream::SplitSink<WebSocket, Message>,
    error: &RealtimeWsLocalError,
) -> AppResult<()> {
    client_sender
        .send(Message::Text(error.frame().to_string()))
        .await
        .map_err(|send_error| {
            AppError::Upstream(format!(
                "client Realtime WebSocket error send failed: {send_error}"
            ))
        })
}

async fn close_client_sender(
    client_sender: &mut futures::stream::SplitSink<WebSocket, Message>,
    code: u16,
    reason: &'static str,
) -> AppResult<()> {
    client_sender
        .send(Message::Close(Some(CloseFrame {
            code,
            reason: Cow::Borrowed(reason),
        })))
        .await
        .map_err(|error| {
            AppError::Upstream(format!("client Realtime WebSocket close failed: {error}"))
        })
}

fn validate_realtime_model_query(
    query: Option<&str>,
) -> Result<&'static str, RealtimeWsLocalError> {
    let Some(query) = query.filter(|query| !query.is_empty()) else {
        return Err(invalid_realtime_model_query(
            "model query parameter is required.",
        ));
    };
    let mut model = None;
    for pair in query.split('&') {
        let (name, value) = pair.split_once('=').unwrap_or((pair, ""));
        if name != "model" {
            return Err(invalid_realtime_model_query(
                "Only the model query parameter is supported.",
            ));
        }
        if model.replace(value).is_some() {
            return Err(invalid_realtime_model_query(
                "model query parameter must appear exactly once.",
            ));
        }
    }
    match model {
        Some(OPENAI_PUBLIC_REALTIME_MODEL) => Ok(OPENAI_PUBLIC_REALTIME_MODEL),
        Some("") | None => Err(invalid_realtime_model_query(
            "model query parameter must be non-empty.",
        )),
        Some(other) => Err(RealtimeWsLocalError::new(
            StatusCode::BAD_REQUEST,
            "invalid_request_error",
            "model_not_supported",
            format!("model is not supported for Realtime WebSocket: {other}"),
            close_code::POLICY,
            "unsupported realtime model",
        )),
    }
}

fn invalid_realtime_model_query(message: impl Into<String>) -> RealtimeWsLocalError {
    RealtimeWsLocalError::new(
        StatusCode::BAD_REQUEST,
        "invalid_request_error",
        "invalid_realtime_model_query",
        message,
        close_code::POLICY,
        "invalid realtime model query",
    )
}

fn realtime_auth_error(error: AppError) -> RealtimeWsLocalError {
    match error {
        AppError::MissingCredential(message)
            if message.contains("codex")
                || message.contains("Codex")
                || message.contains(".codex") =>
        {
            RealtimeWsLocalError::new(
                StatusCode::UNAUTHORIZED,
                "authentication_error",
                "invalid_api_key",
                MISSING_CODEX_AUTH_MESSAGE,
                close_code::ERROR,
                "missing Codex auth",
            )
        }
        other => RealtimeWsLocalError::new(
            other.status(),
            other.error_type(),
            other.code().unwrap_or(other.error_type()),
            other.to_string(),
            close_code_for_error(&other),
            "realtime auth failed",
        ),
    }
}

fn realtime_queue_saturated_error() -> RealtimeWsLocalError {
    RealtimeWsLocalError::new(
        StatusCode::SERVICE_UNAVAILABLE,
        "upstream_error",
        "realtime_proxy_queue_saturated",
        "Realtime WebSocket proxy queue was saturated.",
        close_code::AGAIN,
        "realtime proxy queue saturated",
    )
}

struct RealtimeWsLocalError {
    status: StatusCode,
    error_type: &'static str,
    code: &'static str,
    message: String,
    close_code: u16,
    close_reason: &'static str,
}

impl RealtimeWsLocalError {
    fn new(
        status: StatusCode,
        error_type: &'static str,
        code: &'static str,
        message: impl Into<String>,
        close_code: u16,
        close_reason: &'static str,
    ) -> Self {
        Self {
            status,
            error_type,
            code,
            message: message.into(),
            close_code,
            close_reason,
        }
    }

    fn frame(&self) -> Value {
        json!({
            "type": "error",
            "status": self.status.as_u16(),
            "error": {
                "type": self.error_type,
                "code": self.code,
                "message": self.message,
                "param": null,
            }
        })
    }
}

impl std::fmt::Display for RealtimeWsLocalError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}: {}", self.code, self.message)
    }
}

impl std::fmt::Debug for RealtimeWsLocalError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("RealtimeWsLocalError")
            .field("status", &self.status)
            .field("error_type", &self.error_type)
            .field("code", &self.code)
            .finish_non_exhaustive()
    }
}

fn close_code_from_upstream(frame: Option<&specter::CloseFrame>) -> u16 {
    frame
        .map(|frame| frame.code.as_u16())
        .filter(|code| (1000..=4999).contains(code))
        .unwrap_or(close_code::NORMAL)
}

fn close_reason_from_upstream(frame: Option<&specter::CloseFrame>) -> &'static str {
    let _ = frame;
    "upstream realtime closed"
}

enum ClientDataFrame {
    Text(String),
    Binary(Vec<u8>),
}

async fn next_client_data_frame(client: &mut WebSocket) -> AppResult<Option<ClientDataFrame>> {
    while let Some(message) = client.next().await {
        match message
            .map_err(|error| AppError::Upstream(format!("client WebSocket read failed: {error}")))?
        {
            Message::Text(text) => return Ok(Some(ClientDataFrame::Text(text))),
            Message::Binary(bytes) => return Ok(Some(ClientDataFrame::Binary(bytes))),
            Message::Ping(bytes) => {
                client.send(Message::Pong(bytes)).await.map_err(|error| {
                    AppError::Upstream(format!("client WebSocket pong failed: {error}"))
                })?;
            }
            Message::Pong(_) => {}
            Message::Close(_) => return Ok(None),
        }
    }
    Ok(None)
}

fn client_frame_json(frame: ClientDataFrame) -> AppResult<Value> {
    Ok(match frame {
        ClientDataFrame::Text(text) => serde_json::from_str(&text)?,
        ClientDataFrame::Binary(bytes) => serde_json::from_slice(&bytes)?,
    })
}

fn ensure_responses_websocket_capable(state: &AppState, value: &Value) -> AppResult<()> {
    let request = responses_request_body(value)?;
    let plan = plan_with_state(state, RequestFormat::Responses, request)?;
    match plan.action {
        DispatchAction::CodexResponses => Ok(()),
        DispatchAction::BedrockAnthropicMessages
        | DispatchAction::GoogleGenerateContent
        | DispatchAction::CursorAgent
        | DispatchAction::WindsurfChat => Err(AppError::ModelNotSupported(plan.requested_model)),
    }
}

fn responses_request_body(value: &Value) -> AppResult<&Value> {
    if value.get("type").and_then(Value::as_str) == Some("response.create") {
        return Ok(value.get("response").unwrap_or(value));
    }
    Ok(value)
}

fn close_code_for_error(error: &AppError) -> u16 {
    match error {
        AppError::BadRequest(_) | AppError::BadRequestCode { .. } => close_code::INVALID,
        AppError::ModelNotSupported(_) | AppError::NotFound(_) => close_code::POLICY,
        AppError::MissingCredential(_)
        | AppError::Upstream(_)
        | AppError::Io(_)
        | AppError::Json(_)
        | AppError::Compaction(_) => close_code::ERROR,
        AppError::TooManyRequests { .. } => close_code::AGAIN,
    }
}

const BRIDGE_RESPONSE_STATE_LIMIT: usize = 16;

#[derive(Clone, Debug, Eq, PartialEq)]
struct BridgeRouteFingerprint {
    route: ResponsesRoute,
    model: String,
    upstream_model: String,
    target_format: String,
    remote_compaction_policy: RemoteCompactionPolicy,
}

struct BridgeSessionState {
    responses: VecDeque<BridgeResponseState>,
    compaction_session_binding: String,
}

impl Default for BridgeSessionState {
    fn default() -> Self {
        Self {
            responses: VecDeque::new(),
            compaction_session_binding: format!("ws:{}", uuid::Uuid::new_v4().simple()),
        }
    }
}

impl BridgeSessionState {
    fn get(&self, response_id: &str) -> Option<&BridgeResponseState> {
        self.responses
            .iter()
            .find(|response| response.response_id == response_id)
    }

    fn record(&mut self, response: BridgeResponseState) {
        self.responses
            .retain(|existing| existing.response_id != response.response_id);
        self.responses.push_back(response);
        while self.responses.len() > BRIDGE_RESPONSE_STATE_LIMIT {
            self.responses.pop_front();
        }
    }
}

#[derive(Clone, Debug)]
struct BridgeResponseState {
    fingerprint: BridgeRouteFingerprint,
    response_id: String,
    full_request: Value,
    output_item_done_items: Vec<Value>,
}

#[derive(Debug)]
struct BridgeExecutionResult {
    response_id: String,
    full_request: Value,
    output_item_done_items: Vec<Value>,
}

enum BridgeResponseOutcome {
    Continue,
    Closed,
}

#[derive(Debug)]
struct BridgePolicyError {
    code: &'static str,
    message: String,
}

impl BridgePolicyError {
    fn new(code: &'static str, message: impl ToString) -> Self {
        Self {
            code,
            message: message.to_string(),
        }
    }

    fn frame(&self) -> Value {
        websocket_request_error(StatusCode::BAD_REQUEST, self.code, &self.message)
    }
}

async fn dispatch_responses_client_event(
    state: &AppState,
    client: &mut WebSocket,
    session: &mut BridgeSessionState,
    value: Value,
) -> AppResult<bool> {
    match value.get("type").and_then(Value::as_str) {
        Some("response.processed") => return Ok(true),
        Some("response.create") => {}
        Some(other) => {
            send_unsupported_event(client, Some(other)).await?;
            return Ok(true);
        }
        None if value.get("model").is_some() => {}
        None => {
            send_unsupported_event(client, None).await?;
            return Ok(true);
        }
    }

    let request = match responses_request_body(&value) {
        Ok(request) => request,
        Err(error) => {
            send_ws_app_error(client, &error).await?;
            return Ok(true);
        }
    };
    let fingerprint = match resolve_bridge_route(state, request).await {
        Ok(fingerprint) => fingerprint,
        Err(error) => {
            send_ws_app_error(client, &error).await?;
            return Ok(true);
        }
    };
    match execute_bridge_response_create(state, client, value, fingerprint, session).await? {
        BridgeResponseOutcome::Continue => Ok(true),
        BridgeResponseOutcome::Closed => Ok(false),
    }
}

async fn resolve_bridge_route(
    state: &AppState,
    request: &Value,
) -> AppResult<BridgeRouteFingerprint> {
    let plan = plan_with_state(state, RequestFormat::Responses, request)?;
    let route = match plan.action {
        DispatchAction::CodexResponses => ResponsesRoute::CodexResponses,
        DispatchAction::BedrockAnthropicMessages => ResponsesRoute::BedrockMessages,
        DispatchAction::GoogleGenerateContent => ResponsesRoute::GoogleGenerateContent {
            upstream_model: plan.target.upstream_model.clone(),
        },
        DispatchAction::CursorAgent => {
            return Err(AppError::ModelNotSupported(plan.requested_model))
        }
        DispatchAction::WindsurfChat => {
            return Err(AppError::ModelNotSupported(plan.requested_model))
        }
    };
    let target_format = plan.target.target_format.as_str().to_string();
    Ok(BridgeRouteFingerprint {
        route,
        model: plan.requested_model,
        upstream_model: plan.target.upstream_model,
        target_format,
        remote_compaction_policy: plan.remote_compaction_policy,
    })
}

async fn execute_bridge_response_create(
    state: &AppState,
    client: &mut WebSocket,
    value: Value,
    fingerprint: BridgeRouteFingerprint,
    session: &mut BridgeSessionState,
) -> AppResult<BridgeResponseOutcome> {
    let mut request = responses_request_body_owned(value)?;
    let mut request = match prepare_bridge_request(request.take(), &fingerprint, session) {
        Ok(request) => request,
        Err(error) => {
            send_ws_json(client, error.frame()).await?;
            return Ok(BridgeResponseOutcome::Continue);
        }
    };

    match is_v2_context_compaction_trigger(&request) {
        Err(error) => {
            send_ws_app_error(client, &error).await?;
            return Ok(BridgeResponseOutcome::Continue);
        }
        Ok(false) => {}
        Ok(true) if fingerprint.route == ResponsesRoute::CodexResponses => {
            // Native Codex Responses can handle the v2 trigger through the existing path.
        }
        Ok(true) => {
            match fingerprint.remote_compaction_policy {
                RemoteCompactionPolicy::ProxyVisibleSummary => {
                    let context = bridge_compaction_context(session, &fingerprint)?;
                    let result =
                        send_synthetic_context_compaction_lifecycle(client, &request, &context)
                            .await?;
                    session.record(BridgeResponseState {
                        fingerprint,
                        response_id: result.response_id,
                        full_request: result.full_request,
                        output_item_done_items: result.output_item_done_items,
                    });
                }
                RemoteCompactionPolicy::Off => {
                    send_ws_json(
                        client,
                        websocket_request_error(
                            StatusCode::CONFLICT,
                            "compaction_disabled_for_target",
                            "remote compaction is disabled for target",
                        ),
                    )
                    .await?;
                }
                RemoteCompactionPolicy::Native => {
                    send_ws_json(
                        client,
                        websocket_request_error(
                            StatusCode::BAD_REQUEST,
                            "unsupported_compaction_item_for_target",
                            "native compaction is not supported for the resolved target",
                        ),
                    )
                    .await?;
                }
                RemoteCompactionPolicy::Local => {
                    send_ws_json(
                        client,
                        context_compaction_unavailable_frame(StatusCode::SERVICE_UNAVAILABLE),
                    )
                    .await?;
                }
            }
            return Ok(BridgeResponseOutcome::Continue);
        }
    }

    if fingerprint.route == ResponsesRoute::CodexResponses {
        if let Err(error) =
            validate_codex_catalog_websocket_request(state, &request, &fingerprint.upstream_model)
                .await
        {
            send_ws_app_error(client, &error).await?;
            return Ok(BridgeResponseOutcome::Continue);
        }
    }
    if let Err(error) = prepare_bridge_compaction_carriers(&mut request, session, &fingerprint) {
        send_ws_app_error(client, &error).await?;
        return Ok(BridgeResponseOutcome::Continue);
    }

    if request.get("generate").and_then(Value::as_bool) == Some(false) {
        let response_id = format!("resp_ws_bridge_{}", uuid::Uuid::new_v4().simple());
        send_synthetic_lifecycle(client, &response_id).await?;
        session.record(BridgeResponseState {
            fingerprint,
            response_id,
            full_request: request,
            output_item_done_items: Vec::new(),
        });
        return Ok(BridgeResponseOutcome::Continue);
    }

    let mut provider_request = request.clone();
    provider_request["stream"] = Value::Bool(true);
    provider_request
        .as_object_mut()
        .expect("Responses request is an object")
        .remove("generate");

    let result = match fingerprint.route {
        ResponsesRoute::CodexResponses => {
            execute_codex_ws_response(state, client, provider_request, request).await?
        }
        ResponsesRoute::BedrockMessages | ResponsesRoute::GoogleGenerateContent { .. } => {
            run_bridge_provider_task(state.clone(), client, provider_request, request).await?
        }
        ResponsesRoute::CursorAgent { .. } => {
            send_ws_json(
                client,
                websocket_request_error(
                    StatusCode::BAD_REQUEST,
                    "unsupported_route",
                    "cursor_agent route is not supported over the Responses WebSocket bridge",
                ),
            )
            .await?;
            return Ok(BridgeResponseOutcome::Continue);
        }
        ResponsesRoute::WindsurfChat { .. } => {
            send_ws_json(
                client,
                websocket_request_error(
                    StatusCode::BAD_REQUEST,
                    "unsupported_route",
                    "windsurf_chat route is not supported over the Responses WebSocket bridge",
                ),
            )
            .await?;
            return Ok(BridgeResponseOutcome::Continue);
        }
    };
    let Some(result) = result else {
        return Ok(BridgeResponseOutcome::Closed);
    };
    session.record(BridgeResponseState {
        fingerprint,
        response_id: result.response_id,
        full_request: result.full_request,
        output_item_done_items: result.output_item_done_items,
    });
    Ok(BridgeResponseOutcome::Continue)
}

fn responses_request_body_owned(value: Value) -> AppResult<Value> {
    if value.get("type").and_then(Value::as_str) == Some("response.create") {
        return Ok(value.get("response").cloned().unwrap_or(value));
    }
    Ok(value)
}

fn prepare_bridge_request(
    mut request: Value,
    fingerprint: &BridgeRouteFingerprint,
    session: &BridgeSessionState,
) -> Result<Value, BridgePolicyError> {
    if matches!(request.get("previous_response_id"), Some(Value::Null)) {
        let Some(object) = request.as_object_mut() else {
            return Err(BridgePolicyError::new(
                "previous_response_field_mismatch",
                "Responses request must be a JSON object",
            ));
        };
        object.remove("previous_response_id");
        return Ok(request);
    }
    let previous_response_id = match request.get("previous_response_id") {
        None => return Ok(request),
        Some(Value::String(value)) => value.clone(),
        Some(_) => {
            return Err(BridgePolicyError::new(
                "previous_response_field_mismatch",
                "previous_response_id must be a string",
            ));
        }
    };
    let Some(prior) = session.get(&previous_response_id) else {
        return Err(BridgePolicyError::new(
            "unknown_previous_response_id",
            "unknown previous_response_id",
        ));
    };

    if prior.fingerprint.target_format != fingerprint.target_format {
        return Err(BridgePolicyError::new(
            "previous_response_target_format_mismatch",
            format!(
                "previous_response_id belongs to target format {}, not {}",
                prior.fingerprint.target_format, fingerprint.target_format
            ),
        ));
    }
    if bridge_route_lane(&prior.fingerprint.route) != bridge_route_lane(&fingerprint.route) {
        return Err(BridgePolicyError::new(
            "previous_response_route_mismatch",
            format!(
                "previous_response_id belongs to {}, not {}",
                prior.fingerprint.model, fingerprint.model
            ),
        ));
    }
    if prior.fingerprint != *fingerprint {
        return Err(BridgePolicyError::new(
            "previous_response_model_mismatch",
            format!(
                "previous_response_id belongs to {}, not {}",
                prior.fingerprint.model, fingerprint.model
            ),
        ));
    }

    let object = request.as_object_mut().ok_or_else(|| {
        BridgePolicyError::new(
            "previous_response_field_mismatch",
            "Responses request must be a JSON object",
        )
    })?;
    object.remove("previous_response_id");
    ensure_incremental_request_matches(&prior.full_request, &request).map_err(|error| {
        BridgePolicyError::new("previous_response_field_mismatch", error.to_string())
    })?;

    let delta_input = request
        .get("input")
        .cloned()
        .unwrap_or(Value::Array(Vec::new()));
    let mut full_request = prior.full_request.clone();
    if let Some(object) = full_request.as_object_mut() {
        object.remove("generate");
    }
    if prior.output_item_done_items.is_empty() && is_empty_input(&delta_input) {
        return Ok(full_request);
    }
    let mut merged_input = Vec::new();
    append_input_values(full_request.get("input"), &mut merged_input);
    merged_input.extend(prior.output_item_done_items.iter().cloned());
    append_input_values(Some(&delta_input), &mut merged_input);
    full_request["input"] = Value::Array(merged_input);
    Ok(full_request)
}

fn bridge_route_lane(route: &ResponsesRoute) -> &'static str {
    match route {
        ResponsesRoute::CodexResponses => "codex",
        ResponsesRoute::BedrockMessages => "bedrock",
        ResponsesRoute::GoogleGenerateContent { .. } => "google",
        ResponsesRoute::CursorAgent { .. } => "cursor",
        ResponsesRoute::WindsurfChat { .. } => "windsurf",
    }
}

fn is_empty_input(value: &Value) -> bool {
    value.as_array().is_some_and(Vec::is_empty)
        || value.as_str().is_some_and(str::is_empty)
        || value.is_null()
}

fn ensure_incremental_request_matches(baseline: &Value, incremental: &Value) -> AppResult<()> {
    let baseline = baseline
        .as_object()
        .ok_or_else(|| AppError::BadRequest("stored Responses request must be an object".into()))?;
    let incremental = incremental
        .as_object()
        .ok_or_else(|| AppError::BadRequest("Responses request must be an object".into()))?;
    for (key, value) in incremental {
        if is_incremental_ignored_field(key) || key == "input" {
            continue;
        }
        if baseline.get(key) != Some(value) {
            return Err(AppError::BadRequest(format!(
                "previous_response_id request field changed: {key}"
            )));
        }
    }
    Ok(())
}

fn is_incremental_ignored_field(key: &str) -> bool {
    matches!(key, "type" | "client_metadata" | "generate")
}

fn append_input_values(value: Option<&Value>, output: &mut Vec<Value>) {
    match value {
        Some(Value::Array(values)) => output.extend(values.iter().cloned()),
        Some(Value::String(text)) if !text.is_empty() => output.push(json!({
            "type": "message",
            "role": "user",
            "content": [{ "type": "input_text", "text": text }]
        })),
        Some(Value::Null) | None => {}
        Some(other) => output.push(other.clone()),
    }
}

fn is_sse_response(headers: &HeaderMap) -> bool {
    headers
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.starts_with("text/event-stream"))
}

async fn forward_responses_response_to_ws(
    sender: mpsc::Sender<Value>,
    response: UpstreamResponse,
    full_request: Value,
) -> AppResult<BridgeExecutionResult> {
    if !response.status.is_success() {
        let status = response.status;
        let body = to_bytes(response.body, usize::MAX)
            .await
            .map_err(|error| AppError::Upstream(error.to_string()))?;
        let message = String::from_utf8_lossy(&body);
        send_bridge_frame(
            &sender,
            websocket_request_error(status, "upstream_error", message.as_ref()),
        )
        .await?;
        return Err(AppError::Upstream(format!(
            "HTTP-backed Responses bridge upstream returned {status}"
        )));
    }

    if !is_sse_response(&response.headers) {
        let body = to_bytes(response.body, usize::MAX)
            .await
            .map_err(|error| AppError::Upstream(error.to_string()))?;
        let response_json: Value = serde_json::from_slice(&body)?;
        let response_id = response_json
            .get("id")
            .and_then(Value::as_str)
            .unwrap_or("resp_ws_bridge_json")
            .to_string();
        send_bridge_frame(
            &sender,
            json!({ "type": "response.created", "response": response_json }),
        )
        .await?;
        let output_item_done_items = output_items_from_response(&response_json);
        for (index, item) in output_item_done_items.iter().enumerate() {
            send_bridge_frame(
                &sender,
                json!({
                    "type": "response.output_item.added",
                    "output_index": index,
                    "item": output_item_added_shape(item),
                }),
            )
            .await?;
            send_bridge_frame(
                &sender,
                json!({
                    "type": "response.output_item.done",
                    "output_index": index,
                    "item": item,
                }),
            )
            .await?;
        }
        send_bridge_frame(
            &sender,
            json!({ "type": "response.completed", "response": response_json }),
        )
        .await?;
        return Ok(BridgeExecutionResult {
            response_id,
            full_request,
            output_item_done_items,
        });
    }

    let mut parser = ResponsesSseParser::new();
    let mut response_id = None;
    let mut output_item_done_items = Vec::new();
    let mut terminal = false;
    let mut bridge_terminal = false;
    let mut stream = response.body.into_data_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = match chunk {
            Ok(chunk) => chunk,
            Err(error) => {
                send_bridge_failure(&sender, response_id.as_deref(), error.to_string()).await?;
                return Err(AppError::Upstream(error.to_string()));
            }
        };
        let frames = match parser.push_bytes(chunk) {
            Ok(frames) => frames,
            Err(error) => {
                send_bridge_failure(&sender, response_id.as_deref(), error.to_string()).await?;
                return Err(error);
            }
        };
        for frame in frames {
            let event = normalize_bridge_stream_event(frame.data, response_id.as_deref());
            observe_bridge_event(
                &event,
                &mut response_id,
                &mut output_item_done_items,
                &mut terminal,
            );
            if is_bridge_terminal_event(&event) {
                bridge_terminal = true;
            }
            send_bridge_frame(&sender, event).await?;
            if terminal {
                return Ok(BridgeExecutionResult {
                    response_id: response_id.take().unwrap_or_else(|| {
                        format!("resp_ws_bridge_{}", uuid::Uuid::new_v4().simple())
                    }),
                    full_request,
                    output_item_done_items,
                });
            }
        }
    }
    let frames = match parser.finish() {
        Ok(frames) => frames,
        Err(_error) if bridge_terminal => Vec::new(),
        Err(error) => {
            send_bridge_failure(&sender, response_id.as_deref(), error.to_string()).await?;
            return Err(error);
        }
    };
    for frame in frames {
        let event = normalize_bridge_stream_event(frame.data, response_id.as_deref());
        observe_bridge_event(
            &event,
            &mut response_id,
            &mut output_item_done_items,
            &mut terminal,
        );
        send_bridge_frame(&sender, event).await?;
        if terminal {
            return Ok(BridgeExecutionResult {
                response_id: response_id
                    .take()
                    .unwrap_or_else(|| format!("resp_ws_bridge_{}", uuid::Uuid::new_v4().simple())),
                full_request,
                output_item_done_items,
            });
        }
    }
    if !terminal {
        let message = "Responses SSE stream ended before response.completed";
        send_bridge_failure(&sender, response_id.as_deref(), message).await?;
        return Err(AppError::Upstream(message.into()));
    }
    Ok(BridgeExecutionResult {
        response_id: response_id
            .unwrap_or_else(|| format!("resp_ws_bridge_{}", uuid::Uuid::new_v4().simple())),
        full_request,
        output_item_done_items,
    })
}

fn normalize_bridge_stream_event(event: Value, response_id: Option<&str>) -> Value {
    if event.get("type").and_then(Value::as_str) == Some("error") {
        if let Some(response_id) = response_id {
            let message = event
                .pointer("/error/message")
                .and_then(Value::as_str)
                .or_else(|| event.get("message").and_then(Value::as_str))
                .unwrap_or("upstream Responses stream failed");
            let code = event
                .pointer("/error/code")
                .and_then(Value::as_str)
                .or_else(|| event.pointer("/error/type").and_then(Value::as_str))
                .unwrap_or("upstream_error");
            return json!({
                "type": "response.failed",
                "response": {
                    "id": response_id,
                    "status": "failed",
                    "error": {
                        "code": code,
                        "message": message
                    }
                }
            });
        }
    }
    event
}

async fn run_bridge_provider_task(
    state: AppState,
    client: &mut WebSocket,
    provider_request: Value,
    full_request: Value,
) -> AppResult<Option<BridgeExecutionResult>> {
    let (sender, mut receiver) = mpsc::channel(REALTIME_WS_QUEUE_CAPACITY);
    let mut terminal_forwarded = false;
    let mut task = tokio::spawn(async move {
        let response = match execute_responses_request(
            &state,
            HeaderMap::new(),
            provider_request,
            ExecuteResponsesOptions { force_stream: true },
        )
        .await
        {
            Ok(response) => response,
            Err(error) => {
                let _ = send_bridge_frame(
                    &sender,
                    websocket_request_error(
                        error.status(),
                        error.code().unwrap_or(error.error_type()),
                        error.to_string(),
                    ),
                )
                .await;
                return Err(error);
            }
        };
        forward_responses_response_to_ws(sender, response, full_request).await
    });

    loop {
        tokio::select! {
            frame = receiver.recv() => {
                if let Some(frame) = frame {
                    let is_terminal = is_bridge_terminal_event(&frame);
                    if let Err(error) = send_ws_json(client, frame).await {
                        task.abort();
                        return Err(error);
                    }
                    terminal_forwarded = terminal_forwarded || is_terminal;
                }
            }
            result = &mut task => {
                while let Ok(frame) = receiver.try_recv() {
                    send_ws_json(client, frame).await?;
                }
                return match result {
                    Ok(Ok(result)) => Ok(Some(result)),
                    Ok(Err(_error)) => Ok(None),
                    Err(error) => Err(AppError::Upstream(format!(
                        "Responses WebSocket bridge task failed: {error}"
                    ))),
                };
            }
            message = client.next(), if !terminal_forwarded => {
                let Some(message) = message else {
                    task.abort();
                    return Ok(None);
                };
                let message = match message {
                    Ok(message) => message,
                    Err(error) => {
                        task.abort();
                        return Err(AppError::Upstream(format!(
                            "client WebSocket read failed: {error}"
                        )));
                    }
                };
                match message {
                    Message::Text(text) => {
                        let value: Value = match serde_json::from_str(&text) {
                            Ok(value) => value,
                            Err(error) => {
                                task.abort();
                                return Err(AppError::Json(error));
                            }
                        };
                        match value.get("type").and_then(Value::as_str) {
                            Some("response.processed") => {}
                                Some("response.create") | None if is_response_create_or_raw_body(&value) => {
                                if let Err(error) = send_ws_json(
                                    client,
                                    websocket_request_error(
                                        StatusCode::BAD_REQUEST,
                                        "response_already_in_flight",
                                            "response.create is already in flight",
                                    ),
                                )
                                .await
                                {
                                    task.abort();
                                    return Err(error);
                                }
                            }
                            Some(other) => {
                                if let Err(error) = send_ws_json(
                                    client,
                                    websocket_request_error(
                                        StatusCode::BAD_REQUEST,
                                        "unsupported_websocket_event",
                                            format!("unsupported Responses WebSocket event: {other}"),
                                    ),
                                )
                                .await
                                {
                                    task.abort();
                                    return Err(error);
                                }
                            }
                            None => {}
                        }
                    }
                    Message::Binary(bytes) => {
                        let value: Value = match serde_json::from_slice(&bytes) {
                            Ok(value) => value,
                            Err(error) => {
                                task.abort();
                                return Err(AppError::Json(error));
                            }
                        };
                        if is_response_create_or_raw_body(&value) {
                            if let Err(error) = send_ws_json(
                                client,
                                websocket_request_error(
                                    StatusCode::BAD_REQUEST,
                                    "response_already_in_flight",
                                        "response.create is already in flight",
                                ),
                            )
                            .await
                            {
                                task.abort();
                                return Err(error);
                            }
                        }
                    }
                    Message::Ping(bytes) => {
                        if let Err(error) = client.send(Message::Pong(bytes)).await {
                            task.abort();
                            return Err(AppError::Upstream(format!(
                                "client WebSocket pong failed: {error}"
                            )));
                        }
                    }
                    Message::Pong(_) => {}
                    Message::Close(_) => {
                        task.abort();
                        return Ok(None);
                    }
                }
            }
        }
    }
}

async fn execute_codex_ws_response(
    state: &AppState,
    client: &mut WebSocket,
    provider_request: Value,
    full_request: Value,
) -> AppResult<Option<BridgeExecutionResult>> {
    let payload = prepare_responses_frame_payload(state, provider_request)?;
    let mut upstream =
        upstream::codex::connect_responses_wss(state, &state.runtime.codex_responses_wss_url)
            .await?;
    upstream
        .send_text(payload)
        .await
        .map_err(|error| AppError::Upstream(format!("Codex WSS send failed: {error}")))?;

    let mut response_id = None;
    let mut output_item_done_items = Vec::new();
    loop {
        tokio::select! {
            client_message = client.next() => {
                let Some(message) = client_message else {
                    let _ = upstream.close(None).await;
                    return Ok(None);
                };
                let message = message.map_err(|error| {
                    AppError::Upstream(format!("client WebSocket read failed: {error}"))
                })?;
                if handle_codex_inflight_client_message(client, message, &mut upstream).await? {
                    return Ok(None);
                }
            }
            upstream_message = upstream.next() => {
                match upstream_message {
                    Ok(Some(message)) => {
                        if let Some(result) = handle_codex_upstream_message(
                            message,
                            client,
                            &mut response_id,
                            &mut output_item_done_items,
                            full_request.clone(),
                        )
                        .await?
                        {
                            let _ = upstream.close(None).await;
                            return Ok(Some(result));
                        }
                    }
                    Ok(None) => {
                        send_codex_upstream_failure(
                            client,
                            response_id.as_deref(),
                            "Codex WSS upstream closed before a terminal response event",
                        )
                        .await?;
                        let _ = client.send(Message::Close(None)).await;
                        return Ok(None);
                    }
                    Err(error) => {
                        tracing::warn!(%error, "Codex WSS read failed; reporting downstream error");
                        send_codex_upstream_failure(
                            client,
                            response_id.as_deref(),
                            format!("Codex WSS read failed: {error}"),
                        )
                        .await?;
                        let _ = client
                            .send(Message::Close(Some(CloseFrame {
                                code: close_code::ERROR,
                                reason: Cow::Owned("Codex WSS read failed".to_string()),
                            })))
                            .await;
                        return Ok(None);
                    }
                }
            }
        }
    }
}

async fn send_codex_upstream_failure(
    client: &mut WebSocket,
    response_id: Option<&str>,
    message: impl ToString,
) -> AppResult<()> {
    let message = message.to_string();
    let frame = if let Some(response_id) = response_id {
        json!({
            "type": "response.failed",
            "response": {
                "id": response_id,
                "status": "failed",
                "error": {
                    "code": "upstream_error",
                    "message": message
                }
            }
        })
    } else {
        websocket_request_error(StatusCode::BAD_GATEWAY, "upstream_error", message)
    };
    send_ws_json(client, frame).await
}

async fn handle_codex_inflight_client_message(
    client: &mut WebSocket,
    message: Message,
    upstream: &mut specter::WebSocket,
) -> AppResult<bool> {
    match message {
        Message::Text(text) => {
            let value: Value = serde_json::from_str(&text)?;
            match value.get("type").and_then(Value::as_str) {
                Some("response.processed") => {
                    upstream.send_text(text).await.map_err(|error| {
                        AppError::Upstream(format!("Codex WSS send failed: {error}"))
                    })?;
                }
                Some("response.create") | None if is_response_create_or_raw_body(&value) => {
                    send_response_already_in_flight(client).await?;
                }
                Some(other) => {
                    send_unsupported_event(client, Some(other)).await?;
                }
                None => {}
            }
        }
        Message::Binary(bytes) => {
            let value: Value = serde_json::from_slice(&bytes)?;
            if is_response_create_or_raw_body(&value) {
                send_response_already_in_flight(client).await?;
            } else {
                upstream.send_binary(bytes).await.map_err(|error| {
                    AppError::Upstream(format!("Codex WSS send failed: {error}"))
                })?;
            }
        }
        Message::Ping(bytes) => upstream
            .send(UpstreamMessage::Ping(bytes.into()))
            .await
            .map_err(|error| AppError::Upstream(format!("Codex WSS ping failed: {error}")))?,
        Message::Pong(bytes) => upstream
            .send(UpstreamMessage::Pong(bytes.into()))
            .await
            .map_err(|error| AppError::Upstream(format!("Codex WSS pong failed: {error}")))?,
        Message::Close(frame) => {
            let _ = upstream.close(to_upstream_close_frame(frame)).await;
            return Ok(true);
        }
    }
    Ok(false)
}

async fn handle_codex_upstream_message(
    message: UpstreamMessage,
    client: &mut WebSocket,
    response_id: &mut Option<String>,
    output_item_done_items: &mut Vec<Value>,
    full_request: Value,
) -> AppResult<Option<BridgeExecutionResult>> {
    match message {
        UpstreamMessage::Text(text) => {
            let event: Value = serde_json::from_str(&text)?;
            let mut terminal = false;
            observe_bridge_event(&event, response_id, output_item_done_items, &mut terminal);
            send_ws_json(client, event).await?;
            if terminal {
                return Ok(Some(BridgeExecutionResult {
                    response_id: response_id.clone().unwrap_or_else(|| {
                        format!("resp_ws_bridge_{}", uuid::Uuid::new_v4().simple())
                    }),
                    full_request,
                    output_item_done_items: output_item_done_items.clone(),
                }));
            }
        }
        UpstreamMessage::Binary(bytes) => {
            let event: Value = serde_json::from_slice(&bytes)?;
            let mut terminal = false;
            observe_bridge_event(&event, response_id, output_item_done_items, &mut terminal);
            send_ws_json(client, event).await?;
            if terminal {
                return Ok(Some(BridgeExecutionResult {
                    response_id: response_id.clone().unwrap_or_else(|| {
                        format!("resp_ws_bridge_{}", uuid::Uuid::new_v4().simple())
                    }),
                    full_request,
                    output_item_done_items: output_item_done_items.clone(),
                }));
            }
        }
        UpstreamMessage::Ping(bytes) => {
            client
                .send(Message::Ping(bytes.to_vec()))
                .await
                .map_err(|error| {
                    AppError::Upstream(format!("client WebSocket send failed: {error}"))
                })?;
        }
        UpstreamMessage::Pong(bytes) => {
            client
                .send(Message::Pong(bytes.to_vec()))
                .await
                .map_err(|error| {
                    AppError::Upstream(format!("client WebSocket send failed: {error}"))
                })?;
        }
        UpstreamMessage::Close(frame) => {
            send_codex_upstream_failure(
                client,
                response_id.as_deref(),
                "Codex WSS upstream closed before a terminal response event",
            )
            .await?;
            client
                .send(Message::Close(to_client_close_frame(frame)))
                .await
                .map_err(|error| {
                    AppError::Upstream(format!("client WebSocket send failed: {error}"))
                })?;
        }
    }
    Ok(None)
}

async fn send_bridge_frame(sender: &mpsc::Sender<Value>, value: Value) -> AppResult<()> {
    sender
        .send(value)
        .await
        .map_err(|_| AppError::Upstream("client WebSocket bridge receiver closed".into()))
}

async fn send_bridge_failure(
    sender: &mpsc::Sender<Value>,
    response_id: Option<&str>,
    message: impl ToString,
) -> AppResult<()> {
    let message = message.to_string();
    let frame = if let Some(response_id) = response_id {
        json!({
            "type": "response.failed",
            "response": {
                "id": response_id,
                "status": "failed",
                "error": {
                    "code": "upstream_error",
                    "message": message
                }
            }
        })
    } else {
        websocket_request_error(StatusCode::BAD_GATEWAY, "upstream_error", message)
    };
    send_bridge_frame(sender, frame).await
}

fn observe_bridge_event(
    event: &Value,
    response_id: &mut Option<String>,
    output_item_done_items: &mut Vec<Value>,
    terminal: &mut bool,
) {
    match event.get("type").and_then(Value::as_str) {
        Some("response.created") => {
            if let Some(id) = event.pointer("/response/id").and_then(Value::as_str) {
                *response_id = Some(id.to_string());
            }
        }
        Some("response.output_item.done") => {
            if let Some(item) = event.get("item") {
                output_item_done_items.push(item.clone());
            }
        }
        Some("response.completed") | Some("response.failed") | Some("response.incomplete") => {
            if let Some(id) = event.pointer("/response/id").and_then(Value::as_str) {
                *response_id = Some(id.to_string());
            }
            *terminal = true;
        }
        _ => {}
    }
}

fn is_bridge_terminal_event(event: &Value) -> bool {
    matches!(
        event.get("type").and_then(Value::as_str),
        Some("response.completed" | "response.failed" | "response.incomplete")
    )
}

fn output_items_from_response(response: &Value) -> Vec<Value> {
    response
        .get("output")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
}

fn output_item_added_shape(item: &Value) -> Value {
    let mut added = item.clone();
    let Some(object) = added.as_object_mut() else {
        return added;
    };
    object.insert("status".into(), Value::String("in_progress".into()));
    match object.get("type").and_then(Value::as_str) {
        Some("message") => {
            object.insert("content".into(), Value::Array(Vec::new()));
        }
        Some("function_call") => {
            object.insert("arguments".into(), Value::String(String::new()));
        }
        Some("custom_tool_call") => {
            object.insert("input".into(), Value::String(String::new()));
        }
        _ => {}
    }
    added
}

async fn send_synthetic_lifecycle(client: &mut WebSocket, response_id: &str) -> AppResult<()> {
    send_ws_json(
        client,
        json!({
            "type": "response.created",
            "response": { "id": response_id }
        }),
    )
    .await?;
    send_ws_json(
        client,
        json!({
            "type": "response.completed",
            "response": {
                "id": response_id,
                "usage": {
                    "input_tokens": 0,
                    "input_tokens_details": Value::Null,
                    "output_tokens": 0,
                    "output_tokens_details": Value::Null,
                    "total_tokens": 0
                }
            }
        }),
    )
    .await
}

async fn send_synthetic_context_compaction_lifecycle(
    client: &mut WebSocket,
    full_request: &Value,
    context: &CompactionPackContext,
) -> AppResult<BridgeExecutionResult> {
    let response_id = format!("resp_ws_compact_{}", uuid::Uuid::new_v4().simple());
    let item = proxy_visible_context_compaction_item(full_request, context)?;
    send_ws_json(
        client,
        json!({
            "type": "response.created",
            "response": {
                "id": response_id,
                "status": "in_progress"
            }
        }),
    )
    .await?;
    send_ws_json(
        client,
        json!({
            "type": "response.output_item.added",
            "output_index": 0,
            "item": output_item_added_shape(&item),
        }),
    )
    .await?;
    send_ws_json(
        client,
        json!({
            "type": "response.output_item.done",
            "output_index": 0,
            "item": item,
        }),
    )
    .await?;
    send_ws_json(
        client,
        json!({
            "type": "response.completed",
            "response": {
                "id": response_id,
                "status": "completed",
                "output": [item],
                "usage": {
                    "input_tokens": 0,
                    "input_tokens_details": Value::Null,
                    "output_tokens": 0,
                    "output_tokens_details": Value::Null,
                    "total_tokens": 0
                }
            }
        }),
    )
    .await?;
    Ok(BridgeExecutionResult {
        response_id,
        full_request: full_request.clone(),
        output_item_done_items: vec![item],
    })
}

fn bridge_compaction_context(
    session: &BridgeSessionState,
    fingerprint: &BridgeRouteFingerprint,
) -> AppResult<CompactionPackContext> {
    let instance_id = std::env::var("UMP_COMPACTION_INSTANCE_ID").map_err(|_| {
        CompactionHttpError::new(
            StatusCode::SERVICE_UNAVAILABLE,
            "proxy_compaction_unavailable",
            "server_error",
            "UMP_COMPACTION_INSTANCE_ID is required for proxy-visible compaction",
        )
    })?;
    Ok(CompactionPackContext {
        auth_subject: format!("local-no-auth:{instance_id}"),
        session_binding: session.compaction_session_binding.clone(),
        route_binding: "WS /v1/responses".into(),
        target_provider: bridge_route_provider(&fingerprint.route).to_string(),
        target_format: fingerprint.target_format.clone(),
        target_model: fingerprint.upstream_model.clone(),
    })
}

fn bridge_route_provider(route: &ResponsesRoute) -> &'static str {
    match route {
        ResponsesRoute::CodexResponses => "codex",
        ResponsesRoute::BedrockMessages => "bedrock",
        ResponsesRoute::GoogleGenerateContent { .. } => "google",
        ResponsesRoute::CursorAgent { .. } => "cursor",
        ResponsesRoute::WindsurfChat { .. } => "windsurf",
    }
}

fn prepare_bridge_compaction_carriers(
    request: &mut Value,
    session: &BridgeSessionState,
    fingerprint: &BridgeRouteFingerprint,
) -> AppResult<()> {
    let carriers = request
        .get("input")
        .map(find_compaction_carriers)
        .unwrap_or_default();
    if carriers.is_empty() {
        return Ok(());
    }
    let target = bridge_resolved_target(fingerprint)?;
    let context = if carriers.iter().any(|carrier| carrier.is_ump_pack) {
        Some(bridge_compaction_context(session, fingerprint)?)
    } else {
        None
    };
    prepare_responses_input_for_target(
        request,
        &target,
        CompactionLimits::default(),
        context.as_ref(),
    )
    .map(|_| ())
}

fn bridge_resolved_target(fingerprint: &BridgeRouteFingerprint) -> AppResult<ResolvedTarget> {
    let target_format = match fingerprint.target_format.as_str() {
        "responses" => TargetFormat::Responses,
        "anthropic_messages" => TargetFormat::AnthropicMessages,
        "google_generate_content" => TargetFormat::GoogleGenerateContent,
        "openai_images" => TargetFormat::OpenaiImages,
        other => {
            return Err(AppError::BadRequest(format!(
                "unsupported target format {other}"
            )))
        }
    };
    Ok(ResolvedTarget {
        provider: bridge_route_provider_enum(&fingerprint.route),
        upstream_model: fingerprint.upstream_model.clone(),
        target_format,
    })
}

fn bridge_route_provider_enum(route: &ResponsesRoute) -> Provider {
    match route {
        ResponsesRoute::CodexResponses => Provider::Codex,
        ResponsesRoute::BedrockMessages => Provider::Bedrock,
        ResponsesRoute::GoogleGenerateContent { .. } => Provider::Google,
        ResponsesRoute::CursorAgent { .. } => Provider::Cursor,
        ResponsesRoute::WindsurfChat { .. } => Provider::Windsurf,
    }
}

async fn send_ws_json(client: &mut WebSocket, value: Value) -> AppResult<()> {
    client
        .send(Message::Text(value.to_string()))
        .await
        .map_err(|error| AppError::Upstream(format!("client WebSocket send failed: {error}")))
}

async fn send_ws_app_error(client: &mut WebSocket, error: &AppError) -> AppResult<()> {
    send_ws_json(
        client,
        websocket_request_error(
            error.status(),
            error.code().unwrap_or(error.error_type()),
            error.to_string(),
        ),
    )
    .await
}

async fn send_unsupported_event(client: &mut WebSocket, event_type: Option<&str>) -> AppResult<()> {
    let message = event_type
        .map(|event_type| format!("unsupported Responses WebSocket event: {event_type}"))
        .unwrap_or_else(|| "unsupported Responses WebSocket event".to_string());
    send_ws_json(
        client,
        websocket_request_error(
            StatusCode::BAD_REQUEST,
            "unsupported_websocket_event",
            message,
        ),
    )
    .await
}

async fn send_response_already_in_flight(client: &mut WebSocket) -> AppResult<()> {
    send_ws_json(
        client,
        websocket_request_error(
            StatusCode::BAD_REQUEST,
            "response_already_in_flight",
            "response.create is already in flight",
        ),
    )
    .await
}

fn websocket_request_error(status: StatusCode, code: &str, message: impl ToString) -> Value {
    json!({
        "type": "error",
        "status": status.as_u16(),
        "error": {
            "code": code,
            "message": message.to_string()
        }
    })
}

fn is_response_create_or_raw_body(value: &Value) -> bool {
    value.get("type").and_then(Value::as_str) == Some("response.create")
        || value.get("model").is_some()
}

fn prepare_responses_frame_payload(state: &AppState, value: Value) -> AppResult<String> {
    ensure_responses_websocket_capable(state, &value)?;
    let payload =
        upstream::codex::prepare_response_create_event_payload_with_resolver(value, |model| {
            state.resolve_model_for_format(model, "responses")
        })?;
    Ok(payload.to_string())
}

fn to_upstream_close_frame(frame: Option<CloseFrame<'static>>) -> Option<specter::CloseFrame> {
    frame.map(|frame| specter::CloseFrame {
        code: specter::CloseCode::from_u16(frame.code)
            .unwrap_or(specter::CloseCode::Library(frame.code)),
        reason: frame.reason.into_owned(),
    })
}

fn to_client_close_frame(frame: Option<specter::CloseFrame>) -> Option<CloseFrame<'static>> {
    frame.map(|frame| CloseFrame {
        code: frame.code.as_u16(),
        reason: Cow::Owned(frame.reason),
    })
}

async fn dispatch_responses_websocket(state: &AppState, client: &mut WebSocket) -> AppResult<()> {
    let mut session = BridgeSessionState::default();
    loop {
        let Some(frame) = next_client_data_frame(client).await? else {
            return Ok(());
        };
        let value = client_frame_json(frame)?;
        if !dispatch_responses_client_event(state, client, &mut session, value).await? {
            return Ok(());
        }
    }
}

#[cfg(test)]
mod tests {
    use axum::{body::Bytes, http::HeaderValue};

    use super::*;

    #[test]
    fn realtime_ws_accepts_only_single_gpt_realtime_2_model_query() {
        assert_eq!(
            validate_realtime_model_query(Some("model=gpt-realtime-2")).unwrap(),
            OPENAI_PUBLIC_REALTIME_MODEL
        );

        for query in [
            None,
            Some(""),
            Some("model="),
            Some("model=gpt-realtime-2&model=gpt-realtime-2"),
        ] {
            let error = validate_realtime_model_query(query).unwrap_err();
            assert_eq!(error.status, StatusCode::BAD_REQUEST);
            assert_eq!(error.error_type, "invalid_request_error");
            assert_eq!(error.code, "invalid_realtime_model_query");
            assert_eq!(error.close_code, close_code::POLICY);
        }

        let error = validate_realtime_model_query(Some("model=gpt-realtime")).unwrap_err();
        assert_eq!(error.status, StatusCode::BAD_REQUEST);
        assert_eq!(error.error_type, "invalid_request_error");
        assert_eq!(error.code, "model_not_supported");
        assert_eq!(error.close_code, close_code::POLICY);
    }

    #[test]
    fn realtime_ws_missing_codex_auth_error_is_sanitized_openai_shape() {
        let error = realtime_auth_error(AppError::MissingCredential("~/.codex/auth.json"));
        let frame = error.frame();

        assert_eq!(frame["type"], "error");
        assert_eq!(frame["status"], 401);
        assert_eq!(frame["error"]["type"], "authentication_error");
        assert_eq!(frame["error"]["code"], "invalid_api_key");
        assert_eq!(frame["error"]["message"], MISSING_CODEX_AUTH_MESSAGE);
        assert!(frame["error"]["param"].is_null());
    }

    #[tokio::test]
    async fn bridge_stream_eof_after_created_emits_response_failed() {
        let stream = futures::stream::iter(vec![Ok::<_, std::io::Error>(Bytes::from_static(
            b"event: response.created\ndata: {\"type\":\"response.created\",\"response\":{\"id\":\"resp_1\"}}\n\n",
        ))]);
        let response = UpstreamResponse::stream(
            "bedrock",
            StatusCode::OK,
            crate::upstream_response::sse_headers(),
            stream,
        );
        let (sender, mut receiver) = mpsc::channel(8);

        let error = forward_responses_response_to_ws(sender, response, json!({}))
            .await
            .unwrap_err();

        let created = receiver.recv().await.unwrap();
        let failed = receiver.recv().await.unwrap();
        assert!(error
            .to_string()
            .contains("ended before terminal response event"));
        assert_eq!(created["type"], "response.created");
        assert_eq!(failed["type"], "response.failed");
        assert_eq!(failed["response"]["id"], "resp_1");
        assert!(failed["response"]["error"]["message"]
            .as_str()
            .unwrap()
            .contains("ended before terminal response event"));
    }

    #[tokio::test]
    async fn bridge_non_success_before_created_emits_top_level_error() {
        let response = UpstreamResponse::bytes(
            "bedrock",
            StatusCode::BAD_GATEWAY,
            HeaderMap::new(),
            Bytes::from_static(b"provider down"),
        );
        let (sender, mut receiver) = mpsc::channel(8);

        let error = forward_responses_response_to_ws(sender, response, json!({}))
            .await
            .unwrap_err();

        let frame = receiver.recv().await.unwrap();
        assert!(error.to_string().contains("502 Bad Gateway"));
        assert_eq!(frame["type"], "error");
        assert_eq!(frame["status"], 502);
        assert_eq!(frame["error"]["code"], "upstream_error");
    }

    #[tokio::test]
    async fn bridge_non_stream_success_emits_output_item_done_before_completed() {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );
        let response = UpstreamResponse::bytes(
            "bedrock",
            StatusCode::OK,
            headers,
            Bytes::from_static(
                br#"{
                    "id":"resp_json_1",
                    "status":"completed",
                    "output":[{
                        "id":"msg_1",
                        "type":"message",
                        "status":"completed",
                        "role":"assistant",
                        "content":[{"type":"output_text","text":"hello"}]
                    }],
                    "usage":{
                        "input_tokens":1,
                        "input_tokens_details":{"cached_tokens":0},
                        "output_tokens":1,
                        "output_tokens_details":{"reasoning_tokens":0},
                        "total_tokens":2
                    }
                }"#,
            ),
        );
        let (sender, mut receiver) = mpsc::channel(8);

        let result = forward_responses_response_to_ws(sender, response, json!({}))
            .await
            .unwrap();

        let created = receiver.recv().await.unwrap();
        let added = receiver.recv().await.unwrap();
        let done = receiver.recv().await.unwrap();
        let completed = receiver.recv().await.unwrap();
        assert!(receiver.try_recv().is_err());
        assert_eq!(result.response_id, "resp_json_1");
        assert_eq!(created["type"], "response.created");
        assert_eq!(added["type"], "response.output_item.added");
        assert_eq!(added["item"]["status"], "in_progress");
        assert_eq!(done["type"], "response.output_item.done");
        assert_eq!(done["item"]["content"][0]["text"], "hello");
        assert_eq!(completed["type"], "response.completed");
    }

    #[tokio::test]
    async fn bridge_top_level_stream_error_after_created_emits_single_response_failed() {
        let stream = futures::stream::iter(vec![Ok::<_, std::io::Error>(Bytes::from_static(
            b"event: response.created\ndata: {\"type\":\"response.created\",\"response\":{\"id\":\"resp_1\"}}\n\nevent: error\ndata: {\"type\":\"error\",\"error\":{\"code\":\"provider_error\",\"message\":\"boom\"}}\n\n",
        ))]);
        let response = UpstreamResponse::stream(
            "bedrock",
            StatusCode::OK,
            crate::upstream_response::sse_headers(),
            stream,
        );
        let (sender, mut receiver) = mpsc::channel(8);

        let result = forward_responses_response_to_ws(sender, response, json!({}))
            .await
            .unwrap();

        let created = receiver.recv().await.unwrap();
        let failed = receiver.recv().await.unwrap();
        assert!(receiver.try_recv().is_err());
        assert_eq!(result.response_id, "resp_1");
        assert_eq!(created["type"], "response.created");
        assert_eq!(failed["type"], "response.failed");
        assert_eq!(failed["response"]["id"], "resp_1");
        assert_eq!(failed["response"]["error"]["code"], "provider_error");
        assert_eq!(failed["response"]["error"]["message"], "boom");
    }

    #[test]
    fn bridge_accepts_independent_route_switch_without_previous_response_id() {
        let mut session = BridgeSessionState::default();
        session.record(BridgeResponseState {
            fingerprint: BridgeRouteFingerprint {
                route: ResponsesRoute::BedrockMessages,
                model: "claude-opus-4-7".into(),
                upstream_model: "anthropic.claude-opus-4-7".into(),
                target_format: "responses".into(),
                remote_compaction_policy: RemoteCompactionPolicy::Local,
            },
            response_id: "resp_prior".into(),
            full_request: json!({
                "model": "claude-opus-4-7",
                "input": "prior"
            }),
            output_item_done_items: Vec::new(),
        });
        let request = json!({
            "model": "gpt-5.5",
            "input": "independent turn"
        });
        let fingerprint = BridgeRouteFingerprint {
            route: ResponsesRoute::CodexResponses,
            model: "gpt-5.5".into(),
            upstream_model: "gpt-5.5".into(),
            target_format: "responses".into(),
            remote_compaction_policy: RemoteCompactionPolicy::Local,
        };

        let prepared = prepare_bridge_request(request, &fingerprint, &session).unwrap();

        assert_eq!(prepared["model"], "gpt-5.5");
        assert_eq!(prepared["input"], "independent turn");
    }

    #[test]
    fn bridge_allows_null_previous_response_id_as_fresh_chain() {
        let session = BridgeSessionState::default();
        let request = json!({
            "model": "gpt-5.5",
            "input": "fresh chain",
            "previous_response_id": null
        });
        let fingerprint = BridgeRouteFingerprint {
            route: ResponsesRoute::CodexResponses,
            model: "gpt-5.5".into(),
            upstream_model: "gpt-5.5".into(),
            target_format: "responses".into(),
            remote_compaction_policy: RemoteCompactionPolicy::Local,
        };

        let prepared = prepare_bridge_request(request, &fingerprint, &session).unwrap();

        assert!(prepared.get("previous_response_id").is_none());
        assert_eq!(prepared["input"], "fresh chain");
    }

    #[test]
    fn bridge_rejects_non_string_previous_response_id() {
        let session = BridgeSessionState::default();
        let request = json!({
            "model": "gpt-5.5",
            "input": "bad continuation",
            "previous_response_id": 42
        });
        let fingerprint = BridgeRouteFingerprint {
            route: ResponsesRoute::CodexResponses,
            model: "gpt-5.5".into(),
            upstream_model: "gpt-5.5".into(),
            target_format: "responses".into(),
            remote_compaction_policy: RemoteCompactionPolicy::Local,
        };

        let error = prepare_bridge_request(request, &fingerprint, &session).unwrap_err();

        assert_eq!(error.code, "previous_response_field_mismatch");
    }

    #[test]
    fn bridge_rejects_unknown_previous_response_id() {
        let session = BridgeSessionState::default();
        let request = json!({
            "model": "gpt-5.5",
            "input": [],
            "previous_response_id": "resp_missing"
        });
        let fingerprint = BridgeRouteFingerprint {
            route: ResponsesRoute::CodexResponses,
            model: "gpt-5.5".into(),
            upstream_model: "gpt-5.5".into(),
            target_format: "responses".into(),
            remote_compaction_policy: RemoteCompactionPolicy::Local,
        };

        let error = prepare_bridge_request(request, &fingerprint, &session).unwrap_err();

        assert_eq!(error.code, "unknown_previous_response_id");
    }

    #[test]
    fn bridge_previous_response_id_rejects_same_lane_different_model() {
        let mut session = BridgeSessionState::default();
        session.record(BridgeResponseState {
            fingerprint: BridgeRouteFingerprint {
                route: ResponsesRoute::BedrockMessages,
                model: "claude-opus-4-7".into(),
                upstream_model: "anthropic.claude-opus-4-7".into(),
                target_format: "responses".into(),
                remote_compaction_policy: RemoteCompactionPolicy::Local,
            },
            response_id: "resp_prewarm".into(),
            full_request: json!({
                "model": "claude-opus-4-7",
                "input": "warm"
            }),
            output_item_done_items: Vec::new(),
        });
        let request = json!({
            "model": "claude-sonnet-4-7",
            "input": [],
            "previous_response_id": "resp_prewarm"
        });
        let fingerprint = BridgeRouteFingerprint {
            route: ResponsesRoute::BedrockMessages,
            model: "claude-sonnet-4-7".into(),
            upstream_model: "anthropic.claude-sonnet-4-7".into(),
            target_format: "responses".into(),
            remote_compaction_policy: RemoteCompactionPolicy::Local,
        };

        let error = prepare_bridge_request(request, &fingerprint, &session).unwrap_err();

        assert_eq!(error.code, "previous_response_model_mismatch");
    }

    #[test]
    fn bridge_previous_response_id_rejects_cross_lane_mismatch() {
        let mut session = BridgeSessionState::default();
        session.record(BridgeResponseState {
            fingerprint: BridgeRouteFingerprint {
                route: ResponsesRoute::BedrockMessages,
                model: "claude-opus-4-7".into(),
                upstream_model: "anthropic.claude-opus-4-7".into(),
                target_format: "responses".into(),
                remote_compaction_policy: RemoteCompactionPolicy::Local,
            },
            response_id: "resp_prewarm".into(),
            full_request: json!({
                "model": "claude-opus-4-7",
                "input": "warm",
                "generate": false
            }),
            output_item_done_items: Vec::new(),
        });
        let request = json!({
            "model": "gpt-5.5",
            "input": [],
            "previous_response_id": "resp_prewarm"
        });
        let fingerprint = BridgeRouteFingerprint {
            route: ResponsesRoute::CodexResponses,
            model: "gpt-5.5".into(),
            upstream_model: "gpt-5.5".into(),
            target_format: "responses".into(),
            remote_compaction_policy: RemoteCompactionPolicy::Local,
        };

        let error = prepare_bridge_request(request, &fingerprint, &session).unwrap_err();

        assert_eq!(error.code, "previous_response_route_mismatch");
    }

    #[test]
    fn bridge_previous_response_id_rejects_target_format_mismatch() {
        let mut session = BridgeSessionState::default();
        session.record(BridgeResponseState {
            fingerprint: BridgeRouteFingerprint {
                route: ResponsesRoute::GoogleGenerateContent {
                    upstream_model: "gemini-3.1-flash-lite".into(),
                },
                model: "facade-google-model".into(),
                upstream_model: "gemini-3.1-flash-lite".into(),
                target_format: "responses".into(),
                remote_compaction_policy: RemoteCompactionPolicy::Local,
            },
            response_id: "resp_prewarm".into(),
            full_request: json!({
                "model": "facade-google-model",
                "input": "warm",
                "generate": false
            }),
            output_item_done_items: Vec::new(),
        });
        let request = json!({
            "model": "facade-google-model",
            "input": [],
            "previous_response_id": "resp_prewarm"
        });
        let fingerprint = BridgeRouteFingerprint {
            route: ResponsesRoute::GoogleGenerateContent {
                upstream_model: "gemini-3.1-flash-lite".into(),
            },
            model: "facade-google-model".into(),
            upstream_model: "gemini-3.1-flash-lite".into(),
            target_format: "google_generate_content".into(),
            remote_compaction_policy: RemoteCompactionPolicy::Local,
        };

        let error = prepare_bridge_request(request, &fingerprint, &session).unwrap_err();

        assert_eq!(error.code, "previous_response_target_format_mismatch");
    }

    #[test]
    fn bridge_lru_evicts_old_previous_response_ids() {
        let mut session = BridgeSessionState::default();
        let fingerprint = BridgeRouteFingerprint {
            route: ResponsesRoute::CodexResponses,
            model: "gpt-5.5".into(),
            upstream_model: "gpt-5.5".into(),
            target_format: "responses".into(),
            remote_compaction_policy: RemoteCompactionPolicy::Local,
        };
        for index in 0..=BRIDGE_RESPONSE_STATE_LIMIT {
            session.record(BridgeResponseState {
                fingerprint: fingerprint.clone(),
                response_id: format!("resp_{index}"),
                full_request: json!({
                    "model": "gpt-5.5",
                    "input": format!("turn {index}")
                }),
                output_item_done_items: Vec::new(),
            });
        }
        let request = json!({
            "model": "gpt-5.5",
            "input": [],
            "previous_response_id": "resp_0"
        });

        let error = prepare_bridge_request(request, &fingerprint, &session).unwrap_err();

        assert_eq!(error.code, "unknown_previous_response_id");
    }
}
