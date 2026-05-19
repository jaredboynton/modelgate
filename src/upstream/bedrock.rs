use std::{
    collections::HashMap,
    time::{Duration, Instant, SystemTime},
};

use aws_credential_types::{provider::ProvideCredentials, Credentials};
use aws_sigv4::{
    http_request::{sign, SignableBody, SignableRequest, SigningSettings},
    sign::v4,
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

pub const MANTLE_MESSAGES_PATH: &str = "/anthropic/v1/messages";
pub const MANTLE_SERVICE: &str = "bedrock-mantle";
pub const BEDROCK_RUNTIME_SERVICE: &str = "bedrock";
pub const BEDROCK_PROVIDER: &str = "bedrock";
pub const DEFAULT_ANTHROPIC_VERSION: &str = "2023-06-01";
pub const BEDROCK_RUNTIME_ANTHROPIC_VERSION: &str = "bedrock-2023-05-31";
pub const DEFAULT_MANTLE_MAX_ATTEMPTS: usize = 6;
const MANTLE_RETRY_BASE_DELAY_MS: u64 = 100;
const MANTLE_RETRY_MAX_DELAY_MS: u64 = 2_000;
const AWS_EVENT_STREAM_MIN_MESSAGE_LEN: usize = 16;
const AWS_EVENT_STREAM_MAX_MESSAGE_LEN: usize = 16 * 1024 * 1024;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum MantleAuthSelection {
    Header {
        name: &'static str,
        value: String,
        source: &'static str,
    },
    Profile {
        profile: String,
        region: String,
        service: &'static str,
    },
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct MantleMessagesRequest {
    pub url: String,
    pub path: &'static str,
    pub body: Value,
    pub auth: MantleAuthSelection,
    pub headers: HeaderMap,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct MantleRetryPolicy {
    pub max_attempts: usize,
}

impl Default for MantleRetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: DEFAULT_MANTLE_MAX_ATTEMPTS,
        }
    }
}

pub fn mantle_base_url(region: &str) -> String {
    format!("https://bedrock-mantle.{region}.api.aws")
}

pub fn mantle_messages_url(region: &str) -> String {
    format!("{}{}", mantle_base_url(region), MANTLE_MESSAGES_PATH)
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

pub fn normalize_mantle_model(model: &str) -> AppResult<&'static str> {
    let alias = model_alias::resolve_model(model)
        .ok_or_else(|| AppError::ModelNotSupported(model.to_string()))?;
    if alias.provider != Provider::Bedrock {
        return Err(AppError::ModelNotSupported(model.to_string()));
    }
    Ok(alias.upstream_model)
}

pub fn is_runtime_inference_profile_model(model: &str) -> bool {
    matches!(
        model.split_once('.').map(|(prefix, _)| prefix),
        Some("us" | "eu" | "au" | "jp" | "global")
    )
}

pub fn select_mantle_auth(auth: BedrockAuth, region: &str) -> MantleAuthSelection {
    match auth {
        BedrockAuth::Bearer { token, source } => MantleAuthSelection::Header {
            name: "x-api-key",
            value: token,
            source,
        },
        BedrockAuth::Profile { name } => MantleAuthSelection::Profile {
            profile: name,
            region: region.to_string(),
            service: MANTLE_SERVICE,
        },
    }
}

