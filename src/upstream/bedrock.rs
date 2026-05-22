use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use axum::{
    body::to_bytes,
    http::{header, HeaderMap, HeaderName, HeaderValue, StatusCode},
};
use base64::{engine::general_purpose::STANDARD as BASE64_STANDARD, Engine as _};
use bytes::Bytes;
use futures::{stream, StreamExt};
use rand_core::{OsRng, RngCore};
use serde_json::Value;

use crate::{
    auth::bedrock::{resolve_bedrock_auth, BedrockAuth},
    model_alias::{self, Provider},
    AppError, AppResult, AppState, UpstreamResponse,
};

pub const BEDROCK_RUNTIME_SERVICE: &str = "bedrock";
pub const BEDROCK_PROVIDER: &str = "bedrock";
pub const BEDROCK_RUNTIME_ANTHROPIC_VERSION: &str = "bedrock-2023-05-31";
pub const DEFAULT_BEDROCK_MAX_ATTEMPTS: usize = 6;
const BEDROCK_RETRY_BASE_DELAY_MS: u64 = 100;
const BEDROCK_RETRY_MAX_DELAY_MS: u64 = 2_000;
const AWS_EVENT_STREAM_MIN_MESSAGE_LEN: usize = 16;
const AWS_EVENT_STREAM_MAX_MESSAGE_LEN: usize = 16 * 1024 * 1024;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum BedrockRuntimeAuthSelection {
    Header {
        name: &'static str,
        value: String,
        source: &'static str,
    },
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct BedrockRuntimeInvokeRequest {
    pub url: String,
    pub body: Value,
    pub auth: BedrockRuntimeAuthSelection,
    pub headers: HeaderMap,
    pub stream: bool,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct BedrockRetryPolicy {
    pub max_attempts: usize,
}

impl Default for BedrockRetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: DEFAULT_BEDROCK_MAX_ATTEMPTS,
        }
    }
}

pub fn runtime_invoke_url(region: &str, model_id: &str) -> String {
    runtime_model_url(region, model_id, "invoke")
}

pub fn runtime_invoke_with_response_stream_url(region: &str, model_id: &str) -> String {
    runtime_model_url(region, model_id, "invoke-with-response-stream")
}

fn runtime_model_url(region: &str, model_id: &str, operation: &str) -> String {
    format!(
        "https://bedrock-runtime.{region}.amazonaws.com/model/{}/{operation}",
        percent_encode_model_id(model_id)
    )
}

pub fn resolve_bedrock_runtime_model_id(model: &str) -> AppResult<&'static str> {
    let alias = model_alias::resolve_model(model)
        .ok_or_else(|| AppError::ModelNotSupported(model.to_string()))?;
    if alias.provider != Provider::Bedrock {
        return Err(AppError::ModelNotSupported(model.to_string()));
    }
    Ok(alias.upstream_model)
}

pub fn select_bedrock_runtime_auth(
    auth: BedrockAuth,
    _region: &str,
) -> BedrockRuntimeAuthSelection {
    match auth {
        BedrockAuth::Bearer { token, source } => BedrockRuntimeAuthSelection::Header {
            name: "authorization",
            value: token,
            source,
        },
    }
}

