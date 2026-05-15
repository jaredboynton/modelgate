use std::{env, ffi::OsString, fs, net::SocketAddr, time::Duration};

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

static COMPACTION_ENV_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

struct EnvRestore {
    key: &'static str,
    previous: Option<OsString>,
}

impl EnvRestore {
    fn set(key: &'static str, value: &str) -> Self {
        let previous = env::var_os(key);
        env::set_var(key, value);
        Self { key, previous }
    }
}

impl Drop for EnvRestore {
    fn drop(&mut self) {
        match &self.previous {
            Some(value) => env::set_var(self.key, value),
            None => env::remove_var(self.key),
        }
    }
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
async fn websocket_facade_allows_model_switches_after_terminal_events() {
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
    let bedrock_created = expect_json_frame(&mut ws).await;
    assert_eq!(bedrock_created["type"], "response.created");
    let bedrock_completed = expect_json_frame(&mut ws).await;
    assert_eq!(bedrock_completed["type"], "response.completed");

    assert_no_raw_sse(&google_created);
    assert_no_raw_sse(&google_completed);
    assert_no_raw_sse(&bedrock_created);
    assert_no_raw_sse(&bedrock_completed);

    proxy.handle.abort();
}

#[tokio::test]
async fn websocket_facade_generate_false_prewarm_then_real_cross_model_turn_reaches_provider() {
    let test_state = http_backed_mixed_state();
    let proxy = spawn_proxy(test_state.state.clone()).await;
    let mut ws = connect_proxy_ws(&proxy).await;

    ws.send_text(response_create_generate_false("facade-bedrock-model").to_string())
        .await
        .unwrap();
    let _created = expect_json_frame(&mut ws).await;
    let _completed = expect_json_frame(&mut ws).await;

    ws.send_text(response_create("facade-google-model", "real turn").to_string())
        .await
        .unwrap();

    let error = expect_json_frame(&mut ws).await;
    assert_eq!(error["type"], "error");
    assert_ne!(error["error"]["code"], "websocket_route_model_changed");
    let message = error["error"]["message"].as_str().unwrap();
    assert!(
        message.contains("missing credential") || message.contains("Google API key"),
        "{message}"
    );

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
async fn websocket_facade_rejects_malformed_previous_response_id_and_recovers() {
    let test_state = http_backed_mixed_state();
    let proxy = spawn_proxy(test_state.state.clone()).await;
    let mut ws = connect_proxy_ws(&proxy).await;

    ws.send_text(
        response_create_with_previous_response_id_value(
            "facade-google-model",
            json!({ "id": "not-a-string" }),
        )
        .to_string(),
    )
    .await
    .unwrap();

    let error = expect_json_frame(&mut ws).await;
    assert_eq!(error["type"], "error");
    assert_eq!(error["error"]["code"], "previous_response_field_mismatch");
    assert!(error["error"]["message"]
        .as_str()
        .unwrap()
        .contains("previous_response_id must be a string"));
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
        Some("previous_response_route_mismatch")
            | Some("previous_response_model_mismatch")
            | Some("previous_response_target_format_mismatch")
    ));
    let message = error["error"]["message"].as_str().unwrap();
    assert!(
        !message.contains("route/model changes on one socket"),
        "cross-route continuation must be a recoverable previous_response_id policy error: {message}"
    );
    assert_no_raw_sse(&error);

    ws.send_text(response_create_generate_false("facade-bedrock-model").to_string())
        .await
        .unwrap();
    let next_created = expect_json_frame(&mut ws).await;
    assert_eq!(next_created["type"], "response.created");
    let next_completed = expect_json_frame(&mut ws).await;
    assert_eq!(next_completed["type"], "response.completed");

    proxy.handle.abort();
}