pub fn build_mantle_messages_request(
    state: &AppState,
    mut body: Value,
    headers: &HeaderMap,
) -> AppResult<MantleMessagesRequest> {
    let model = body
        .get("model")
        .and_then(|value| value.as_str())
        .ok_or_else(|| AppError::BadRequest("missing model".into()))?;
    body["model"] = Value::String(normalize_mantle_model(model)?.to_string());

    let auth = select_mantle_auth(resolve_bedrock_auth(state)?, &state.bedrock_region);
    Ok(MantleMessagesRequest {
        url: mantle_messages_url(&state.bedrock_region),
        path: MANTLE_MESSAGES_PATH,
        body,
        auth,
        headers: mantle_forward_headers(headers),
    })
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
    let normalized_model = normalize_mantle_model(model)?;
    if is_runtime_inference_profile_model(normalized_model) {
        return send_runtime_invoke_request(
            &state.http,
            build_runtime_invoke_request(state, body, &headers, normalized_model)?,
            MantleRetryPolicy::default(),
        )
        .await;
    }

    let request = build_mantle_messages_request(state, body, &headers)?;
    send_mantle_messages_request(&state.http, request, MantleRetryPolicy::default()).await
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct RuntimeInvokeRequest {
    pub url: String,
    pub body: Value,
    pub auth: MantleAuthSelection,
    pub headers: HeaderMap,
    pub stream: bool,
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

pub fn build_runtime_invoke_request(
    state: &AppState,
    body: Value,
    headers: &HeaderMap,
    model_id: &str,
) -> AppResult<RuntimeInvokeRequest> {
    let operation = if request_stream_enabled(&body) {
        RuntimeInvokeOperation::InvokeWithResponseStream
    } else {
        RuntimeInvokeOperation::Invoke
    };
    build_runtime_invoke_request_with_operation(state, body, headers, model_id, operation)
}

fn build_runtime_invoke_request_with_operation(
    state: &AppState,
    mut body: Value,
    headers: &HeaderMap,
    model_id: &str,
    operation: RuntimeInvokeOperation,
) -> AppResult<RuntimeInvokeRequest> {
    if let Some(object) = body.as_object_mut() {
        object.remove("model");
        object.remove("stream");
        object
            .entry("anthropic_version")
            .or_insert_with(|| Value::String(BEDROCK_RUNTIME_ANTHROPIC_VERSION.into()));
    }
    let auth = select_mantle_auth(resolve_bedrock_auth(state)?, &state.bedrock_region);
    Ok(RuntimeInvokeRequest {
        url: operation.url(&state.bedrock_region, model_id),
        body,
        auth,
        headers: runtime_forward_headers(headers),
        stream: operation == RuntimeInvokeOperation::InvokeWithResponseStream,
    })
}

fn request_stream_enabled(body: &Value) -> bool {
    body.get("stream").and_then(Value::as_bool).unwrap_or(false)
}

pub async fn send_mantle_messages_request(
    client: &reqwest::Client,
    request: MantleMessagesRequest,
    retry_policy: MantleRetryPolicy,
) -> AppResult<UpstreamResponse> {
    let attempts = retry_policy.max_attempts.max(1);
    let started = Instant::now();
    let mut last_error = None;

    for attempt in 1..=attempts {
        match send_once(client, &request).await {
            Ok(response) if should_retry_status(response.status()) && attempt < attempts => {
                wait_before_retry(attempt).await;
                continue;
            }
            Ok(response)
                if is_profile_auth(&request.auth)
                    && should_retry_auth_status(response.status())
                    && attempt < attempts =>
            {
                wait_before_retry(attempt).await;
                continue;
            }
            Ok(response) => {
                return Ok(UpstreamResponse::from_reqwest(BEDROCK_PROVIDER, response)
                    .with_latency_ms(started.elapsed().as_millis()));
            }
            Err(MantleSendError::Reqwest(error))
                if should_retry_error(&error) && attempt < attempts =>
            {
                last_error = Some(error);
                wait_before_retry(attempt).await;
            }
            Err(MantleSendError::Reqwest(error)) => return Err(reqwest_error(error)),
            Err(MantleSendError::App(error)) => return Err(error),
        }
    }

    Err(reqwest_error(
        last_error.expect("retry loop must retain the last reqwest error"),
    ))
}

async fn send_once(
    client: &reqwest::Client,
    request: &MantleMessagesRequest,
) -> Result<reqwest::Response, MantleSendError> {
    let body = Bytes::from(serde_json::to_vec(&request.body)?);
    let mut headers = request.headers.clone();
    apply_mantle_auth_headers(request, &body, &mut headers).await?;

    client
        .post(&request.url)
        .headers(headers)
        .body(body)
        .send()
        .await
        .map_err(MantleSendError::Reqwest)
}

pub async fn send_runtime_invoke_request(
    client: &reqwest::Client,
    request: RuntimeInvokeRequest,
    retry_policy: MantleRetryPolicy,
) -> AppResult<UpstreamResponse> {
    let attempts = retry_policy.max_attempts.max(1);
    let started = Instant::now();
    let mut last_error = None;

    for attempt in 1..=attempts {
        match send_runtime_once(client, &request).await {
            Ok(response) if should_retry_status(response.status()) && attempt < attempts => {
                wait_before_retry(attempt).await;
                continue;
            }
            Ok(response)
                if is_profile_auth(&request.auth)
                    && should_retry_auth_status(response.status())
                    && attempt < attempts =>
            {
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
            Err(MantleSendError::Reqwest(error))
                if should_retry_error(&error) && attempt < attempts =>
            {
                last_error = Some(error);
                wait_before_retry(attempt).await;
            }
            Err(MantleSendError::Reqwest(error)) => return Err(reqwest_error(error)),
            Err(MantleSendError::App(error)) => return Err(error),
        }
    }

    Err(reqwest_error(
        last_error.expect("retry loop must retain the last reqwest error"),
    ))
}

async fn wait_before_retry(attempt: usize) {
    tokio::time::sleep(mantle_retry_delay(attempt, OsRng.next_u64())).await;
}

pub fn mantle_retry_delay(attempt: usize, jitter_seed: u64) -> Duration {
    let exponent = attempt.saturating_sub(1).min(10) as u32;
    let exponential_delay_ms = MANTLE_RETRY_BASE_DELAY_MS.saturating_mul(2_u64.pow(exponent));
    let capped_delay_ms = exponential_delay_ms.min(MANTLE_RETRY_MAX_DELAY_MS);
    let jitter_floor_ms = capped_delay_ms / 2;
    let jitter_range_ms = capped_delay_ms.saturating_sub(jitter_floor_ms).max(1);
    Duration::from_millis(jitter_floor_ms + (jitter_seed % (jitter_range_ms + 1)))
}

async fn send_runtime_once(
    client: &reqwest::Client,
    request: &RuntimeInvokeRequest,
) -> Result<reqwest::Response, MantleSendError> {
    let body = Bytes::from(serde_json::to_vec(&request.body)?);
    let mut headers = request.headers.clone();
    apply_runtime_auth_headers(request, &body, &mut headers).await?;

    client
        .post(&request.url)
        .headers(headers)
        .body(body)
        .send()
        .await
        .map_err(MantleSendError::Reqwest)
}

fn runtime_eventstream_response_from_reqwest(response: reqwest::Response) -> UpstreamResponse {
    let status = response.status();
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/event-stream"),
    );
    let mut decoder = AwsEventStreamDecoder::default();
    let stream = response
        .bytes_stream()
        .map(move |chunk| match chunk {
            Ok(bytes) => match decoder.push(bytes) {
                Ok(chunks) => chunks.into_iter().map(Ok).collect::<Vec<_>>(),
                Err(error) => vec![Err(error)],
            },
            Err(error) => vec![Err(AppError::Upstream(format!(
                "Bedrock Runtime stream failed: {error}"
            )))],
        })
        .flat_map(stream::iter);
    UpstreamResponse::stream(BEDROCK_PROVIDER, status, headers, stream)
}

#[derive(Default)]
struct AwsEventStreamDecoder {
    pending: Vec<u8>,
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
            if let Some(chunk) = runtime_event_stream_message_to_sse(&headers, payload)? {
                chunks.push(chunk);
            }
        }
        Ok(chunks)
    }
}