pub fn build_runtime_invoke_request(
    state: &AppState,
    body: Value,
    headers: &HeaderMap,
    model_id: &str,
) -> AppResult<BedrockRuntimeInvokeRequest> {
    let operation = if request_stream_enabled(&body) {
        RuntimeInvokeOperation::InvokeWithResponseStream
    } else {
        RuntimeInvokeOperation::Invoke
    };
    build_runtime_invoke_request_with_operation(state, body, headers, model_id, operation)
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum RuntimeInvokeOperation {
    Invoke,
    InvokeWithResponseStream,
}

impl RuntimeInvokeOperation {
    fn url(self, region: &str, model_id: &str) -> String {
        match self {
            Self::Invoke => runtime_invoke_url(region, model_id),
            Self::InvokeWithResponseStream => {
                runtime_invoke_with_response_stream_url(region, model_id)
            }
        }
    }
}

fn build_runtime_invoke_request_with_operation(
    state: &AppState,
    mut body: Value,
    headers: &HeaderMap,
    model_id: &str,
    operation: RuntimeInvokeOperation,
) -> AppResult<BedrockRuntimeInvokeRequest> {
    if let Some(object) = body.as_object_mut() {
        object.remove("model");
        object.remove("stream");
        object
            .entry("anthropic_version")
            .or_insert_with(|| Value::String(BEDROCK_RUNTIME_ANTHROPIC_VERSION.into()));
    }
    let auth = select_bedrock_runtime_auth(resolve_bedrock_auth(state)?, &state.bedrock_region);
    Ok(BedrockRuntimeInvokeRequest {
        url: operation.url(&state.bedrock_region, model_id),
        body,
        auth,
        headers: runtime_forward_headers(
            headers,
            operation == RuntimeInvokeOperation::InvokeWithResponseStream,
        ),
        stream: operation == RuntimeInvokeOperation::InvokeWithResponseStream,
    })
}

fn request_stream_enabled(body: &Value) -> bool {
    body.get("stream").and_then(Value::as_bool).unwrap_or(false)
}

pub async fn forward_messages(
    state: &AppState,
    body: serde_json::Value,
    headers: HeaderMap,
) -> AppResult<Bytes> {
    let response = forward_messages_response(state, body, headers).await?;
    to_bytes(response.body, usize::MAX)
        .await
        .map_err(|error| AppError::Upstream(error.to_string()))
}

pub async fn forward_messages_response(
    state: &AppState,
    body: serde_json::Value,
    headers: HeaderMap,
) -> AppResult<UpstreamResponse> {
    let model = body
        .get("model")
        .and_then(|value| value.as_str())
        .ok_or_else(|| AppError::BadRequest("missing model".into()))?;
    let normalized_model = resolve_bedrock_runtime_model_id(model)?;

    send_runtime_invoke_request(
        &state.http,
        build_runtime_invoke_request(state, body, &headers, normalized_model)?,
        BedrockRetryPolicy::default(),
    )
    .await
}

pub async fn send_runtime_invoke_request(
    client: &reqwest::Client,
    request: BedrockRuntimeInvokeRequest,
    retry_policy: BedrockRetryPolicy,
) -> AppResult<UpstreamResponse> {
    let attempts = retry_policy.max_attempts.max(1);
    let started = Instant::now();
    let mut last_error = None;

    let body_bytes = Bytes::from(serde_json::to_vec(&request.body)?);

    for attempt in 1..=attempts {
        match send_runtime_once(client, &request, &body_bytes).await {
            Ok(response) if should_retry_status(response.status()) && attempt < attempts => {
                wait_before_retry(attempt).await;
                continue;
            }
            Ok(response) => {
                let response = if request.stream && response.status().is_success() {
                    runtime_eventstream_response_from_reqwest(response)
                } else {
                    UpstreamResponse::from_reqwest(BEDROCK_PROVIDER, response)
                };
                return Ok(response.with_latency_ms(started.elapsed().as_millis()));
            }
            Err(BedrockSendError::Reqwest(error))
                if should_retry_error(&error) && attempt < attempts =>
            {
                last_error = Some(error);
                wait_before_retry(attempt).await;
            }
            Err(BedrockSendError::Reqwest(error)) => return Err(reqwest_error(error)),
            Err(BedrockSendError::App(error)) => return Err(error),
        }
    }

    Err(reqwest_error(
        last_error.expect("retry loop must retain the last reqwest error"),
    ))
}

async fn wait_before_retry(attempt: usize) {
    tokio::time::sleep(bedrock_retry_delay(attempt, OsRng.next_u64())).await;
}

pub fn bedrock_retry_delay(attempt: usize, jitter_seed: u64) -> Duration {
    let exponent = attempt.saturating_sub(1).min(10) as u32;
    let exponential_delay_ms = BEDROCK_RETRY_BASE_DELAY_MS.saturating_mul(2_u64.pow(exponent));
    let capped_delay_ms = exponential_delay_ms.min(BEDROCK_RETRY_MAX_DELAY_MS);
    let jitter_floor_ms = capped_delay_ms / 2;
    let jitter_range_ms = capped_delay_ms.saturating_sub(jitter_floor_ms).max(1);
    Duration::from_millis(jitter_floor_ms + (jitter_seed % (jitter_range_ms + 1)))
}

async fn send_runtime_once(
    client: &reqwest::Client,
    request: &BedrockRuntimeInvokeRequest,
    body: &Bytes,
) -> Result<reqwest::Response, BedrockSendError> {
    let mut headers = request.headers.clone();
    apply_runtime_auth_headers(request, &mut headers).await?;

    client
        .post(&request.url)
        .headers(headers)
        .body(body.clone())
        .send()
        .await
        .map_err(BedrockSendError::Reqwest)
}

fn runtime_eventstream_response_from_reqwest(response: reqwest::Response) -> UpstreamResponse {
    let status = response.status();
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/event-stream"),
    );
    let decoder = AwsEventStreamDecoder::default();
    let stream = stream::unfold(
        (response.bytes_stream(), decoder, false),
        move |(mut bytes_stream, mut decoder, stream_ended)| async move {
            if stream_ended {
                return None;
            }
            match bytes_stream.next().await {
                Some(Ok(bytes)) => match decoder.push(bytes) {
                    Ok(chunks) => Some((Ok(chunks), (bytes_stream, decoder, false))),
                    Err(error) => Some((Err(error), (bytes_stream, decoder, true))),
                },
                Some(Err(error)) => {
                    let err = AppError::Upstream(format!("Bedrock Runtime stream failed: {error}"));
                    Some((Err(err), (bytes_stream, decoder, true)))
                }
                None => {
                    if !decoder.has_seen_terminal_event {
                        let err =
                            AppError::Upstream("premature EOF before message_stop event".into());
                        Some((Err(err), (bytes_stream, decoder, true)))
                    } else {
                        None
                    }
                }
            }
        },
    )
    .flat_map(|result| match result {
        Ok(chunks) => {
            let mapped = chunks.into_iter().map(Ok).collect::<Vec<_>>();
            stream::iter(mapped)
        }
        Err(error) => stream::iter(vec![Err(error)]),
    });

    UpstreamResponse::stream(BEDROCK_PROVIDER, status, headers, stream)
}