#[tokio::test]
async fn websocket_facade_rejects_previous_response_id_after_target_format_hot_change() {
    let test_state = http_backed_google_state();
    let proxy = spawn_proxy(test_state.state.clone()).await;
    let mut ws = connect_proxy_ws(&proxy).await;

    ws.send_text(response_create_generate_false("facade-google-model").to_string())
        .await
        .unwrap();
    let created = expect_json_frame(&mut ws).await;
    let response_id = created["response"]["id"].as_str().unwrap().to_string();
    let _completed = expect_json_frame(&mut ws).await;

    fs::write(
        test_state._auth_home.path().join("config.json"),
        json!({
            "routes": [{
                "source": { "model": "facade-google-model", "format": "responses" },
                "target": {
                    "provider": "codex",
                    "model": "gpt-5.5",
                    "format": "responses"
                }
            }]
        })
        .to_string(),
    )
    .unwrap();

    ws.send_text(
        response_create_with_previous_response_id("facade-google-model", &response_id).to_string(),
    )
    .await
    .unwrap();

    let error = expect_json_frame(&mut ws).await;
    assert_eq!(error["type"], "error");
    assert_eq!(
        error["error"]["code"],
        "previous_response_target_format_mismatch"
    );
    assert_no_raw_sse(&error);

    proxy.handle.abort();
}

#[tokio::test]
async fn websocket_facade_invalid_target_format_edge_fails_before_credentials() {
    let test_state = invalid_google_target_format_state();
    let proxy = spawn_proxy(test_state.state.clone()).await;
    let mut ws = connect_proxy_ws(&proxy).await;

    ws.send_text(response_create_generate_false("invalid-google-target").to_string())
        .await
        .unwrap();

    let error = expect_json_frame(&mut ws).await;
    assert_eq!(error["type"], "error");
    assert_eq!(error["status"], 400);
    assert_eq!(error["error"]["code"], "model_not_supported");
    assert!(!error["error"]["message"]
        .as_str()
        .unwrap()
        .contains("Google API key"));
    assert_no_raw_sse(&error);

    proxy.handle.abort();
}

#[tokio::test]
async fn websocket_facade_emulates_context_compaction_for_non_codex_target() {
    let _guard = COMPACTION_ENV_LOCK.lock().await;
    let _key_env = EnvRestore::set(
        "UMP_COMPACTION_KEYS_JSON",
        r#"{"current":"fixture","keys":{"fixture":"AAECAwQFBgcICQoLDA0ODxAREhMUFRYXGBkaGxwdHh8"}}"#,
    );
    let _instance_env = EnvRestore::set("UMP_COMPACTION_INSTANCE_ID", "ws-facade-test");
    let test_state = http_backed_mixed_proxy_compaction_state();
    let proxy = spawn_proxy(test_state.state.clone()).await;
    let mut ws = connect_proxy_ws(&proxy).await;

    ws.send_text(response_create_context_compaction("facade-bedrock-model").to_string())
        .await
        .unwrap();

    let created = expect_json_frame(&mut ws).await;
    assert_eq!(created["type"], "response.created");
    let added = expect_json_frame(&mut ws).await;
    assert_eq!(added["type"], "response.output_item.added");
    assert_eq!(added["item"]["type"], "context_compaction");
    let done = expect_json_frame(&mut ws).await;
    assert_eq!(done["type"], "response.output_item.done");
    assert_eq!(done["item"]["type"], "context_compaction");
    let encrypted_content = done["item"]["encrypted_content"].as_str().unwrap();
    assert!(encrypted_content.starts_with("ump.compaction.v1."));
    let completed = expect_json_frame(&mut ws).await;
    assert_eq!(completed["type"], "response.completed");
    assert_no_raw_sse(&created);
    assert_no_raw_sse(&added);
    assert_no_raw_sse(&done);
    assert_no_raw_sse(&completed);

    ws.send_text(
        response_create_generate_false_with_input(
            "facade-bedrock-model",
            json!([{
                "type": "context_compaction",
                "encrypted_content": encrypted_content
            }]),
        )
        .to_string(),
    )
    .await
    .unwrap();

    let roundtrip_created = expect_json_frame(&mut ws).await;
    assert_eq!(roundtrip_created["type"], "response.created");
    let roundtrip_completed = expect_json_frame(&mut ws).await;
    assert_eq!(roundtrip_completed["type"], "response.completed");
    assert_no_raw_sse(&roundtrip_created);
    assert_no_raw_sse(&roundtrip_completed);

    proxy.handle.abort();
}

