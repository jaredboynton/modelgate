use std::sync::Arc;

use bytes::Bytes;
use http::{HeaderMap, HeaderValue, Method, StatusCode};
use unified_model_proxy_v2::{
    upstream::google::{send_google_direct, GoogleRequest},
    AppState,
};
use wiremock::{
    matchers::{body_string_contains, header, method, path},
    Mock, MockServer, ResponseTemplate,
};

fn google_test_state() -> AppState {
    let codex_home = tempfile::tempdir().unwrap();
    let auth_home = tempfile::tempdir().unwrap();
    let mut state = AppState::for_tests(
        codex_home.path().to_path_buf(),
        auth_home.path().to_path_buf(),
    );
    state.google_api_key = Some(Arc::from("fixture-key"));
    state
}

#[tokio::test]
async fn google_direct_transport_uses_warpsock_to_post_body_and_headers() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(
            "/v1beta/models/gemini-3-flash-preview:generateContent",
        ))
        .and(header("x-goog-api-key", "fixture-key"))
        .and(header("accept", "application/json"))
        .and(body_string_contains("transport-check"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "application/json")
                .set_body_raw(r#"{"candidates":[]}"#, "application/json"),
        )
        .mount(&server)
        .await;

    let mut headers = HeaderMap::new();
    headers.insert("x-goog-api-key", HeaderValue::from_static("fixture-key"));
    headers.insert("accept", HeaderValue::from_static("application/json"));

    let response = send_google_direct(
        &google_test_state(),
        Method::POST,
        GoogleRequest {
            url: format!(
                "{}/v1beta/models/gemini-3-flash-preview:generateContent",
                server.uri()
            ),
            headers,
        },
        Bytes::from_static(br#"{"contents":[{"parts":[{"text":"transport-check"}]}]}"#),
        false,
    )
    .await
    .unwrap();

    assert_eq!(response.status, StatusCode::OK);
}