#[derive(Default)]
struct AwsEventStreamDecoder {
    pending: Vec<u8>,
    has_seen_terminal_event: bool,
}

impl AwsEventStreamDecoder {
    fn push(&mut self, chunk: Bytes) -> AppResult<Vec<Bytes>> {
        self.pending.extend_from_slice(&chunk);
        let mut chunks = Vec::new();
        loop {
            if self.pending.len() < 12 {
                break;
            }
            let total_len = u32::from_be_bytes([
                self.pending[0],
                self.pending[1],
                self.pending[2],
                self.pending[3],
            ]) as usize;
            let headers_len = u32::from_be_bytes([
                self.pending[4],
                self.pending[5],
                self.pending[6],
                self.pending[7],
            ]) as usize;
            if !(AWS_EVENT_STREAM_MIN_MESSAGE_LEN..=AWS_EVENT_STREAM_MAX_MESSAGE_LEN)
                .contains(&total_len)
            {
                return Err(AppError::Upstream(format!(
                    "invalid Bedrock Runtime event stream message length: {total_len}"
                )));
            }
            if 12 + headers_len + 4 > total_len {
                return Err(AppError::Upstream(
                    "invalid Bedrock Runtime event stream headers length".into(),
                ));
            }
            if self.pending.len() < total_len {
                break;
            }

            let message = self.pending.drain(..total_len).collect::<Vec<_>>();
            let headers_end = 12 + headers_len;
            let payload_end = total_len - 4;
            let headers = parse_event_stream_headers(&message[12..headers_end])?;
            let payload = &message[headers_end..payload_end];
            if let Some((chunk, terminal)) = runtime_event_stream_message_to_sse(&headers, payload)?
            {
                if terminal {
                    self.has_seen_terminal_event = true;
                }
                chunks.push(chunk);
            }
        }
        Ok(chunks)
    }
}

fn runtime_event_stream_message_to_sse(
    headers: &HashMap<String, String>,
    payload: &[u8],
) -> AppResult<Option<(Bytes, bool)>> {
    let event_type = headers.get(":event-type").map(String::as_str);
    match event_type {
        Some("chunk") => runtime_chunk_payload_to_sse(payload),
        Some(event) if event.ends_with("Exception") || event.ends_with("Error") => {
            Err(runtime_event_stream_error(event, payload))
        }
        _ if headers.get(":message-type").map(String::as_str) == Some("exception") => {
            let event = headers
                .get(":exception-type")
                .or_else(|| headers.get(":event-type"))
                .map(String::as_str)
                .unwrap_or("exception");
            Err(runtime_event_stream_error(event, payload))
        }
        _ => Ok(None),
    }
}

