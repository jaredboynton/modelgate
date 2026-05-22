use bytes::Bytes;
use http::{header, HeaderMap, HeaderName, HeaderValue, Method};
use serde_json::{json, Value};

use crate::{
    auth::google::api_key, upstream::bedrock, AppError, AppResult, AppState, UpstreamResponse,
};

const GOOGLE_PROVIDER: &str = "google";
const BEDROCK_FALLBACK_PROVIDER: &str = "bedrock";
const DEFAULT_BEDROCK_FALLBACK_MODEL: &str = "anthropic/claude-haiku-4-5";

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct GoogleRequest {
    pub url: String,
    pub headers: HeaderMap,
}

pub fn rewrite_google_path(path: &str) -> AppResult<String> {
    let prefix = "/api/provider/google/";
    let stripped = path
        .strip_prefix(prefix)
        .ok_or_else(|| AppError::BadRequest("invalid Google provider path".into()))?;
    let rewritten = stripped.replace("v1beta1/publishers/google/models/", "v1beta/models/");
    Ok(format!(
        "https://generativelanguage.googleapis.com/{rewritten}"
    ))
}

pub fn build_google_request(state: &AppState, path: &str) -> AppResult<GoogleRequest> {
    let key = api_key(state)?;
    build_google_request_with_headers(state, path, HeaderMap::new(), key)
}

pub fn build_google_request_with_headers(
    _state: &AppState,
    path: &str,
    headers: HeaderMap,
    key: String,
) -> AppResult<GoogleRequest> {
    build_google_request_for_url(rewrite_google_path(path)?, headers, key)
}

pub fn build_google_generate_content_request_with_headers(
    upstream_model: &str,
    headers: HeaderMap,
    key: String,
) -> AppResult<GoogleRequest> {
    build_google_generate_content_request_for_action(
        "https://generativelanguage.googleapis.com",
        upstream_model,
        headers,
        key,
        false,
    )
}

pub fn build_google_stream_generate_content_request_with_headers(
    upstream_model: &str,
    headers: HeaderMap,
    key: String,
) -> AppResult<GoogleRequest> {
    build_google_generate_content_request_for_action(
        "https://generativelanguage.googleapis.com",
        upstream_model,
        headers,
        key,
        true,
    )
}

pub fn build_google_generate_content_request_with_base_url(
    base_url: &str,
    upstream_model: &str,
    headers: HeaderMap,
    key: String,
) -> AppResult<GoogleRequest> {
    build_google_generate_content_request_for_action(base_url, upstream_model, headers, key, false)
}

pub fn build_google_stream_generate_content_request_with_base_url(
    base_url: &str,
    upstream_model: &str,
    headers: HeaderMap,
    key: String,
) -> AppResult<GoogleRequest> {
    build_google_generate_content_request_for_action(base_url, upstream_model, headers, key, true)
}

fn build_google_generate_content_request_for_action(
    base_url: &str,
    upstream_model: &str,
    headers: HeaderMap,
    key: String,
    stream: bool,
) -> AppResult<GoogleRequest> {
    if upstream_model.trim().is_empty() || upstream_model.contains('/') {
        return Err(AppError::BadRequest("invalid Google upstream model".into()));
    }
    let base_url = base_url.trim_end_matches('/');
    let action = if stream {
        "streamGenerateContent?alt=sse"
    } else {
        "generateContent"
    };
    build_google_request_for_url(
        format!("{base_url}/v1beta/models/{upstream_model}:{action}"),
        headers,
        key,
    )
}

fn build_google_request_for_url(
    url: String,
    headers: HeaderMap,
    key: String,
) -> AppResult<GoogleRequest> {
    let mut outbound_headers = google_passthrough_headers(&headers);
    outbound_headers.insert(
        HeaderName::from_static("x-goog-api-key"),
        HeaderValue::from_str(&key)
            .map_err(|_| AppError::BadRequest("invalid GOOGLE_API_KEY header value".into()))?,
    );
    Ok(GoogleRequest {
        url,
        headers: outbound_headers,
    })
}