fn runtime_event_stream_message_to_sse(
    headers: &HashMap<String, String>,
    payload: &[u8],
) -> AppResult<Option<Bytes>> {
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

fn runtime_chunk_payload_to_sse(payload: &[u8]) -> AppResult<Option<Bytes>> {
    let payload = runtime_chunk_payload_bytes(payload)?;
    if payload.is_empty() {
        return Ok(None);
    }
    if payload.starts_with(b"data:") || payload.starts_with(b"event:") {
        return Ok(Some(Bytes::from(payload)));
    }
    let text = std::str::from_utf8(&payload).map_err(|error| {
        AppError::Upstream(format!("Bedrock Runtime chunk was not UTF-8: {error}"))
    })?;
    let text = text.trim_end_matches(['\r', '\n']);
    Ok(Some(Bytes::from(format!("data: {text}\n\n"))))
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

async fn apply_mantle_auth_headers(
    request: &MantleMessagesRequest,
    body: &Bytes,
    headers: &mut HeaderMap,
) -> AppResult<()> {
    match &request.auth {
        MantleAuthSelection::Header { name, value, .. } => {
            headers.insert(
                HeaderName::from_static(name),
                HeaderValue::from_str(value)
                    .map_err(|_| AppError::BadRequest(format!("invalid {name} header value")))?,
            );
            Ok(())
        }
        MantleAuthSelection::Profile {
            profile,
            region,
            service,
        } => {
            let sdk_config = aws_config::defaults(aws_config::BehaviorVersion::latest())
                .profile_name(profile.clone())
                .load()
                .await;
            let provider = sdk_config
                .credentials_provider()
                .ok_or(AppError::MissingCredential("AWS profile credentials"))?;
            let credentials = provider.provide_credentials().await.map_err(|err| {
                AppError::Upstream(format!(
                    "failed to load AWS credentials for profile {profile}: {err}"
                ))
            })?;
            let signed = sign_mantle_headers_for_credentials(
                request,
                body,
                headers,
                &credentials,
                SystemTime::now(),
                region,
                service,
            )?;
            *headers = signed;
            Ok(())
        }
    }
}

async fn apply_runtime_auth_headers(
    request: &RuntimeInvokeRequest,
    body: &Bytes,
    headers: &mut HeaderMap,
) -> AppResult<()> {
    match &request.auth {
        MantleAuthSelection::Header { value, .. } => {
            let value = format!("Bearer {value}");
            headers.insert(
                header::AUTHORIZATION,
                HeaderValue::from_str(&value).map_err(|_| {
                    AppError::BadRequest("invalid authorization header value".into())
                })?,
            );
            Ok(())
        }
        MantleAuthSelection::Profile {
            profile,
            region,
            service: _,
        } => {
            let sdk_config = aws_config::defaults(aws_config::BehaviorVersion::latest())
                .profile_name(profile.clone())
                .load()
                .await;
            let provider = sdk_config
                .credentials_provider()
                .ok_or(AppError::MissingCredential("AWS profile credentials"))?;
            let credentials = provider.provide_credentials().await.map_err(|err| {
                AppError::Upstream(format!(
                    "failed to load AWS credentials for profile {profile}: {err}"
                ))
            })?;
            let signed = sign_headers_for_credentials(
                request.url.as_str(),
                body,
                headers,
                &credentials,
                SystemTime::now(),
                region,
                BEDROCK_RUNTIME_SERVICE,
            )?;
            *headers = signed;
            Ok(())
        }
    }
}

pub fn sign_mantle_headers_for_credentials(
    request: &MantleMessagesRequest,
    body: &[u8],
    headers: &HeaderMap,
    credentials: &Credentials,
    time: SystemTime,
    region: &str,
    service: &str,
) -> AppResult<HeaderMap> {
    sign_headers_for_credentials(
        request.url.as_str(),
        body,
        headers,
        credentials,
        time,
        region,
        service,
    )
}

fn sign_headers_for_credentials(
    url: &str,
    body: &[u8],
    headers: &HeaderMap,
    credentials: &Credentials,
    time: SystemTime,
    region: &str,
    service: &str,
) -> AppResult<HeaderMap> {
    let mut signed_headers = headers.clone();
    let identity = credentials.clone().into();
    let signing_settings = SigningSettings::default();
    let signing_params = v4::SigningParams::builder()
        .identity(&identity)
        .region(region)
        .name(service)
        .time(time)
        .settings(signing_settings)
        .build()
        .map_err(|err| AppError::Upstream(format!("Bedrock SigV4 params: {err}")))?
        .into();
    let signable_headers = headers
        .iter()
        .filter_map(|(name, value)| {
            value
                .to_str()
                .ok()
                .map(|value| (name.as_str().to_string(), value.to_string()))
        })
        .collect::<Vec<_>>();
    let signable_request = SignableRequest::new(
        "POST",
        url,
        signable_headers
            .iter()
            .map(|(name, value)| (name.as_str(), value.as_str())),
        SignableBody::Bytes(body),
    )
    .map_err(|err| AppError::Upstream(format!("Bedrock SigV4 request: {err}")))?;
    let (instructions, _signature) = sign(signable_request, &signing_params)
        .map_err(|err| AppError::Upstream(format!("Bedrock SigV4 signing failed: {err}")))?
        .into_parts();
    for (name, value) in instructions.headers() {
        signed_headers.insert(
            HeaderName::from_bytes(name.as_bytes()).map_err(|err| {
                AppError::Upstream(format!("Bedrock SigV4 header name failed: {err}"))
            })?,
            HeaderValue::from_str(value).map_err(|err| {
                AppError::Upstream(format!("Bedrock SigV4 header value failed: {err}"))
            })?,
        );
    }
    Ok(signed_headers)
}

fn is_profile_auth(auth: &MantleAuthSelection) -> bool {
    matches!(auth, MantleAuthSelection::Profile { .. })
}

fn should_retry_auth_status(status: StatusCode) -> bool {
    matches!(status, StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN)
}

#[derive(Debug)]
enum MantleSendError {
    Reqwest(reqwest::Error),
    App(AppError),
}

impl From<AppError> for MantleSendError {
    fn from(error: AppError) -> Self {
        Self::App(error)
    }
}

impl From<serde_json::Error> for MantleSendError {
    fn from(error: serde_json::Error) -> Self {
        Self::App(AppError::Json(error))
    }
}

pub fn mantle_forward_headers(headers: &HeaderMap) -> HeaderMap {
    let mut forwarded = HeaderMap::new();
    forwarded.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/json"),
    );
    forwarded.insert(
        HeaderName::from_static("anthropic-version"),
        HeaderValue::from_static(DEFAULT_ANTHROPIC_VERSION),
    );

    for name in [
        header::ACCEPT,
        header::CONTENT_TYPE,
        HeaderName::from_static("anthropic-version"),
        HeaderName::from_static("anthropic-beta"),
    ] {
        if let Some(value) = headers.get(&name) {
            forwarded.insert(name, value.clone());
        }
    }
    forwarded
}

pub fn runtime_forward_headers(headers: &HeaderMap) -> HeaderMap {
    let mut forwarded = HeaderMap::new();
    forwarded.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/json"),
    );
    forwarded.insert(header::ACCEPT, HeaderValue::from_static("application/json"));
    for name in [header::ACCEPT, header::CONTENT_TYPE] {
        if let Some(value) = headers.get(&name) {
            forwarded.insert(name, value.clone());
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
