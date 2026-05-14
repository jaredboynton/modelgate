use std::{net::SocketAddr, time::Duration};

use serde_json::{json, Value};
use specter::Message as SpecterMessage;
use tempfile::TempDir;
use tokio::{net::TcpListener, task::JoinHandle};
use unified_model_proxy_v2::{build_router, AppState};

struct TestState {
    _codex_home: TempDir,
    _auth_home: TempDir,
    state: AppState,
}

struct SpawnedServer {
    addr: SocketAddr,
    handle: JoinHandle<()>,
}

#[tokio::test]
async fn websocket_facade_generate_false_sends_synthetic_lifecycle() {
    let test_state = http_backed_google_state();
    let proxy = spawn_proxy(test_state.state.clone()).await;
    let mut ws = connect_proxy_ws(&proxy).await;

    ws.send_text(include_str!("fixtures/codex_ws/generate_false.json"))
        .await
        .unwrap();

    let created = expect_json_frame(&mut ws).await;
    assert_eq!(created["type"], "response.created");
    let response_id = created["response"]["id"].as_str().unwrap().to_string();
    assert!(response_id.starts_with("resp_ws_bridge_"));

    let completed = expect_json_frame(&mut ws).await;
    assert_eq!(completed["type"], "response.completed");
    assert_eq!(completed["response"]["id"], response_id);
    assert_eq!(completed["response"]["usage"]["total_tokens"], 0);
    assert_no_raw_sse(&created);
    assert_no_raw_sse(&completed);

    ws.send_text(
        json!({
            "type": "response.processed",
            "response_id": response_id
        })
        .to_string(),
    )
    .await
    .unwrap();

    let _ = ws.close(None).await;
    proxy.handle.abort();
}

#[tokio::test]
async fn websocket_facade_rejects_model_switches_after_terminal_events() {
    let test_state = http_backed_mixed_state();
    let proxy = spawn_proxy(test_state.state.clone()).await;
    let mut ws = connect_proxy_ws(&proxy).await;

    ws.send_text(response_create_generate_false("facade-google-model").to_string())
        .await
        .unwrap();
    let google_created = expect_json_frame(&mut ws).await;
    assert_eq!(google_created["type"], "response.created");
    let google_completed = expect_json_frame(&mut ws).await;
    assert_eq!(google_completed["type"], "response.completed");

    ws.send_text(response_create_generate_false("facade-bedrock-model").to_string())
        .await
        .unwrap();
    let error = expect_json_frame(&mut ws).await;
    assert_eq!(error["type"], "error");
    assert_eq!(error["error"]["code"], "websocket_route_model_changed");

    assert_no_raw_sse(&google_created);
    assert_no_raw_sse(&google_completed);
    assert_no_raw_sse(&error);

    proxy.handle.abort();
}

#[tokio::test]
async fn websocket_facade_generate_false_prewarm_then_real_cross_model_turn_rejects() {
    let test_state = http_backed_mixed_state();
    let proxy = spawn_proxy(test_state.state.clone()).await;
    let mut ws = connect_proxy_ws(&proxy).await;

    ws.send_text(response_create_generate_false("facade-google-model").to_string())
        .await
        .unwrap();
    let _created = expect_json_frame(&mut ws).await;
    let _completed = expect_json_frame(&mut ws).await;

    ws.send_text(response_create("facade-bedrock-model", "real turn").to_string())
        .await
        .unwrap();

    let error = expect_json_frame(&mut ws).await;
    assert_eq!(error["type"], "error");
    assert_eq!(error["error"]["code"], "websocket_route_model_changed");

    proxy.handle.abort();
}

