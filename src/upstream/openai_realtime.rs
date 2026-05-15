use axum::http::{HeaderMap, Method};
use bytes::Bytes;

use crate::{
    upstream::openai_public::{
        build_public_openai_request, send_public_openai_http_with_refresh, PublicOpenAiRequest,
        OPENAI_PUBLIC_BASE_URL,
    },
    AppResult, AppState, UpstreamResponse,
};

pub const REALTIME_WS_PATH: &str = "/v1/realtime";
pub const REALTIME_TRANSCRIPTION_SESSIONS_PATH: &str = "/v1/realtime/transcription_sessions";

pub fn realtime_ws_url(base_url: &str, model: &str) -> String {
    let base = base_url
        .trim_end_matches('/')
        .replacen("https://", "wss://", 1)
        .replacen("http://", "ws://", 1);
    format!("{base}{REALTIME_WS_PATH}?model={model}")
}

pub fn realtime_transcription_sessions_url(base_url: &str) -> String {
    format!(
        "{}{}",
        base_url.trim_end_matches('/'),
        REALTIME_TRANSCRIPTION_SESSIONS_PATH
    )
}

pub fn build_realtime_transcription_sessions_request(
    state: &AppState,
    inbound_headers: &HeaderMap,
) -> AppResult<PublicOpenAiRequest> {
    build_public_openai_request(
        state,
        realtime_transcription_sessions_url(OPENAI_PUBLIC_BASE_URL),
        inbound_headers,
    )
}

pub fn build_realtime_ws_request(
    state: &AppState,
    inbound_headers: &HeaderMap,
    model: &str,
) -> AppResult<PublicOpenAiRequest> {
    build_public_openai_request(
        state,
        realtime_ws_url(OPENAI_PUBLIC_BASE_URL, model),
        inbound_headers,
    )
}

pub async fn forward_realtime_transcription_session(
    state: &AppState,
    inbound_headers: HeaderMap,
    body: Bytes,
) -> AppResult<UpstreamResponse> {
    send_public_openai_http_with_refresh(
        state,
        Method::POST,
        realtime_transcription_sessions_url(OPENAI_PUBLIC_BASE_URL),
        inbound_headers,
        body,
    )
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::{header, HeaderValue};

    #[test]
    fn realtime_urls_target_public_openai_surfaces() {
        assert_eq!(
            realtime_transcription_sessions_url("https://api.openai.com/"),
            "https://api.openai.com/v1/realtime/transcription_sessions"
        );
        assert_eq!(
            realtime_ws_url("https://api.openai.com", "gpt-realtime-2"),
            "wss://api.openai.com/v1/realtime?model=gpt-realtime-2"
        );
    }

    #[test]
    fn realtime_request_uses_codex_bearer_and_omits_responses_beta_originator() {
        let codex_home = tempfile::tempdir().unwrap();
        let auth_home = tempfile::tempdir().unwrap();
        std::fs::write(
            codex_home.path().join("auth.json"),
            r#"{"tokens":{"access_token":"codex-access","account_id":"account-123"}}"#,
        )
        .unwrap();
        let state = AppState::for_tests(
            codex_home.path().to_path_buf(),
            auth_home.path().to_path_buf(),
        );
        let mut headers = HeaderMap::new();
        headers.insert(
            header::AUTHORIZATION,
            HeaderValue::from_static("Bearer caller"),
        );
        headers.insert(
            "OpenAI-Beta",
            HeaderValue::from_static("responses_websockets=2026-02-06"),
        );
        headers.insert("originator", HeaderValue::from_static("codex_cli_rs"));
        headers.insert("x-request-id", HeaderValue::from_static("req-123"));

        let request = build_realtime_transcription_sessions_request(&state, &headers).unwrap();

        assert_eq!(
            request
                .headers
                .get(header::AUTHORIZATION)
                .and_then(|value| value.to_str().ok()),
            Some("Bearer codex-access")
        );
        assert_eq!(
            request
                .headers
                .get("x-request-id")
                .and_then(|value| value.to_str().ok()),
            Some("req-123")
        );
        assert!(!request.headers.contains_key("OpenAI-Beta"));
        assert!(!request.headers.contains_key("originator"));
        assert!(!request.headers.contains_key("ChatGPT-Account-Id"));
    }
}
