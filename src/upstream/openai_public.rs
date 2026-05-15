use axum::http::{header, HeaderMap, HeaderName, HeaderValue, Method, StatusCode};
use bytes::Bytes;
use serde_json::Value;

use crate::{
    auth::codex::{load_codex_auth, refresh_codex_auth},
    error::openai_error_body,
    AppError, AppResult, AppState, UpstreamResponse,
};

pub const OPENAI_PUBLIC_PROVIDER: &str = "openai";
pub const OPENAI_PUBLIC_BASE_URL: &str = "https://api.openai.com";
pub const MISSING_CODEX_AUTH_MESSAGE: &str =
    "Missing Codex OAuth credentials at ~/.codex/auth.json.";
pub const EXPIRED_AFTER_REFRESH_MESSAGE: &str =
    "Codex OAuth bearer token expired; refresh was attempted once and the request still requires a valid Codex login.";
pub const INVALID_AFTER_REFRESH_MESSAGE: &str =
    "Codex OAuth bearer token was invalid; refresh was attempted once and the request still requires a valid Codex login.";
pub const GENERIC_REJECTED_AFTER_REFRESH_MESSAGE: &str =
    "Codex OAuth token was rejected; refresh was attempted once and the request still requires a valid Codex login.";
pub const MISSING_SCOPE_MESSAGE: &str =
    "Codex OAuth token does not include the required scope for this route or model.";
pub const MODEL_PERMISSION_DENIED_MESSAGE: &str =
    "Codex OAuth token is not permitted to use the requested model.";
pub const ROUTE_PERMISSION_DENIED_MESSAGE: &str =
    "Codex OAuth token is not permitted to use this OpenAI route.";
pub const UPSTREAM_FORBIDDEN_MESSAGE: &str =
    "OpenAI rejected this Codex OAuth request as forbidden.";
pub const UNSUPPORTED_FEATURE_MESSAGE: &str =
    "This route family is not supported by the Codex OAuth public OpenAI facade.";
pub const UNSUPPORTED_ROUTE_MESSAGE: &str =
    "This OpenAI route is not implemented for the Codex OAuth public OpenAI facade.";

const SENSITIVE_AND_HOP_BY_HOP_HEADERS: &[&str] = &[
    "authorization",
    "cookie",
    "set-cookie",
    "x-api-key",
    "api-key",
    "openai-api-key",
    "x-openai-api-key",
    "x-goog-api-key",
    "proxy-authorization",
    "host",
    "content-length",
    "connection",
    "keep-alive",
    "proxy-authenticate",
    "te",
    "trailer",
    "transfer-encoding",
    "upgrade",
    "originator",
];