#[tokio::test]
async fn websocket_facade_previous_response_id_is_connection_local_and_stripped_before_adapter() {
    let test_state = http_backed_google_state();
    let proxy = spawn_proxy(test_state.state.clone()).await;
    let mut ws = connect_proxy_ws(&proxy).await;

    ws.send_text(include_str!("fixtures/codex_ws/generate_false.json"))
        .await
        .unwrap();
    let created = expect_json_frame(&mut ws).await;
    let response_id = created["response"]["id"].as_str().unwrap().to_string();
    let _completed = expect_json_frame(&mut ws).await;

    let followup = include_str!("fixtures/codex_ws/previous_response_id_delta.json")
        .replace("__REPLACE_RESPONSE_ID__", &response_id);
    ws.send_text(followup).await.unwrap();

    let error = expect_json_frame(&mut ws).await;
    assert_eq!(error["type"], "error");
    assert!(
        !error["error"]["message"]
            .as_str()
            .unwrap()
            .contains("previous_response_id is not supported"),
        "bridge must strip known previous_response_id before reaching HTTP adapters"
    );
    let message = error["error"]["message"].as_str().unwrap();
    assert!(
        message.contains("missing credential") || message.contains("Google API key"),
        "{message}"
    );

    proxy.handle.abort();
}

#[tokio::test]
async fn websocket_facade_unknown_previous_response_id_returns_json_error_not_raw_sse() {
    let test_state = http_backed_google_state();
    let proxy = spawn_proxy(test_state.state.clone()).await;
    let mut ws = connect_proxy_ws(&proxy).await;

    let followup = include_str!("fixtures/codex_ws/previous_response_id_delta.json")
        .replace("__REPLACE_RESPONSE_ID__", "resp_unknown");
    ws.send_text(followup).await.unwrap();

    let error = expect_json_frame(&mut ws).await;
    assert_eq!(error["type"], "error");
    assert_eq!(error["error"]["code"], "unknown_previous_response_id");
    assert!(error["error"]["message"]
        .as_str()
        .unwrap()
        .contains("unknown previous_response_id"));
    assert_no_raw_sse(&error);

    proxy.handle.abort();
}

#[tokio::test]
async fn websocket_facade_first_frame_unknown_previous_response_id_keeps_socket_usable() {
    let test_state = http_backed_mixed_state();
    let proxy = spawn_proxy(test_state.state.clone()).await;
    let mut ws = connect_proxy_ws(&proxy).await;

    let followup = response_create_with_previous_response_id(
        "facade-google-model",
        "resp_never_seen_on_this_socket",
    );
    ws.send_text(followup.to_string()).await.unwrap();

    let error = expect_json_frame(&mut ws).await;
    assert_eq!(error["type"], "error");
    assert_eq!(error["error"]["code"], "unknown_previous_response_id");
    assert!(error["error"]["message"]
        .as_str()
        .unwrap()
        .contains("unknown previous_response_id"));
    assert_no_raw_sse(&error);

    ws.send_text(response_create_generate_false("facade-bedrock-model").to_string())
        .await
        .unwrap();
    let created = expect_json_frame(&mut ws).await;
    assert_eq!(created["type"], "response.created");
    let completed = expect_json_frame(&mut ws).await;
    assert_eq!(completed["type"], "response.completed");

    proxy.handle.abort();
}

#[tokio::test]
async fn websocket_facade_cross_route_previous_response_id_error_keeps_socket_usable() {
    let test_state = http_backed_mixed_state();
    let proxy = spawn_proxy(test_state.state.clone()).await;
    let mut ws = connect_proxy_ws(&proxy).await;

    ws.send_text(response_create_generate_false("facade-google-model").to_string())
        .await
        .unwrap();
    let created = expect_json_frame(&mut ws).await;
    let response_id = created["response"]["id"].as_str().unwrap().to_string();
    let _completed = expect_json_frame(&mut ws).await;

    ws.send_text(
        response_create_with_previous_response_id("facade-bedrock-model", &response_id).to_string(),
    )
    .await
    .unwrap();

    let error = expect_json_frame(&mut ws).await;
    assert_eq!(error["type"], "error");
    assert!(matches!(
        error["error"]["code"].as_str(),
        Some("previous_response_route_mismatch") | Some("previous_response_model_mismatch")
    ));
    let message = error["error"]["message"].as_str().unwrap();
    assert!(
        !message.contains("route/model changes on one socket"),
        "cross-route continuation must be a recoverable previous_response_id policy error: {message}"
    );
    assert_no_raw_sse(&error);

    ws.send_text(response_create_generate_false("facade-google-model").to_string())
        .await
        .unwrap();
    let next_created = expect_json_frame(&mut ws).await;
    assert_eq!(next_created["type"], "response.created");
    let next_completed = expect_json_frame(&mut ws).await;
    assert_eq!(next_completed["type"], "response.completed");

    proxy.handle.abort();
}

