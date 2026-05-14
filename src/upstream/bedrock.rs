use std::time::{Instant, SystemTime};

use aws_credential_types::{provider::ProvideCredentials, Credentials};
use aws_sigv4::{
    http_request::{sign, SignableBody, SignableRequest, SigningSettings},
    sign::v4,
};
use axum::{
    body::to_bytes,
    http::{header, HeaderMap, HeaderName, HeaderValue, StatusCode},
};
use bytes::Bytes;
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
        Self { max_attempts: 2 }
    }
}

pub fn mantle_base_url(region: &str) -> String {
    format!("https://bedrock-mantle.{region}.api.aws")
}

pub fn mantle_messages_url(region: &str) -> String {
    format!("{}{}", mantle_base_url(region), MANTLE_MESSAGES_PATH)
}

pub fn runtime_invoke_url(region: &str, model_id: &str) -> String {
    format!(
        "https://bedrock-runtime.{region}.amazonaws.com/model/{}/invoke",
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
}

pub fn build_runtime_invoke_request(
    state: &AppState,
    mut body: Value,
    headers: &HeaderMap,
    model_id: &str,
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
        url: runtime_invoke_url(&state.bedrock_region, model_id),
        body,
        auth,
        headers: runtime_forward_headers(headers),
    })
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
                continue;
            }
            Ok(response)
                if is_profile_auth(&request.auth)
                    && should_retry_auth_status(response.status())
                    && attempt < attempts =>
            {
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
                continue;
            }
            Ok(response)
                if is_profile_auth(&request.auth)
                    && should_retry_auth_status(response.status())
                    && attempt < attempts =>
            {
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
            }
            Err(MantleSendError::Reqwest(error)) => return Err(reqwest_error(error)),
            Err(MantleSendError::App(error)) => return Err(error),
        }
    }

    Err(reqwest_error(
        last_error.expect("retry loop must retain the last reqwest error"),
    ))
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