#[derive(Debug, Clone)]
pub struct PublicOpenAiRequest {
    pub url: String,
    pub headers: HeaderMap,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum UnsupportedOpenAiRouteKind {
    Feature,
    Route,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ClassifiedOpenAiError {
    pub status: StatusCode,
    pub error_type: &'static str,
    pub code: &'static str,
    pub message: &'static str,
    pub refresh_eligible: bool,
}

pub fn build_public_openai_request(
    state: &AppState,
    url: impl Into<String>,
    inbound_headers: &HeaderMap,
) -> AppResult<PublicOpenAiRequest> {
    let auth = load_codex_auth(state)?;
    build_public_openai_request_with_bearer(url, inbound_headers, &auth.access_token)
}

pub fn require_public_openai_auth(state: &AppState) -> AppResult<()> {
    load_codex_auth(state).map(|_| ())
}

pub fn build_public_openai_request_with_bearer(
    url: impl Into<String>,
    inbound_headers: &HeaderMap,
    access_token: &str,
) -> AppResult<PublicOpenAiRequest> {
    let mut headers = public_openai_forward_headers(inbound_headers);
    headers.insert(
        header::AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {access_token}"))
            .map_err(|_| AppError::BadRequest("invalid Codex access token".into()))?,
    );
    Ok(PublicOpenAiRequest {
        url: url.into(),
        headers,
    })
}

pub fn public_openai_forward_headers(inbound_headers: &HeaderMap) -> HeaderMap {
    let mut headers = HeaderMap::new();
    for (name, value) in inbound_headers {
        if should_forward_public_openai_header(name) {
            headers.append(name.clone(), value.clone());
        }
    }
    headers
}

pub fn unsupported_openai_route_response(kind: UnsupportedOpenAiRouteKind) -> UpstreamResponse {
    let (code, message) = match kind {
        UnsupportedOpenAiRouteKind::Feature => ("unsupported_feature", UNSUPPORTED_FEATURE_MESSAGE),
        UnsupportedOpenAiRouteKind::Route => ("unsupported_route", UNSUPPORTED_ROUTE_MESSAGE),
    };
    openai_error_response(
        StatusCode::NOT_IMPLEMENTED,
        "invalid_request_error",
        code,
        message,
    )
}

pub fn missing_codex_auth_response() -> UpstreamResponse {
    openai_error_response(
        StatusCode::UNAUTHORIZED,
        "authentication_error",
        "invalid_api_key",
        MISSING_CODEX_AUTH_MESSAGE,
    )
}

pub async fn send_public_openai_http_with_refresh(
    state: &AppState,
    method: Method,
    url: impl Into<String>,
    inbound_headers: HeaderMap,
    body: Bytes,
) -> AppResult<UpstreamResponse> {
    let url = url.into();
    let first = match send_public_openai_http_once(
        state,
        method.clone(),
        &url,
        &inbound_headers,
        body.clone(),
    )
    .await
    {
        Ok(response) => response,
        Err(error) if is_missing_codex_auth_error(&error) => {
            return Ok(missing_codex_auth_response())
        }
        Err(error) => return Err(error),
    };

    let first_status = first.status();
    let first_headers = specter_headers_to_http(first.headers())?;
    let first_body = first.into_body();
    if !is_refresh_eligible_401(first_status, &first_body, false) {
        return Ok(classify_or_passthrough_response(
            first_status,
            first_headers,
            first_body,
            false,
        ));
    }

    if refresh_codex_auth(state).await.is_err() {
        return Ok(classified_openai_error_response(
            classify_openai_auth_or_permission_error(first_status, &first_body, true),
        ));
    }

    let second = send_public_openai_http_once(state, method, &url, &inbound_headers, body).await?;
    let second_status = second.status();
    let second_headers = specter_headers_to_http(second.headers())?;
    let second_body = second.into_body();
    Ok(classify_or_passthrough_response(
        second_status,
        second_headers,
        second_body,
        true,
    ))
}

pub fn classify_openai_auth_or_permission_error(
    status: StatusCode,
    body: &[u8],
    refresh_attempted: bool,
) -> Option<ClassifiedOpenAiError> {
    match status {
        StatusCode::UNAUTHORIZED => Some(classify_unauthorized(body, refresh_attempted)),
        StatusCode::FORBIDDEN => Some(classify_forbidden(body)),
        _ => None,
    }
}

pub fn is_refresh_eligible_401(status: StatusCode, body: &[u8], refresh_attempted: bool) -> bool {
    classify_openai_auth_or_permission_error(status, body, refresh_attempted)
        .is_some_and(|classification| classification.refresh_eligible)
}

fn should_forward_public_openai_header(name: &HeaderName) -> bool {
    let lower = name.as_str().to_ascii_lowercase();
    if SENSITIVE_AND_HOP_BY_HOP_HEADERS.contains(&lower.as_str()) {
        return false;
    }
    !is_public_openai_sensitive_header(&lower)
}

fn is_public_openai_sensitive_header(lower: &str) -> bool {
    lower.starts_with("openai-")
        || lower.starts_with("chatgpt-")
        || lower
            .split(|byte: char| !byte.is_ascii_alphanumeric())
            .any(|part| {
                matches!(
                    part,
                    "account" | "session" | "user" | "org" | "organization" | "project"
                )
            })
}

async fn send_public_openai_http_once(
    state: &AppState,
    method: Method,
    url: &str,
    inbound_headers: &HeaderMap,
    body: Bytes,
) -> AppResult<specter::Response> {
    let request = build_public_openai_request(state, url, inbound_headers)?;
    state
        .specter
        .request(method, request.url)
        .headers(specter::Headers::from(request.headers))
        .body(body)
        .send()
        .await
        .map_err(|error| AppError::Upstream(format!("OpenAI public transport: {error}")))
}

fn classify_or_passthrough_response(
    status: StatusCode,
    headers: HeaderMap,
    body: Bytes,
    refresh_attempted: bool,
) -> UpstreamResponse {
    if let Some(classification) =
        classify_openai_auth_or_permission_error(status, &body, refresh_attempted)
    {
        return classified_openai_error_response(Some(classification));
    }
    UpstreamResponse::bytes(OPENAI_PUBLIC_PROVIDER, status, headers, body)
}

fn classify_unauthorized(body: &[u8], refresh_attempted: bool) -> ClassifiedOpenAiError {
    let (message, refresh_eligible) = if refresh_attempted {
        (
            if upstream_error_text(body).contains("expired") {
                EXPIRED_AFTER_REFRESH_MESSAGE
            } else if upstream_error_text(body).contains("invalid") {
                INVALID_AFTER_REFRESH_MESSAGE
            } else {
                GENERIC_REJECTED_AFTER_REFRESH_MESSAGE
            },
            false,
        )
    } else {
        (GENERIC_REJECTED_AFTER_REFRESH_MESSAGE, true)
    };
    ClassifiedOpenAiError {
        status: StatusCode::UNAUTHORIZED,
        error_type: "authentication_error",
        code: "invalid_api_key",
        message,
        refresh_eligible,
    }
}

fn classify_forbidden(body: &[u8]) -> ClassifiedOpenAiError {
    let text = upstream_error_text(body);
    let (code, message) = if text.contains("missing_scope") || text.contains("missing scope") {
        ("missing_scope", MISSING_SCOPE_MESSAGE)
    } else if text.contains("model_permission_denied")
        || ((text.contains("model") || text.contains("gpt-")) && permission_denied_text(&text))
    {
        ("model_permission_denied", MODEL_PERMISSION_DENIED_MESSAGE)
    } else if text.contains("route_permission_denied")
        || text.contains("endpoint_permission_denied")
        || ((text.contains("route") || text.contains("endpoint") || text.contains("path"))
            && permission_denied_text(&text))
    {
        ("route_permission_denied", ROUTE_PERMISSION_DENIED_MESSAGE)
    } else {
        ("upstream_forbidden", UPSTREAM_FORBIDDEN_MESSAGE)
    };
    ClassifiedOpenAiError {
        status: StatusCode::FORBIDDEN,
        error_type: "permission_error",
        code,
        message,
        refresh_eligible: false,
    }
}

fn permission_denied_text(text: &str) -> bool {
    text.contains("not permitted")
        || text.contains("permission")
        || text.contains("forbidden")
        || text.contains("denied")
}

fn upstream_error_text(body: &[u8]) -> String {
    let parsed = serde_json::from_slice::<Value>(body).ok();
    let mut parts = Vec::new();
    if let Some(error) = parsed.as_ref().and_then(|value| value.get("error")) {
        for key in ["code", "type", "message"] {
            if let Some(value) = error.get(key).and_then(Value::as_str) {
                parts.push(value);
            }
        }
    }
    if parts.is_empty() {
        parts.push(std::str::from_utf8(body).unwrap_or_default());
    }
    parts.join(" ").to_ascii_lowercase()
}

fn classified_openai_error_response(
    classification: Option<ClassifiedOpenAiError>,
) -> UpstreamResponse {
    let classification = classification.expect("classification is required");
    openai_error_response(
        classification.status,
        classification.error_type,
        classification.code,
        classification.message,
    )
}

fn openai_error_response(
    status: StatusCode,
    error_type: &'static str,
    code: &'static str,
    message: &'static str,
) -> UpstreamResponse {
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/json"),
    );
    UpstreamResponse::bytes(
        OPENAI_PUBLIC_PROVIDER,
        status,
        headers,
        Bytes::from(
            serde_json::to_vec(&openai_error_body(message, error_type, None, Some(code)))
                .expect("OpenAI error envelope serializes"),
        ),
    )
}

fn is_missing_codex_auth_error(error: &AppError) -> bool {
    matches!(error, AppError::MissingCredential("~/.codex/auth.json"))
}

fn specter_headers_to_http(headers: &specter::Headers) -> AppResult<HeaderMap> {
    let mut out = HeaderMap::new();
    for (name, value) in headers.iter() {
        let name = HeaderName::from_bytes(name.as_bytes())
            .map_err(|err| AppError::Upstream(format!("invalid upstream header name: {err}")))?;
        let value = HeaderValue::from_str(value)
            .map_err(|err| AppError::Upstream(format!("invalid upstream header value: {err}")))?;
        out.append(name, value);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn header_value(headers: &HeaderMap, name: &str) -> Option<String> {
        headers
            .get(name)
            .and_then(|value| value.to_str().ok())
            .map(ToOwned::to_owned)
    }

    #[test]
    fn public_openai_headers_use_codex_bearer_without_responses_headers() {
        let mut inbound = HeaderMap::new();
        inbound.insert(
            header::AUTHORIZATION,
            HeaderValue::from_static("Bearer caller"),
        );
        inbound.insert(header::COOKIE, HeaderValue::from_static("session=secret"));
        inbound.insert("x-api-key", HeaderValue::from_static("secret"));
        inbound.insert(
            "OpenAI-Beta",
            HeaderValue::from_static("responses_websockets=2026-02-06"),
        );
        inbound.insert("originator", HeaderValue::from_static("codex_cli_rs"));
        inbound.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );

        let request = build_public_openai_request_with_bearer(
            "https://api.openai.com/v1/realtime/transcription_sessions",
            &inbound,
            "codex-access",
        )
        .unwrap();

        assert_eq!(
            header_value(&request.headers, "authorization").as_deref(),
            Some("Bearer codex-access")
        );
        assert_eq!(
            header_value(&request.headers, "content-type").as_deref(),
            Some("application/json")
        );
        assert!(!request.headers.contains_key("cookie"));
        assert!(!request.headers.contains_key("x-api-key"));
        assert!(!request.headers.contains_key("OpenAI-Beta"));
        assert!(!request.headers.contains_key("originator"));
    }

    #[test]
    fn public_openai_forward_headers_strip_sensitive_api_and_hop_by_hop_headers() {
        let mut inbound = HeaderMap::new();
        for name in [
            "authorization",
            "cookie",
            "ChatGPT-Account-Id",
            "x-api-key",
            "api-key",
            "openai-beta",
            "openai-api-key",
            "OpenAI-Organization",
            "OpenAI-Project",
            "OpenAI-Session-Id",
            "OpenAI-User",
            "OpenAI-Arbitrary-Sensitive",
            "x-account-id",
            "x-session-id",
            "x-user-id",
            "x-org-id",
            "x-project-id",
            "x-openai-api-key",
            "connection",
            "keep-alive",
            "proxy-authorization",
            "te",
            "trailer",
            "transfer-encoding",
            "upgrade",
            "host",
            "content-length",
        ] {
            inbound.insert(
                HeaderName::from_bytes(name.as_bytes()).unwrap(),
                HeaderValue::from_static("blocked"),
            );
        }
        inbound.insert("x-request-id", HeaderValue::from_static("req-123"));
        inbound.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("multipart/form-data; boundary=x"),
        );

        let headers = public_openai_forward_headers(&inbound);

        assert_eq!(
            header_value(&headers, "x-request-id").as_deref(),
            Some("req-123")
        );
        assert_eq!(
            header_value(&headers, "content-type").as_deref(),
            Some("multipart/form-data; boundary=x")
        );
        for name in [
            "authorization",
            "cookie",
            "ChatGPT-Account-Id",
            "x-api-key",
            "api-key",
            "openai-beta",
            "openai-api-key",
            "OpenAI-Organization",
            "OpenAI-Project",
            "OpenAI-Session-Id",
            "OpenAI-User",
            "OpenAI-Arbitrary-Sensitive",
            "x-account-id",
            "x-session-id",
            "x-user-id",
            "x-org-id",
            "x-project-id",
            "x-openai-api-key",
            "connection",
            "keep-alive",
            "proxy-authorization",
            "te",
            "trailer",
            "transfer-encoding",
            "upgrade",
            "host",
            "content-length",
        ] {
            assert!(!headers.contains_key(name), "{name} leaked upstream");
        }
    }