fn runtime_chunk_payload_to_sse(payload: &[u8]) -> AppResult<Option<(Bytes, bool)>> {
    let payload_bytes = runtime_chunk_payload_bytes(payload)?;
    if payload_bytes.is_empty() {
        return Ok(None);
    }
    if payload_bytes.starts_with(b"data:") || payload_bytes.starts_with(b"event:") {
        let terminal = std::str::from_utf8(&payload_bytes)
            .ok()
            .is_some_and(|text| text.contains("event: message_stop"));
        return Ok(Some((Bytes::from(payload_bytes), terminal)));
    }
    let text = std::str::from_utf8(&payload_bytes).map_err(|error| {
        AppError::Upstream(format!("Bedrock Runtime chunk was not UTF-8: {error}"))
    })?;
    let text = text.trim_end_matches(['\r', '\n']);

    // Parse payload as Anthropic messages stream JSON
    if let Ok(value) = serde_json::from_str::<Value>(text) {
        if let Some(event_type) = value.get("type").and_then(Value::as_str) {
            return Ok(Some((
                Bytes::from(format!("event: {event_type}\ndata: {text}\n\n")),
                event_type == "message_stop",
            )));
        }
    }

    Ok(Some((Bytes::from(format!("data: {text}\n\n")), false)))
}

fn runtime_chunk_payload_bytes(payload: &[u8]) -> AppResult<Vec<u8>> {
    let Ok(value) = serde_json::from_slice::<Value>(payload) else {
        return Ok(payload.to_vec());
    };
    if let Some(encoded) = value.get("bytes").and_then(Value::as_str) {
        return BASE64_STANDARD.decode(encoded).map_err(|error| {
            AppError::Upstream(format!(
                "Bedrock Runtime chunk bytes were not base64: {error}"
            ))
        });
    }
    Ok(payload.to_vec())
}

fn runtime_event_stream_error(event: &str, payload: &[u8]) -> AppError {
    let message = serde_json::from_slice::<Value>(payload)
        .ok()
        .and_then(|value| {
            value
                .get("message")
                .or_else(|| value.get("Message"))
                .and_then(Value::as_str)
                .map(ToOwned::to_owned)
        })
        .or_else(|| std::str::from_utf8(payload).ok().map(ToOwned::to_owned))
        .unwrap_or_else(|| "Bedrock Runtime stream failed".to_string());
    AppError::Upstream(format!("Bedrock Runtime stream {event}: {message}"))
}

fn parse_event_stream_headers(bytes: &[u8]) -> AppResult<HashMap<String, String>> {
    let mut index = 0;
    let mut headers = HashMap::new();
    while index < bytes.len() {
        let name_len = *bytes.get(index).ok_or_else(|| {
            AppError::Upstream("truncated Bedrock Runtime event stream header".into())
        })? as usize;
        index += 1;
        let name_end = index + name_len;
        let name = std::str::from_utf8(bytes.get(index..name_end).ok_or_else(|| {
            AppError::Upstream("truncated Bedrock Runtime event stream header name".into())
        })?)
        .map_err(|error| {
            AppError::Upstream(format!(
                "invalid Bedrock Runtime event stream header name: {error}"
            ))
        })?
        .to_string();
        index = name_end;
        let value_type = *bytes.get(index).ok_or_else(|| {
            AppError::Upstream("missing Bedrock Runtime event stream header value type".into())
        })?;
        index += 1;
        if let Some(value) = parse_event_stream_header_value(bytes, &mut index, value_type)? {
            headers.insert(name, value);
        }
    }
    Ok(headers)
}

