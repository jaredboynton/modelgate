use axum::http::{HeaderMap, Method};
use bytes::Bytes;

use crate::{
    upstream::openai_public::{
        build_public_openai_request, send_public_openai_http_with_refresh, PublicOpenAiRequest,
        OPENAI_PUBLIC_BASE_URL,
    },
    AppResult, AppState, UpstreamResponse,
};

pub const AUDIO_TRANSCRIPTIONS_PATH: &str = "/v1/audio/transcriptions";

pub fn audio_transcriptions_url(base_url: &str) -> String {
    format!(
        "{}{}",
        base_url.trim_end_matches('/'),
        AUDIO_TRANSCRIPTIONS_PATH
    )
}

pub fn build_audio_transcriptions_request(
    state: &AppState,
    inbound_headers: &HeaderMap,
) -> AppResult<PublicOpenAiRequest> {
    build_public_openai_request(
        state,
        audio_transcriptions_url(OPENAI_PUBLIC_BASE_URL),
        inbound_headers,
    )
}

pub async fn forward_audio_transcriptions(
    state: &AppState,
    inbound_headers: HeaderMap,
    body: Bytes,
) -> AppResult<UpstreamResponse> {
    send_public_openai_http_with_refresh(
        state,
        Method::POST,
        audio_transcriptions_url(OPENAI_PUBLIC_BASE_URL),
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
    fn audio_transcriptions_url_targets_public_openai_audio_api() {
        assert_eq!(
            audio_transcriptions_url("https://api.openai.com/"),
            "https://api.openai.com/v1/audio/transcriptions"
        );
    }

    #[test]
    fn audio_transcriptions_request_preserves_multipart_content_type_only() {
        let codex_home = tempfile::tempdir().unwrap();
        let auth_home = tempfile::tempdir().unwrap();
        std::fs::write(
            codex_home.path().join("auth.json"),
            r#"{"tokens":{"access_token":"codex-access"}}"#,
        )
        .unwrap();
        let state = AppState::for_tests(
            codex_home.path().to_path_buf(),
            auth_home.path().to_path_buf(),
        );
        let mut headers = HeaderMap::new();
        headers.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("multipart/form-data; boundary=audio"),
        );
        headers.insert(
            header::AUTHORIZATION,
            HeaderValue::from_static("Bearer caller"),
        );
        headers.insert(header::COOKIE, HeaderValue::from_static("secret=true"));

        let request = build_audio_transcriptions_request(&state, &headers).unwrap();

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
                .get(header::CONTENT_TYPE)
                .and_then(|value| value.to_str().ok()),
            Some("multipart/form-data; boundary=audio")
        );
        assert!(!request.headers.contains_key(header::COOKIE));
    }
}