    #[test]
    fn classify_public_openai_auth_and_permission_errors() {
        let expired = json!({"error": {"message": "token expired"}});
        let expired = classify_openai_auth_or_permission_error(
            StatusCode::UNAUTHORIZED,
            serde_json::to_vec(&expired).unwrap().as_slice(),
            true,
        )
        .unwrap();
        assert_eq!(expired.status, StatusCode::UNAUTHORIZED);
        assert_eq!(expired.error_type, "authentication_error");
        assert_eq!(expired.code, "invalid_api_key");
        assert_eq!(expired.message, EXPIRED_AFTER_REFRESH_MESSAGE);
        assert!(!expired.refresh_eligible);

        let missing_scope = json!({"error": {"code": "missing_scope"}});
        let missing_scope = classify_openai_auth_or_permission_error(
            StatusCode::FORBIDDEN,
            serde_json::to_vec(&missing_scope).unwrap().as_slice(),
            false,
        )
        .unwrap();
        assert_eq!(missing_scope.status, StatusCode::FORBIDDEN);
        assert_eq!(missing_scope.error_type, "permission_error");
        assert_eq!(missing_scope.code, "missing_scope");
        assert_eq!(missing_scope.message, MISSING_SCOPE_MESSAGE);
        assert!(!missing_scope.refresh_eligible);

        let model_denied = json!({"error": {"message": "model gpt-realtime-2 not permitted"}});
        let model_denied = classify_openai_auth_or_permission_error(
            StatusCode::FORBIDDEN,
            serde_json::to_vec(&model_denied).unwrap().as_slice(),
            false,
        )
        .unwrap();
        assert_eq!(model_denied.code, "model_permission_denied");
        assert_eq!(model_denied.message, MODEL_PERMISSION_DENIED_MESSAGE);

        let route_denied = json!({"error": {"message": "route /v1/audio/speech not permitted"}});
        let route_denied = classify_openai_auth_or_permission_error(
            StatusCode::FORBIDDEN,
            serde_json::to_vec(&route_denied).unwrap().as_slice(),
            false,
        )
        .unwrap();
        assert_eq!(route_denied.code, "route_permission_denied");
        assert_eq!(route_denied.message, ROUTE_PERMISSION_DENIED_MESSAGE);
    }

    #[test]
    fn refresh_policy_only_allows_first_eligible_401() {
        let generic = json!({"error": {"message": "unauthorized"}});
        let body = serde_json::to_vec(&generic).unwrap();

        assert!(is_refresh_eligible_401(
            StatusCode::UNAUTHORIZED,
            &body,
            false
        ));
        assert!(!is_refresh_eligible_401(
            StatusCode::UNAUTHORIZED,
            &body,
            true
        ));
        assert!(!is_refresh_eligible_401(
            StatusCode::FORBIDDEN,
            &body,
            false
        ));
    }
}