fn parse_event_stream_header_value(
    bytes: &[u8],
    index: &mut usize,
    value_type: u8,
) -> AppResult<Option<String>> {
    match value_type {
        0 => Ok(Some("true".into())),
        1 => Ok(Some("false".into())),
        2 => {
            *index += 1;
            Ok(None)
        }
        3 => {
            *index += 2;
            Ok(None)
        }
        4 => {
            *index += 4;
            Ok(None)
        }
        5 | 8 => {
            *index += 8;
            Ok(None)
        }
        6 | 7 => {
            let len = event_stream_u16(bytes, *index)? as usize;
            *index += 2;
            let value_end = *index + len;
            let value = if value_type == 7 {
                Some(
                    std::str::from_utf8(bytes.get(*index..value_end).ok_or_else(|| {
                        AppError::Upstream(
                            "truncated Bedrock Runtime event stream string header".into(),
                        )
                    })?)
                    .map_err(|error| {
                        AppError::Upstream(format!(
                            "invalid Bedrock Runtime event stream string header: {error}"
                        ))
                    })?
                    .to_string(),
                )
            } else {
                None
            };
            *index = value_end;
            Ok(value)
        }
        9 => {
            *index += 16;
            Ok(None)
        }
        other => Err(AppError::Upstream(format!(
            "unsupported Bedrock Runtime event stream header value type: {other}"
        ))),
    }
}

fn event_stream_u16(bytes: &[u8], index: usize) -> AppResult<u16> {
    let value = bytes.get(index..index + 2).ok_or_else(|| {
        AppError::Upstream("truncated Bedrock Runtime event stream header length".into())
    })?;
    Ok(u16::from_be_bytes([value[0], value[1]]))
}

async fn apply_runtime_auth_headers(
    request: &BedrockRuntimeInvokeRequest,
    headers: &mut HeaderMap,
) -> AppResult<()> {
    match &request.auth {
        BedrockRuntimeAuthSelection::Header { value, .. } => {
            let value = format!("Bearer {value}");
            headers.insert(
                header::AUTHORIZATION,
                HeaderValue::from_str(&value).map_err(|_| {
                    AppError::BadRequest("invalid authorization header value".into())
                })?,
            );
            Ok(())
        }
    }
}

#[derive(Debug)]
enum BedrockSendError {
    Reqwest(reqwest::Error),
    App(AppError),
}

impl From<AppError> for BedrockSendError {
    fn from(error: AppError) -> Self {
        Self::App(error)
    }
}

impl From<serde_json::Error> for BedrockSendError {
    fn from(error: serde_json::Error) -> Self {
        Self::App(AppError::Json(error))
    }
}

pub fn runtime_forward_headers(headers: &HeaderMap, is_stream: bool) -> HeaderMap {
    let mut forwarded = HeaderMap::new();
    forwarded.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/json"),
    );

    if is_stream {
        forwarded.insert(
            header::ACCEPT,
            HeaderValue::from_static("application/vnd.amazon.eventstream"),
        );
        forwarded.insert(
            HeaderName::from_static("x-amzn-bedrock-accept"),
            HeaderValue::from_static("application/json"),
        );
    } else {
        forwarded.insert(header::ACCEPT, HeaderValue::from_static("application/json"));
    }

    let allowlist = [
        "x-amzn-bedrock-trace",
        "x-amzn-bedrock-guardrailidentifier",
        "x-amzn-bedrock-guardrailversion",
        "x-amzn-bedrock-performanceconfig-latency",
        "x-amzn-bedrock-service-tier",
    ];

    for name_str in allowlist {
        if let Ok(name) = HeaderName::from_bytes(name_str.as_bytes()) {
            if let Some(value) = headers.get(&name) {
                forwarded.insert(name, value.clone());
            }
        }
    }

    if is_stream {
        let bedrock_accept_name = HeaderName::from_static("x-amzn-bedrock-accept");
        if let Some(value) = headers.get(&bedrock_accept_name) {
            forwarded.insert(bedrock_accept_name, value.clone());
        }
    }

    forwarded
}

fn percent_encode_model_id(model_id: &str) -> String {
    let mut encoded = String::new();
    for byte in model_id.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                encoded.push(byte as char)
            }
            other => encoded.push_str(&format!("%{other:02X}")),
        }
    }
    encoded
}

pub fn should_retry_status(status: StatusCode) -> bool {
    matches!(
        status,
        StatusCode::REQUEST_TIMEOUT
            | StatusCode::TOO_MANY_REQUESTS
            | StatusCode::INTERNAL_SERVER_ERROR
            | StatusCode::BAD_GATEWAY
            | StatusCode::SERVICE_UNAVAILABLE
            | StatusCode::GATEWAY_TIMEOUT
    )
}

pub fn should_retry_error(error: &reqwest::Error) -> bool {
    error.is_connect() || error.is_timeout()
}

fn reqwest_error(error: reqwest::Error) -> AppError {
    AppError::Upstream(error.to_string())
}