pub fn translate_google_to_bedrock_messages(body: Value, fallback_model: &str) -> AppResult<Value> {
    let contents = body
        .get("contents")
        .and_then(Value::as_array)
        .ok_or_else(|| AppError::BadRequest("missing Google contents".into()))?;
    let max_tokens = body
        .pointer("/generationConfig/maxOutputTokens")
        .and_then(Value::as_u64)
        .unwrap_or(1024);

    let messages = contents
        .iter()
        .map(google_content_to_bedrock_message)
        .collect::<AppResult<Vec<_>>>()?;

    let mut translated = json!({
        "model": fallback_model,
        "max_tokens": max_tokens,
        "messages": messages,
    });
    if let Some(system) = system_instruction_text(&body) {
        translated["system"] = Value::String(system);
    }
    Ok(translated)
}

pub fn translate_bedrock_messages_to_google_response(body: Value) -> AppResult<Value> {
    let parts = body
        .get("content")
        .and_then(Value::as_array)
        .map(|content| {
            content
                .iter()
                .filter_map(|part| part.get("text").and_then(Value::as_str))
                .map(|text| json!({ "text": text }))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let finish_reason = body
        .get("stop_reason")
        .and_then(Value::as_str)
        .map(google_finish_reason)
        .unwrap_or("STOP");

    let mut translated = json!({
        "candidates": [{
            "content": {
                "role": "model",
                "parts": parts,
            },
            "finishReason": finish_reason,
        }]
    });

    if let Some(usage) = translate_bedrock_usage(body.get("usage")) {
        translated["usageMetadata"] = usage;
    }

    Ok(translated)
}

fn google_content_to_bedrock_message(content: &Value) -> AppResult<Value> {
    let role = match content.get("role").and_then(Value::as_str) {
        Some("model") => "assistant",
        _ => "user",
    };
    let parts = content
        .get("parts")
        .and_then(Value::as_array)
        .ok_or_else(|| AppError::BadRequest("Google content missing parts".into()))?;
    let text_parts = parts
        .iter()
        .filter_map(|part| part.get("text").and_then(Value::as_str))
        .map(|text| json!({ "type": "text", "text": text }))
        .collect::<Vec<_>>();

    Ok(json!({
        "role": role,
        "content": text_parts,
    }))
}

fn system_instruction_text(body: &Value) -> Option<String> {
    body.pointer("/systemInstruction/parts")
        .and_then(Value::as_array)
        .map(|parts| {
            parts
                .iter()
                .filter_map(|part| part.get("text").and_then(Value::as_str))
                .collect::<Vec<_>>()
                .join("\n")
        })
        .filter(|text| !text.is_empty())
}

pub async fn forward_google(
    state: &AppState,
    method: Method,
    path: &str,
    headers: HeaderMap,
    body: Bytes,
) -> AppResult<UpstreamResponse> {
    let key = api_key(state)?;
    let request = build_google_request_with_headers(state, path, headers, key)?;

    let direct =
        send_google_direct(state, method, request, body.clone(), is_sse_request(path)).await?;
    if !direct.status.is_client_error() && !direct.status.is_server_error() {
        return Ok(direct);
    }

    match fallback_to_bedrock(state, body).await {
        Ok(fallback) => Ok(fallback),
        Err(_) => Ok(direct),
    }
}

pub async fn send_google_direct(
    state: &AppState,
    method: Method,
    request: GoogleRequest,
    body: Bytes,
    stream_response: bool,
) -> AppResult<UpstreamResponse> {
    let response = state
        .http
        .request(method, &request.url)
        .headers(request.headers)
        .body(body)
        .send()
        .await
        .map_err(|err| {
            let kind = if stream_response {
                "streaming transport"
            } else {
                "transport"
            };
            AppError::Upstream(format!("Google {kind}: {err}"))
        })?;
    Ok(UpstreamResponse::from_reqwest(GOOGLE_PROVIDER, response))
}

pub async fn forward_generate_content_direct_response(
    state: &AppState,
    upstream_model: &str,
    headers: HeaderMap,
    body: Bytes,
) -> AppResult<UpstreamResponse> {
    let key = api_key(state)?;
    let request = build_google_generate_content_request_with_base_url(
        &state.runtime.google_generate_base_url,
        upstream_model,
        headers,
        key,
    )?;
    forward_generate_content_direct_request(state, request, body).await
}

pub async fn forward_generate_content_direct_request(
    state: &AppState,
    request: GoogleRequest,
    body: Bytes,
) -> AppResult<UpstreamResponse> {
    send_google_direct(state, Method::POST, request, body, false).await
}

pub async fn forward_stream_generate_content_direct_response(
    state: &AppState,
    upstream_model: &str,
    headers: HeaderMap,
    body: Bytes,
) -> AppResult<UpstreamResponse> {
    let key = api_key(state)?;
    let request = build_google_stream_generate_content_request_with_base_url(
        &state.runtime.google_generate_base_url,
        upstream_model,
        headers,
        key,
    )?;
    forward_stream_generate_content_direct_request(state, request, body).await
}

pub async fn forward_stream_generate_content_direct_request(
    state: &AppState,
    request: GoogleRequest,
    body: Bytes,
) -> AppResult<UpstreamResponse> {
    send_google_direct(state, Method::POST, request, body, true).await
}

async fn fallback_to_bedrock(state: &AppState, google_body: Bytes) -> AppResult<UpstreamResponse> {
    let google_body: Value = serde_json::from_slice(&google_body)?;
    let bedrock_body =
        translate_google_to_bedrock_messages(google_body, DEFAULT_BEDROCK_FALLBACK_MODEL)?;
    let response = bedrock::forward_messages(state, bedrock_body, HeaderMap::new()).await?;
    let translated =
        translate_bedrock_messages_to_google_response(serde_json::from_slice(&response)?)?;
    UpstreamResponse::json(BEDROCK_FALLBACK_PROVIDER, translated).map_err(AppError::from)
}

fn google_passthrough_headers(headers: &HeaderMap) -> HeaderMap {
    let mut passthrough = HeaderMap::new();
    for (name, value) in headers {
        if should_forward_google_header(name) {
            passthrough.append(name.clone(), value.clone());
        }
    }
    if !passthrough.contains_key(header::CONTENT_TYPE) {
        passthrough.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );
    }
    passthrough
}

fn should_forward_google_header(name: &HeaderName) -> bool {
    !matches!(
        name.as_str().to_ascii_lowercase().as_str(),
        "host"
            | "content-length"
            | "connection"
            | "keep-alive"
            | "proxy-authenticate"
            | "proxy-authorization"
            | "te"
            | "trailer"
            | "transfer-encoding"
            | "upgrade"
            | "authorization"
            | "cookie"
            | "x-goog-api-key"
    )
}

fn is_sse_request(path: &str) -> bool {
    path.contains("alt=sse") || path.contains(":streamGenerateContent")
}

fn google_finish_reason(stop_reason: &str) -> &'static str {
    match stop_reason {
        "max_tokens" => "MAX_TOKENS",
        "stop_sequence" | "end_turn" => "STOP",
        _ => "OTHER",
    }
}

fn translate_bedrock_usage(usage: Option<&Value>) -> Option<Value> {
    let usage = usage?;
    let mut translated = json!({});
    if let Some(input_tokens) = usage.get("input_tokens").and_then(Value::as_u64) {
        translated["promptTokenCount"] = Value::from(input_tokens);
    }
    if let Some(output_tokens) = usage.get("output_tokens").and_then(Value::as_u64) {
        translated["candidatesTokenCount"] = Value::from(output_tokens);
    }
    if translated
        .as_object()
        .is_some_and(|object| object.is_empty())
    {
        None
    } else {
        Some(translated)
    }
}
