use std::{
    net::SocketAddr,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
};
use unified_model_proxy_v2::{upstream::codex, AppState};
use wiremock::{
    matchers::{header, method, path},
    Mock, MockServer, ResponseTemplate,
};

#[tokio::test]
async fn codex_wss_failure_uses_http_fallback_and_normalizes_sse() {
    let codex_home = tempfile::tempdir().unwrap();
    let auth_home = tempfile::tempdir().unwrap();
    std::fs::write(
        codex_home.path().join("auth.json"),
        serde_json::json!({
            "access_token": "access-token",
            "account_id": "account-123"
        })
        .to_string(),
    )
    .unwrap();
    let state = AppState::for_tests(
        codex_home.path().to_path_buf(),
        auth_home.path().to_path_buf(),
    );

    let (wss_url, handshakes) = rejecting_ws_server().await;
    let http = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/backend-api/codex/responses"))
        .and(header("originator", "codex_cli_rs"))
        .and(header("OpenAI-Beta", "responses_websockets=2026-02-06"))
        .and(header("ChatGPT-Account-Id", "account-123"))
        .respond_with(ResponseTemplate::new(200).set_body_string(concat!(
            "event: codex.debug\n",
            "data: hidden\n",
            "\n",
            "event: response.output_item.done\n",
            "data: {\"item\":{\"id\":\"call_1\",\"type\":\"function_call\"}}\n",
            "\n",
            "event: response.completed\n",
            "data: {\"response\":{\"id\":\"resp_1\",\"status\":\"completed\"}}\n",
            "\n",
        )))
        .mount(&http)
        .await;

    let body = serde_json::json!({
        "model": "openai:gpt-5.5",
        "input": "hello"
    });
    let bytes = codex::responses_with_endpoints(
        &state,
        body,
        &wss_url,
        &format!("{}/backend-api/codex/responses", http.uri()),
    )
    .await
    .unwrap();
    let text = String::from_utf8(bytes.to_vec()).unwrap();

    assert_eq!(handshakes.load(Ordering::SeqCst), 1);
    assert!(!text.contains("codex.debug"));
    assert!(text.contains("event: response.completed"));
    assert!(text.contains(r#""output":[{"id":"call_1","type":"function_call"}]"#));
}

#[test]
fn codex_headers_advertise_remote_compaction_beta_without_dropping_websocket_beta() {
    let codex_home = tempfile::tempdir().unwrap();
    let auth_home = tempfile::tempdir().unwrap();
    std::fs::write(
        codex_home.path().join("auth.json"),
        serde_json::json!({
            "access_token": "access-token",
            "account_id": "account-123"
        })
        .to_string(),
    )
    .unwrap();
    let state = AppState::for_tests(
        codex_home.path().to_path_buf(),
        auth_home.path().to_path_buf(),
    );

    let headers = codex::codex_headers(&state).unwrap();
    assert_eq!(headers["OpenAI-Beta"], "responses_websockets=2026-02-06");
    assert!(headers
        .get("x-codex-beta-features")
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value
            .split(',')
            .any(|feature| feature.trim() == "remote_compaction_v2")));
}

async fn rejecting_ws_server() -> (String, Arc<AtomicUsize>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr: SocketAddr = listener.local_addr().unwrap();
    let handshakes = Arc::new(AtomicUsize::new(0));
    let server_handshakes = handshakes.clone();
    tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();
        server_handshakes.fetch_add(1, Ordering::SeqCst);
        let mut buf = [0_u8; 4096];
        let _ = socket.read(&mut buf).await.unwrap();
        socket
            .write_all(b"HTTP/1.1 403 Forbidden\r\nContent-Length: 0\r\n\r\n")
            .await
            .unwrap();
    });
    (
        format!("ws://{addr}/backend-api/codex/responses"),
        handshakes,
    )
}