#[tokio::test]
async fn websocket_facade_rejects_v2_context_compaction_with_input_encrypted_content() {
    let test_state = http_backed_mixed_state();
    let proxy = spawn_proxy(test_state.state.clone()).await;
    let mut ws = connect_proxy_ws(&proxy).await;

    ws.send_text(
        json!({
            "type": "response.create",
            "response": {
                "model": "facade-bedrock-model",
                "input": [{
                    "type": "context_compaction",
                    "encrypted_content": "client-must-not-send-this"
                }],
                "stream": true
            }
        })
        .to_string(),
    )
    .await
    .unwrap();

    let error = expect_json_frame(&mut ws).await;
    assert_eq!(error["type"], "error");
    assert_eq!(error["status"], 400);
    assert_eq!(
        error["error"]["code"],
        "unsupported_compaction_item_for_target"
    );
    assert_no_raw_sse(&error);

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

fn http_backed_mixed_proxy_compaction_state() -> TestState {
    http_backed_state_with_policy(
        vec![
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
        ],
        Some("proxy_visible_summary"),
    )
}

fn invalid_google_target_format_state() -> TestState {
    let codex_home = TempDir::new().unwrap();
    let auth_home = TempDir::new().unwrap();
    fs::write(
        auth_home.path().join("config.json"),
        json!({
            "routes": [{
                "source": { "model": "invalid-google-target", "format": "responses" },
                "target": {
                    "provider": "google",
                    "model": "gemini-3.1-flash-lite",
                    "format": "responses"
                }
            }]
        })
        .to_string(),
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

fn http_backed_state(routes: Vec<(&str, &str, &str)>) -> TestState {
    http_backed_state_with_policy(routes, None)
}

fn http_backed_state_with_policy(
    routes: Vec<(&str, &str, &str)>,
    remote_compaction_policy: Option<&str>,
) -> TestState {
    let codex_home = TempDir::new().unwrap();
    let auth_home = TempDir::new().unwrap();
    let routes = routes
        .into_iter()
        .map(|(source_model, provider, target_model)| {
            let target_format = match provider {
                "bedrock" => "anthropic_messages",
                "codex" => "responses",
                "google" => "google_generate_content",
                _ => "responses",
            };
            let mut route = json!({
                "source": { "model": source_model, "format": "responses" },
                "target": {
                    "provider": provider,
                    "model": target_model,
                    "format": target_format
                }
            });
            if let Some(remote_compaction_policy) = remote_compaction_policy {
                route.as_object_mut().expect("route object").insert(
                    "remote_compaction_policy".into(),
                    json!(remote_compaction_policy),
                );
            }
            route
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
    response_create_generate_false_with_input(model, json!("warm this connection"))
}

fn response_create_generate_false_with_input(model: &str, input: Value) -> Value {
    json!({
        "type": "response.create",
        "response": {
            "model": model,
            "input": input,
            "stream": true,
            "generate": false
        }
    })
}

fn response_create_context_compaction(model: &str) -> Value {
    json!({
        "type": "response.create",
        "response": {
            "model": model,
            "input": [{
                "type": "context_compaction"
            }],
            "stream": true
        }
    })
}

fn response_create_with_previous_response_id(model: &str, previous_response_id: &str) -> Value {
    response_create_with_previous_response_id_value(
        model,
        Value::String(previous_response_id.into()),
    )
}

fn response_create_with_previous_response_id_value(
    model: &str,
    previous_response_id: Value,
) -> Value {
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
