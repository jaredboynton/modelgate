use std::{net::SocketAddr, time::Duration};

use axum::{
    extract::{
        ws::{Message as AxumMessage, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
    routing::{any, get},
    Router,
};
use serde_json::{json, Value};
use tempfile::TempDir;
use tokio::{net::TcpListener, sync::mpsc, task::JoinHandle};
use unified_model_proxy_v2::{build_router, AppState};
use warpsock::Message as WarpsockMessage;

const MISSING_CODEX_AUTH_MESSAGE: &str = "Missing Codex OAuth credentials at ~/.codex/auth.json.";

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
async fn integration_realtime_ws_requires_single_model_query() {
    for path in [
        "/v1/realtime",
        "/v1/realtime?model=",
        "/v1/realtime?foo=gpt-realtime-2",
        "/v1/realtime?model=gpt-realtime-2&model=gpt-realtime-2",
        "/api/provider/openai/v1/realtime",
        "/api/provider/openai/v1/realtime?foo=gpt-realtime-2",
        "/api/provider/openai/v1/realtime?model=gpt-realtime-2&model=gpt-realtime-2",
    ] {
        let test_state = realtime_test_state_without_codex_auth();
        let proxy = spawn_proxy(test_state.state.clone()).await;
        let mut ws = connect_proxy_ws(&proxy, path).await;

        let error = expect_realtime_error(&mut ws).await;
        assert_eq!(error["status"], 400, "{path}: {error}");
        assert_eq!(
            error["error"]["type"], "invalid_request_error",
            "{path}: {error}"
        );
        assert_eq!(
            error["error"]["code"], "invalid_realtime_model_query",
            "{path}: {error}"
        );
        assert!(error["error"]["param"].is_null(), "{path}: {error}");
        expect_close(&mut ws).await;
        proxy.handle.abort();
    }
}

#[tokio::test]
async fn integration_realtime_ws_unsupported_model_errors_without_upstream() {
    let (capture_tx, mut capture_rx) = mpsc::unbounded_channel();
    let forbidden = spawn_forbidden_upstream(capture_tx).await;
    let test_state = realtime_test_state_with_codex_auth();
    let mut state = test_state.state.clone();
    point_codex_responses_at_forbidden_upstream(&mut state, &forbidden);
    let proxy = spawn_proxy(state).await;

    let mut ws = connect_proxy_ws(&proxy, "/v1/realtime?model=gpt-realtime-1").await;

    let error = expect_realtime_error(&mut ws).await;
    assert_eq!(error["status"], 400, "{error}");
    assert_eq!(error["error"]["type"], "invalid_request_error", "{error}");
    assert_eq!(error["error"]["code"], "model_not_supported", "{error}");
    assert!(error["error"]["message"]
        .as_str()
        .unwrap()
        .contains("gpt-realtime-1"));
    assert!(error["error"]["param"].is_null(), "{error}");
    expect_close(&mut ws).await;
    assert_no_forbidden_upstream(&mut capture_rx).await;

    proxy.handle.abort();
    forbidden.handle.abort();
}

#[tokio::test]
async fn integration_realtime_ws_missing_local_codex_auth_emits_401_without_upstream() {
    let (capture_tx, mut capture_rx) = mpsc::unbounded_channel();
    let forbidden = spawn_forbidden_upstream(capture_tx).await;
    let test_state = realtime_test_state_without_codex_auth();
    let mut state = test_state.state.clone();
    point_codex_responses_at_forbidden_upstream(&mut state, &forbidden);
    let proxy = spawn_proxy(state).await;

    let mut ws = connect_proxy_ws(&proxy, "/v1/realtime?model=gpt-realtime-2").await;

    let error = expect_realtime_error(&mut ws).await;
    assert_missing_codex_auth_error(&error);
    expect_close(&mut ws).await;
    assert_no_forbidden_upstream(&mut capture_rx).await;

    proxy.handle.abort();
    forbidden.handle.abort();
}

#[tokio::test]
async fn integration_realtime_ws_never_calls_codex_responses_paths() {
    let (capture_tx, mut capture_rx) = mpsc::unbounded_channel();
    let forbidden = spawn_forbidden_upstream(capture_tx).await;
    let test_state = realtime_test_state_with_codex_auth();
    let mut state = test_state.state.clone();
    point_codex_responses_at_forbidden_upstream(&mut state, &forbidden);
    let proxy = spawn_proxy(state).await;

    let mut ws = connect_proxy_ws(&proxy, "/v1/realtime?model=codex-responses-ws").await;

    let error = expect_realtime_error(&mut ws).await;
    assert_eq!(error["status"], 400, "{error}");
    assert_eq!(error["error"]["code"], "model_not_supported", "{error}");
    expect_close(&mut ws).await;
    assert_no_forbidden_upstream(&mut capture_rx).await;

    proxy.handle.abort();
    forbidden.handle.abort();
}

#[tokio::test]
async fn integration_realtime_ws_never_calls_app_server_thread_realtime_paths() {
    let (capture_tx, mut capture_rx) = mpsc::unbounded_channel();
    let forbidden = spawn_forbidden_upstream(capture_tx).await;
    let test_state = realtime_test_state_without_codex_auth();
    let mut state = test_state.state.clone();
    point_codex_responses_at_forbidden_upstream(&mut state, &forbidden);
    let proxy = spawn_proxy(state).await;

    let mut ws = connect_proxy_ws(&proxy, "/v1/realtime?model=gpt-realtime-2").await;

    let error = expect_realtime_error(&mut ws).await;
    assert_missing_codex_auth_error(&error);
    expect_close(&mut ws).await;
    assert_no_forbidden_upstream(&mut capture_rx).await;

    proxy.handle.abort();
    forbidden.handle.abort();
}

#[tokio::test]
async fn integration_provider_prefixed_realtime_ws_missing_codex_auth_emits_401_without_upstream() {
    let (capture_tx, mut capture_rx) = mpsc::unbounded_channel();
    let forbidden = spawn_forbidden_upstream(capture_tx).await;
    let test_state = realtime_test_state_without_codex_auth();
    let mut state = test_state.state.clone();
    point_codex_responses_at_forbidden_upstream(&mut state, &forbidden);
    let proxy = spawn_proxy(state).await;

    let mut ws = connect_proxy_ws(
        &proxy,
        "/api/provider/openai/v1/realtime?model=gpt-realtime-2",
    )
    .await;

    let error = expect_realtime_error(&mut ws).await;
    assert_missing_codex_auth_error(&error);
    expect_close(&mut ws).await;
    assert_no_forbidden_upstream(&mut capture_rx).await;

    proxy.handle.abort();
    forbidden.handle.abort();
}

fn realtime_test_state_without_codex_auth() -> TestState {
    realtime_test_state(false)
}

fn realtime_test_state_with_codex_auth() -> TestState {
    realtime_test_state(true)
}

fn realtime_test_state(write_codex_auth: bool) -> TestState {
    let codex_home = TempDir::new().unwrap();
    let auth_home = TempDir::new().unwrap();
    if write_codex_auth {
        std::fs::write(
            codex_home.path().join("auth.json"),
            json!({
                "access_token": "access-token",
                "account_id": "account-123"
            })
            .to_string(),
        )
        .unwrap();
    }
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

fn point_codex_responses_at_forbidden_upstream(state: &mut AppState, upstream: &SpawnedServer) {
    std::sync::Arc::make_mut(&mut state.runtime).codex_responses_wss_url =
        format!("ws://{}/backend-api/codex/responses", upstream.addr);
    std::sync::Arc::make_mut(&mut state.runtime).codex_responses_http_url =
        format!("http://{}/backend-api/codex/responses", upstream.addr);
}

async fn spawn_proxy(state: AppState) -> SpawnedServer {
    spawn_router(build_router(state)).await
}

async fn spawn_forbidden_upstream(capture_tx: mpsc::UnboundedSender<String>) -> SpawnedServer {
    let app = Router::new()
        .route(
            "/backend-api/codex/responses",
            get(forbidden_ws).post(forbidden_http),
        )
        .route("/backend-api/codex/responses/*path", any(forbidden_http))
        .route("/backend-api/conversation", any(forbidden_http))
        .route("/backend-api/conversation/*path", any(forbidden_http))
        .route("/backend-api/thread", any(forbidden_http))
        .route("/backend-api/thread/*path", any(forbidden_http))
        .route("/api/threads/find", any(forbidden_http))
        .route("/api/threads/*path", any(forbidden_http))
        .with_state(capture_tx);
    spawn_router(app).await
}

async fn spawn_router(app: Router) -> SpawnedServer {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let handle = tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    SpawnedServer { addr, handle }
}

async fn forbidden_ws(
    State(capture_tx): State<mpsc::UnboundedSender<String>>,
    ws: WebSocketUpgrade,
    uri: axum::http::Uri,
) -> impl IntoResponse {
    let path = uri.path().to_string();
    ws.on_upgrade(move |mut socket| async move {
        capture_tx.send(path).unwrap();
        let _ = socket
            .send(AxumMessage::Text(
                r#"{"type":"forbidden_upstream_called"}"#.into(),
            ))
            .await;
    })
}

async fn forbidden_http(
    State(capture_tx): State<mpsc::UnboundedSender<String>>,
    uri: axum::http::Uri,
) -> impl IntoResponse {
    capture_tx.send(uri.path().to_string()).unwrap();
    (
        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
        "forbidden upstream called",
    )
}

async fn connect_proxy_ws(proxy: &SpawnedServer, path: &str) -> warpsock::WebSocket {
    warpsock::Client::new()
        .unwrap()
        .websocket(format!("ws://{}{}", proxy.addr, path))
        .connect()
        .await
        .unwrap()
}

async fn expect_realtime_error(ws: &mut warpsock::WebSocket) -> Value {
    let message = tokio::time::timeout(Duration::from_secs(2), ws.next())
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    match message {
        WarpsockMessage::Text(text) => {
            let value = serde_json::from_str::<Value>(&text).unwrap();
            assert_eq!(value["type"], "error", "{value}");
            value
        }
        other => panic!("expected realtime websocket error JSON text frame, got {other:?}"),
    }
}

async fn expect_close(ws: &mut warpsock::WebSocket) {
    let message = tokio::time::timeout(Duration::from_secs(2), ws.next())
        .await
        .unwrap()
        .unwrap();
    if let Some(message) = message {
        assert!(
            matches!(message, WarpsockMessage::Close(_)),
            "expected websocket close frame or closed stream, got {message:?}"
        );
    }
}

fn assert_missing_codex_auth_error(error: &Value) {
    assert_eq!(error["status"], 401, "{error}");
    assert_eq!(error["error"]["type"], "authentication_error", "{error}");
    assert_eq!(error["error"]["code"], "invalid_api_key", "{error}");
    assert_eq!(
        error["error"]["message"], MISSING_CODEX_AUTH_MESSAGE,
        "{error}"
    );
    assert!(error["error"]["param"].is_null(), "{error}");
}

async fn assert_no_forbidden_upstream(capture_rx: &mut mpsc::UnboundedReceiver<String>) {
    let result = tokio::time::timeout(Duration::from_millis(200), capture_rx.recv()).await;
    assert!(
        result.is_err(),
        "forbidden upstream path was called: {result:?}"
    );
}