fn http_backed_google_state() -> TestState {
    http_backed_state(vec![(
        "facade-google-model",
        "google",
        "gemini-3.1-flash-lite",
    )])
}

fn http_backed_mixed_state() -> TestState {
    http_backed_state(vec![
        ("facade-google-model", "google", "gemini-3.1-flash-lite"),
        (
            "facade-google-alt-model",
            "google",
            "gemini-3.1-flash-lite-alt",
        ),
        (
            "facade-bedrock-model",
            "bedrock",
            "anthropic.claude-haiku-4-5",
        ),
    ])
}

fn http_backed_state(routes: Vec<(&str, &str, &str)>) -> TestState {
    let codex_home = TempDir::new().unwrap();
    let auth_home = TempDir::new().unwrap();
    let routes = routes
        .into_iter()
        .map(|(source_model, provider, target_model)| {
            json!({
                "source": { "model": source_model, "format": "responses" },
                "target": {
                    "provider": provider,
                    "model": target_model,
                    "format": "responses"
                }
            })
        })
        .collect::<Vec<_>>();
    std::fs::write(
        auth_home.path().join("config.json"),
        json!({ "routes": routes }).to_string(),
    )
    .unwrap();
    let state = AppState::for_tests(
        codex_home.path().to_path_buf(),
        auth_home.path().to_path_buf(),
    );
    TestState {
        _codex_home: codex_home,
        _auth_home: auth_home,
        state,
    }
}

fn response_create(model: &str, input: &str) -> Value {
    json!({
        "type": "response.create",
        "response": {
            "model": model,
            "input": input,
            "stream": true
        }
    })
}

fn response_create_generate_false(model: &str) -> Value {
    json!({
        "type": "response.create",
        "response": {
            "model": model,
            "input": "warm this connection",
            "stream": true,
            "generate": false
        }
    })
}

fn response_create_with_previous_response_id(model: &str, previous_response_id: &str) -> Value {
    json!({
        "type": "response.create",
        "response": {
            "model": model,
            "input": [],
            "stream": true,
            "previous_response_id": previous_response_id
        }
    })
}

async fn spawn_proxy(state: AppState) -> SpawnedServer {
    let app = build_router(state);
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let handle = tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    SpawnedServer { addr, handle }
}

async fn connect_proxy_ws(proxy: &SpawnedServer) -> specter::WebSocket {
    specter::Client::new()
        .unwrap()
        .websocket(format!("ws://{}/v1/responses", proxy.addr))
        .connect()
        .await
        .unwrap()
}

async fn expect_json_frame(ws: &mut specter::WebSocket) -> Value {
    let message = tokio::time::timeout(Duration::from_secs(2), ws.next())
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    match message {
        SpecterMessage::Text(text) => {
            assert!(
                !text.starts_with("event:") && !text.starts_with("data:"),
                "raw SSE must not be sent over downstream WS: {text}"
            );
            serde_json::from_str(&text).unwrap()
        }
        other => panic!("expected websocket text JSON frame, got {other:?}"),
    }
}

fn assert_no_raw_sse(value: &Value) {
    let text = value.to_string();
    assert!(!text.starts_with("event:"));
    assert!(!text.starts_with("data:"));
}
