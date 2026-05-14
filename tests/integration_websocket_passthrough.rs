use std::{net::SocketAddr, time::Duration};

use axum::{
    extract::{
        ws::{Message as AxumMessage, WebSocketUpgrade},
        State,
    },
    http::HeaderMap,
    response::IntoResponse,
    routing::get,
    Router,
};
use bytes::Bytes;
use serde_json::{json, Value};
use specter::Message as SpecterMessage;
use tempfile::TempDir;
use tokio::{net::TcpListener, sync::mpsc, task::JoinHandle};
use unified_model_proxy_v2::{build_router, AppState};

#[derive(Debug)]
struct CapturedWebSocket {
    headers: Vec<(String, String)>,
    first_frame: String,
}

#[derive(Clone, Copy)]
enum UpstreamBehavior {
    Complete,
    CloseAfterFirstFrame,
    DisconnectAfterFirstFrame,
    DelayComplete(Duration),
    PingThenWaitForPong,
}

#[derive(Debug)]
enum CapturedEvent {
    FirstFrame(CapturedWebSocket),
    Control(String),
}

struct TestState {
    _codex_home: TempDir,
    _auth_home: TempDir,
    state: AppState,
}

#[tokio::test]
async fn responses_websocket_passthrough_uses_codex_auth_and_hot_routing() {
    let (capture_tx, mut capture_rx) = mpsc::unbounded_channel();
    let upstream = spawn_upstream(capture_tx, UpstreamBehavior::Complete).await;

    let test_state = codex_test_state_with_route("gemini-3.1-flash-lite");
    let mut state = test_state.state.clone();
    state.runtime.codex_responses_wss_url =
        format!("ws://{}/backend-api/codex/responses", upstream.addr);
    let proxy = spawn_proxy(state).await;

    let client = specter::Client::new().unwrap();
    let mut ws = client
        .websocket(format!("ws://{}/v1/responses", proxy.addr))
        .connect()
        .await
        .unwrap();
    ws.send_text(
        json!({
            "model": "gemini-3.1-flash-lite",
            "input": "hello from ws",
            "stream": true
        })
        .to_string(),
    )
    .await
    .unwrap();

    let upstream_message = ws.next().await.unwrap().unwrap();
    match upstream_message {
        SpecterMessage::Text(text) => assert!(text.contains("response.completed")),
        other => panic!("unexpected proxy websocket message: {other:?}"),
    }

    let captured = expect_first_frame(&mut capture_rx).await;
    let frame: Value = serde_json::from_str(&captured.first_frame).unwrap();
    assert_eq!(frame["type"], "response.create");
    assert_eq!(frame["model"], "gpt-5.5");
    assert_eq!(frame["stream"], true);
    assert_eq!(frame["input"][0]["content"][0]["text"], "hello from ws");
    assert!(frame
        .get("include")
        .and_then(Value::as_array)
        .unwrap()
        .contains(&json!("reasoning.encrypted_content")));

    assert_eq!(
        header(&captured.headers, "authorization"),
        "Bearer access-token"
    );
    assert_eq!(header(&captured.headers, "originator"), "codex_cli_rs");
    assert_eq!(
        header(&captured.headers, "openai-beta"),
        "responses_websockets=2026-02-06"
    );
    assert_eq!(
        header(&captured.headers, "chatgpt-account-id"),
        "account-123"
    );

    let _ = ws.close(None).await;
    proxy.handle.abort();
    upstream.handle.abort();
}

#[tokio::test]
async fn responses_websocket_accepts_flat_codex_cli_response_create_frame() {
    let (capture_tx, mut capture_rx) = mpsc::unbounded_channel();
    let upstream = spawn_upstream(capture_tx, UpstreamBehavior::Complete).await;

    let test_state = codex_test_state_with_route("codex-cli-flat-model");
    let mut state = test_state.state.clone();
    state.runtime.codex_responses_wss_url =
        format!("ws://{}/backend-api/codex/responses", upstream.addr);
    let proxy = spawn_proxy(state).await;

    let mut ws = connect_proxy_ws(&proxy, "/v1/responses").await;
    ws.send_text(
        json!({
            "type": "response.create",
            "model": "codex-cli-flat-model",
            "input": "flat frame from Codex CLI",
            "stream": true
        })
        .to_string(),
    )
    .await
    .unwrap();

    let upstream_message = ws.next().await.unwrap().unwrap();
    match upstream_message {
        SpecterMessage::Text(text) => assert!(text.contains("response.completed")),
        other => panic!("unexpected proxy websocket message: {other:?}"),
    }

    let captured = expect_first_frame(&mut capture_rx).await;
    let frame: Value = serde_json::from_str(&captured.first_frame).unwrap();
    assert_eq!(frame["type"], "response.create");
    assert_eq!(frame["model"], "gpt-5.5");
    assert_eq!(
        frame["input"][0]["content"][0]["text"],
        "flat frame from Codex CLI"
    );
    assert!(
        frame.get("response").is_none(),
        "flat Codex CLI frames must stay flat upstream"
    );
    assert_eq!(frame["stream"], true);

    let _ = ws.close(None).await;
    proxy.handle.abort();
    upstream.handle.abort();
}

#[tokio::test]
async fn responses_websocket_rejects_followup_model_switch_frames() {
    let (capture_tx, mut capture_rx) = mpsc::unbounded_channel();
    let upstream = spawn_upstream(capture_tx, UpstreamBehavior::Complete).await;

    let test_state = codex_test_state_with_routes(&[
        ("first-frame-model", "gpt-5.5"),
        ("second-frame-model", "gpt-5.4"),
    ]);
    let mut state = test_state.state.clone();
    state.runtime.codex_responses_wss_url =
        format!("ws://{}/backend-api/codex/responses", upstream.addr);
    let proxy = spawn_proxy(state).await;

    let mut ws = connect_proxy_ws(&proxy, "/v1/responses").await;
    ws.send_text(
        json!({
            "type": "response.create",
            "model": "first-frame-model",
            "input": "first",
            "stream": true
        })
        .to_string(),
    )
    .await
    .unwrap();

    let first = expect_first_frame(&mut capture_rx).await;
    let frame: Value = serde_json::from_str(&first.first_frame).unwrap();
    assert_eq!(frame["model"], "gpt-5.5");
    assert_eq!(frame["stream"], true);
    let completed = expect_json_frame(&mut ws).await;
    assert_eq!(completed["type"], "response.completed");

    ws.send_text(
        json!({
            "type": "response.create",
            "model": "second-frame-model",
            "input": "second",
            "stream": true
        })
        .to_string(),
    )
    .await
    .unwrap();

    let error = expect_json_frame(&mut ws).await;
    assert_eq!(error["type"], "error");
    assert_eq!(error["error"]["code"], "websocket_route_model_changed");
    assert_no_upstream_frame(&mut capture_rx).await;

    let _ = ws.close(None).await;
    proxy.handle.abort();
    upstream.handle.abort();
}

#[tokio::test]
async fn responses_websocket_rejects_codex_then_google_and_bedrock_independent_turns() {
    let (capture_tx, mut capture_rx) = mpsc::unbounded_channel();
    let upstream = spawn_upstream(capture_tx, UpstreamBehavior::Complete).await;

    let test_state = codex_test_state_with_route_specs(&[
        ("codex-first-model", "codex", "gpt-5.5"),
        ("google-second-model", "google", "gemini-3-flash-preview"),
        (
            "bedrock-third-model",
            "bedrock",
            "anthropic.claude-opus-4-7",
        ),
    ]);
    let mut state = test_state.state.clone();
    state.runtime.codex_responses_wss_url =
        format!("ws://{}/backend-api/codex/responses", upstream.addr);
    let proxy = spawn_proxy(state).await;

    let mut ws = connect_proxy_ws(&proxy, "/v1/responses").await;
    ws.send_text(
        json!({
            "type": "response.create",
            "model": "codex-first-model",
            "input": "codex turn",
            "stream": true
        })
        .to_string(),
    )
    .await
    .unwrap();

    let first = expect_json_frame(&mut ws).await;
    assert_eq!(first["type"], "response.completed");
    let captured = expect_first_frame(&mut capture_rx).await;
    let frame: Value = serde_json::from_str(&captured.first_frame).unwrap();
    assert_eq!(frame["model"], "gpt-5.5");

    ws.send_text(
        json!({
            "type": "response.create",
            "response": {
                "model": "google-second-model",
                "input": "google independent turn",
                "generate": false
            }
        })
        .to_string(),
    )
    .await
    .unwrap();
    let google_error = expect_json_frame(&mut ws).await;
    assert_eq!(google_error["type"], "error");
    assert_eq!(
        google_error["error"]["code"],
        "websocket_route_model_changed"
    );

    ws.send_text(
        json!({
            "type": "response.create",
            "response": {
                "model": "bedrock-third-model",
                "input": "bedrock independent turn",
                "generate": false
            }
        })
        .to_string(),
    )
    .await
    .unwrap();
    let bedrock_error = expect_json_frame(&mut ws).await;
    assert_eq!(bedrock_error["type"], "error");
    assert_eq!(
        bedrock_error["error"]["code"],
        "websocket_route_model_changed"
    );
    assert_no_upstream_frame(&mut capture_rx).await;

    let _ = ws.close(None).await;
    proxy.handle.abort();
    upstream.handle.abort();
}

#[tokio::test]
async fn responses_websocket_rejects_google_and_bedrock_then_codex_independent_turns() {
    let (capture_tx, mut capture_rx) = mpsc::unbounded_channel();
    let upstream = spawn_upstream(capture_tx, UpstreamBehavior::Complete).await;

    let test_state = codex_test_state_with_route_specs(&[
        ("google-first-model", "google", "gemini-3-flash-preview"),
        (
            "bedrock-second-model",
            "bedrock",
            "anthropic.claude-opus-4-7",
        ),
        ("codex-third-model", "codex", "gpt-5.5"),
    ]);
    let mut state = test_state.state.clone();
    state.runtime.codex_responses_wss_url =
        format!("ws://{}/backend-api/codex/responses", upstream.addr);
    let proxy = spawn_proxy(state).await;

    let mut ws = connect_proxy_ws(&proxy, "/v1/responses").await;
    ws.send_text(
        json!({
            "type": "response.create",
            "response": {
                "model": "google-first-model",
                "input": "google-first-model prewarm",
                "generate": false
            }
        })
        .to_string(),
    )
    .await
    .unwrap();
    let created = expect_json_frame(&mut ws).await;
    assert_eq!(created["type"], "response.created");
    let completed = expect_json_frame(&mut ws).await;
    assert_eq!(completed["type"], "response.completed");

    ws.send_text(
        json!({
            "type": "response.create",
            "response": {
                "model": "bedrock-second-model",
                "input": "bedrock-second-model prewarm",
                "generate": false
            }
        })
        .to_string(),
    )
    .await
    .unwrap();
    let bedrock_error = expect_json_frame(&mut ws).await;
    assert_eq!(bedrock_error["type"], "error");
    assert_eq!(
        bedrock_error["error"]["code"],
        "websocket_route_model_changed"
    );

    ws.send_text(
        json!({
            "type": "response.create",
            "model": "codex-third-model",
            "input": "codex independent turn",
            "stream": true
        })
        .to_string(),
    )
    .await
    .unwrap();
    let codex_error = expect_json_frame(&mut ws).await;
    assert_eq!(codex_error["type"], "error");
    assert_eq!(
        codex_error["error"]["code"],
        "websocket_route_model_changed"
    );
    assert_no_upstream_frame(&mut capture_rx).await;

    let _ = ws.close(None).await;
    proxy.handle.abort();
    upstream.handle.abort();
}

#[tokio::test]
async fn responses_websocket_unsupported_parseable_event_errors_without_closing() {
    let (capture_tx, mut capture_rx) = mpsc::unbounded_channel();
    let upstream = spawn_upstream(capture_tx, UpstreamBehavior::Complete).await;

    let test_state = codex_test_state_with_route("codex-after-unsupported-model");
    let mut state = test_state.state.clone();
    state.runtime.codex_responses_wss_url =
        format!("ws://{}/backend-api/codex/responses", upstream.addr);
    let proxy = spawn_proxy(state).await;

    let mut ws = connect_proxy_ws(&proxy, "/v1/responses").await;
    ws.send_text(json!({ "type": "session.update", "session": {} }).to_string())
        .await
        .unwrap();
    let error = expect_json_frame(&mut ws).await;
    assert_eq!(error["type"], "error");
    assert_eq!(error["error"]["code"], "unsupported_websocket_event");

    ws.send_text(
        json!({
            "type": "response.create",
            "model": "codex-after-unsupported-model",
            "input": "still usable",
            "stream": true
        })
        .to_string(),
    )
    .await
    .unwrap();
    let completed = expect_json_frame(&mut ws).await;
    assert_eq!(completed["type"], "response.completed");
    let captured = expect_first_frame(&mut capture_rx).await;
    let frame: Value = serde_json::from_str(&captured.first_frame).unwrap();
    assert_eq!(frame["input"][0]["content"][0]["text"], "still usable");

    let _ = ws.close(None).await;
    proxy.handle.abort();
    upstream.handle.abort();
}

#[tokio::test]
async fn responses_websocket_in_flight_response_create_errors_without_closing() {
    let (capture_tx, mut capture_rx) = mpsc::unbounded_channel();
    let upstream = spawn_upstream(
        capture_tx,
        UpstreamBehavior::DelayComplete(Duration::from_millis(250)),
    )
    .await;

    let test_state = codex_test_state_with_route("slow-codex-model");
    let mut state = test_state.state.clone();
    state.runtime.codex_responses_wss_url =
        format!("ws://{}/backend-api/codex/responses", upstream.addr);
    let proxy = spawn_proxy(state).await;

    let mut ws = connect_proxy_ws(&proxy, "/v1/responses").await;
    ws.send_text(
        json!({
            "type": "response.create",
            "model": "slow-codex-model",
            "input": "first slow turn",
            "stream": true
        })
        .to_string(),
    )
    .await
    .unwrap();
    let captured = expect_first_frame(&mut capture_rx).await;
    let frame: Value = serde_json::from_str(&captured.first_frame).unwrap();
    assert_eq!(frame["input"][0]["content"][0]["text"], "first slow turn");

    ws.send_text(
        json!({
            "type": "response.create",
            "model": "slow-codex-model",
            "input": "second overlapping turn",
            "stream": true
        })
        .to_string(),
    )
    .await
    .unwrap();
    let error = expect_json_frame(&mut ws).await;
    assert_eq!(error["type"], "error");
    assert_eq!(error["error"]["code"], "response_already_in_flight");

    let completed = expect_json_frame(&mut ws).await;
    assert_eq!(completed["type"], "response.completed");
    assert_no_upstream_frame(&mut capture_rx).await;

    let _ = ws.close(None).await;
    proxy.handle.abort();
    upstream.handle.abort();
}

#[tokio::test]
async fn responses_websocket_accepts_binary_raw_responses_body_for_compatibility() {
    let (capture_tx, mut capture_rx) = mpsc::unbounded_channel();
    let upstream = spawn_upstream(capture_tx, UpstreamBehavior::Complete).await;

    let test_state = codex_test_state_with_route("binary-raw-model");
    let mut state = test_state.state.clone();
    state.runtime.codex_responses_wss_url =
        format!("ws://{}/backend-api/codex/responses", upstream.addr);
    let proxy = spawn_proxy(state).await;

    let mut ws = connect_proxy_ws(&proxy, "/api/provider/openai/v1/responses").await;
    ws.send(SpecterMessage::Binary(Bytes::from(
        json!({
            "model": "binary-raw-model",
            "input": "raw binary body",
            "stream": true
        })
        .to_string(),
    )))
    .await
    .unwrap();

    let captured = expect_first_frame(&mut capture_rx).await;
    let frame: Value = serde_json::from_str(&captured.first_frame).unwrap();
    assert_eq!(frame["type"], "response.create");
    assert_eq!(frame["model"], "gpt-5.5");
    assert_eq!(frame["input"][0]["content"][0]["text"], "raw binary body");
    assert_eq!(frame["stream"], true);

    let _ = ws.close(None).await;
    proxy.handle.abort();
    upstream.handle.abort();
}

#[tokio::test]
async fn responses_websocket_fails_closed_for_non_codex_responses_route_without_upstream() {
    let (capture_tx, mut capture_rx) = mpsc::unbounded_channel();
    let upstream = spawn_upstream(capture_tx, UpstreamBehavior::Complete).await;

    let test_state = codex_test_state_without_route();
    let mut state = test_state.state.clone();
    state.runtime.codex_responses_wss_url =
        format!("ws://{}/backend-api/codex/responses", upstream.addr);
    let proxy = spawn_proxy(state).await;

    let mut ws = connect_proxy_ws(&proxy, "/v1/responses").await;
    ws.send_text(
        json!({
            "type": "response.create",
            "response": {
                "model": "gemini-3.1-flash-lite",
                "input": "must not tunnel to Codex"
            }
        })
        .to_string(),
    )
    .await
    .unwrap();

    let error = expect_proxy_error(&mut ws).await;
    assert!(error.contains("missing credential") || error.contains("websocket_proxy_error"));
    assert_no_upstream_frame(&mut capture_rx).await;

    proxy.handle.abort();
    upstream.handle.abort();
}

#[tokio::test]
async fn responses_websocket_fails_closed_for_malformed_json_without_upstream() {
    let (capture_tx, mut capture_rx) = mpsc::unbounded_channel();
    let upstream = spawn_upstream(capture_tx, UpstreamBehavior::Complete).await;

    let test_state = codex_test_state_with_route("malformed-model");
    let mut state = test_state.state.clone();
    state.runtime.codex_responses_wss_url =
        format!("ws://{}/backend-api/codex/responses", upstream.addr);
    let proxy = spawn_proxy(state).await;

    let mut ws = connect_proxy_ws(&proxy, "/v1/responses").await;
    ws.send_text("{not valid json").await.unwrap();

    let error = expect_proxy_error(&mut ws).await;
    assert!(error.contains("expected") || error.contains("JSON") || error.contains("json"));
    assert_no_upstream_frame(&mut capture_rx).await;

    proxy.handle.abort();
    upstream.handle.abort();
}

#[tokio::test]
async fn non_responses_routes_reject_websocket_handshakes() {
    let test_state = codex_test_state_without_route();
    let proxy = spawn_proxy(test_state.state.clone()).await;
    let client = specter::Client::new().unwrap();

    let result = client
        .websocket(format!("ws://{}/v1/messages", proxy.addr))
        .connect()
        .await;

    assert!(
        result.is_err(),
        "non-Codex WebSocket route must fail closed"
    );
    proxy.handle.abort();
}

#[tokio::test]
async fn responses_websocket_forwards_ping_pong_controls_after_first_frame() {
    let (capture_tx, mut capture_rx) = mpsc::unbounded_channel();
    let upstream = spawn_upstream(capture_tx, UpstreamBehavior::PingThenWaitForPong).await;

    let test_state = codex_test_state_with_route("ping-model");
    let mut state = test_state.state.clone();
    state.runtime.codex_responses_wss_url =
        format!("ws://{}/backend-api/codex/responses", upstream.addr);
    let proxy = spawn_proxy(state).await;

    let mut ws = connect_proxy_ws(&proxy, "/v1/responses").await;
    ws.send_text(
        json!({
            "type": "response.create",
            "response": {
                "model": "ping-model",
                "input": "control frames"
            }
        })
        .to_string(),
    )
    .await
    .unwrap();

    let client_message = ws.next().await.unwrap().unwrap();
    match client_message {
        SpecterMessage::Ping(bytes) => assert_eq!(bytes.as_ref(), b"proxy-ping"),
        other => panic!("expected ping from upstream through proxy, got {other:?}"),
    }

    let captured = expect_first_frame(&mut capture_rx).await;
    let frame: Value = serde_json::from_str(&captured.first_frame).unwrap();
    assert_eq!(frame["model"], "gpt-5.5");
    assert_eq!(frame["stream"], true);
    assert_eq!(frame["input"][0]["content"][0]["text"], "control frames");

    let control = expect_control(&mut capture_rx).await;
    assert_eq!(control, "pong:proxy-ping");

    let _ = ws.close(None).await;
    proxy.handle.abort();
    upstream.handle.abort();
}

#[tokio::test]
async fn responses_websocket_reports_json_error_when_upstream_disconnects_abruptly() {
    let (capture_tx, mut capture_rx) = mpsc::unbounded_channel();
    let upstream = spawn_upstream(capture_tx, UpstreamBehavior::DisconnectAfterFirstFrame).await;

    let test_state = codex_test_state_with_route("disconnect-model");
    let mut state = test_state.state.clone();
    state.runtime.codex_responses_wss_url =
        format!("ws://{}/backend-api/codex/responses", upstream.addr);
    let proxy = spawn_proxy(state).await;

    let mut ws = connect_proxy_ws(&proxy, "/v1/responses").await;
    ws.send_text(
        json!({
            "type": "response.create",
            "response": {
                "model": "disconnect-model",
                "input": "upstream disconnect"
            }
        })
        .to_string(),
    )
    .await
    .unwrap();

    let captured = expect_first_frame(&mut capture_rx).await;
    let frame: Value = serde_json::from_str(&captured.first_frame).unwrap();
    assert_eq!(frame["model"], "gpt-5.5");
    assert_eq!(frame["stream"], true);
    assert_eq!(
        frame["input"][0]["content"][0]["text"],
        "upstream disconnect"
    );

    let error = expect_proxy_error(&mut ws).await;
    assert!(error.contains("\"code\":\"upstream_error\""));
    assert!(
        error.contains("closed before a terminal response event")
            || error.contains("Codex WSS read failed")
    );

    proxy.handle.abort();
    upstream.handle.abort();
}

#[tokio::test]
async fn responses_websocket_reports_json_error_when_upstream_closes_before_terminal() {
    let (capture_tx, mut capture_rx) = mpsc::unbounded_channel();
    let upstream = spawn_upstream(capture_tx, UpstreamBehavior::CloseAfterFirstFrame).await;

    let test_state = codex_test_state_with_route("upstream-close-model");
    let mut state = test_state.state.clone();
    state.runtime.codex_responses_wss_url =
        format!("ws://{}/backend-api/codex/responses", upstream.addr);
    let proxy = spawn_proxy(state).await;

    let mut ws = connect_proxy_ws(&proxy, "/v1/responses").await;
    ws.send_text(
        json!({
            "type": "response.create",
            "response": {
                "model": "upstream-close-model",
                "input": "upstream close"
            }
        })
        .to_string(),
    )
    .await
    .unwrap();

    let captured = expect_first_frame(&mut capture_rx).await;
    let frame: Value = serde_json::from_str(&captured.first_frame).unwrap();
    assert_eq!(frame["model"], "gpt-5.5");
    assert_eq!(frame["stream"], true);
    assert_eq!(frame["input"][0]["content"][0]["text"], "upstream close");

    let error = expect_proxy_error(&mut ws).await;
    assert!(error.contains("\"code\":\"upstream_error\""));
    assert!(error.contains("closed before a terminal response event"));

    proxy.handle.abort();
    upstream.handle.abort();
}

struct SpawnedServer {
    addr: SocketAddr,
    handle: JoinHandle<()>,
}

async fn spawn_proxy(state: AppState) -> SpawnedServer {
    let app = build_router(state);
    spawn_router(app).await
}

async fn spawn_upstream(
    capture_tx: mpsc::UnboundedSender<CapturedEvent>,
    behavior: UpstreamBehavior,
) -> SpawnedServer {
    let app = Router::new()
        .route("/backend-api/codex/responses", get(upstream_ws))
        .with_state((capture_tx, behavior));
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

async fn upstream_ws(
    State((capture_tx, behavior)): State<(mpsc::UnboundedSender<CapturedEvent>, UpstreamBehavior)>,
    headers: HeaderMap,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    let headers = headers
        .iter()
        .filter_map(|(name, value)| {
            value
                .to_str()
                .ok()
                .map(|value| (name.as_str().to_ascii_lowercase(), value.to_string()))
        })
        .collect::<Vec<_>>();
    ws.on_upgrade(move |mut socket| async move {
        let first_frame = match socket.recv().await {
            Some(Ok(AxumMessage::Text(text))) => text,
            Some(Ok(AxumMessage::Binary(bytes))) => {
                String::from_utf8_lossy(bytes.as_ref()).into_owned()
            }
            other => panic!("unexpected upstream websocket message: {other:?}"),
        };
        capture_tx
            .send(CapturedEvent::FirstFrame(CapturedWebSocket {
                headers,
                first_frame,
            }))
            .unwrap();
        match behavior {
            UpstreamBehavior::Complete => {
                socket
                    .send(AxumMessage::Text(
                        r#"{"type":"response.completed","response":{"id":"resp_ws","status":"completed"}}"#.into(),
                    ))
                    .await
                    .unwrap();
            }
            UpstreamBehavior::CloseAfterFirstFrame => {
                socket.send(AxumMessage::Close(None)).await.unwrap();
            }
            UpstreamBehavior::DisconnectAfterFirstFrame => {}
            UpstreamBehavior::DelayComplete(delay) => {
                tokio::time::sleep(delay).await;
                socket
                    .send(AxumMessage::Text(
                        r#"{"type":"response.completed","response":{"id":"resp_ws","status":"completed"}}"#.into(),
                    ))
                    .await
                    .unwrap();
            }
            UpstreamBehavior::PingThenWaitForPong => {
                socket
                    .send(AxumMessage::Ping(b"proxy-ping".to_vec()))
                    .await
                    .unwrap();
                match socket.recv().await {
                    Some(Ok(AxumMessage::Pong(bytes))) => capture_tx
                        .send(CapturedEvent::Control(format!(
                            "pong:{}",
                            String::from_utf8_lossy(&bytes)
                        )))
                        .unwrap(),
                    other => panic!("expected upstream pong, got {other:?}"),
                }
            }
        }
    })
}

fn codex_test_state_with_route(model: &str) -> TestState {
    codex_test_state_with_routes(&[(model, "gpt-5.5")])
}

fn codex_test_state_with_routes(routes: &[(&str, &str)]) -> TestState {
    codex_test_state_with_route_specs(
        &routes
            .iter()
            .map(|(source_model, target_model)| (*source_model, "codex", *target_model))
            .collect::<Vec<_>>(),
    )
}

fn codex_test_state_with_route_specs(routes: &[(&str, &str, &str)]) -> TestState {
    let codex_home = TempDir::new().unwrap();
    let auth_home = TempDir::new().unwrap();
    std::fs::write(
        codex_home.path().join("auth.json"),
        json!({
            "access_token": "access-token",
            "account_id": "account-123"
        })
        .to_string(),
    )
    .unwrap();
    let routes = routes
        .iter()
        .map(|(source_model, provider, target_model)| {
            json!({
                "source": { "model": source_model, "format": "responses" },
                "target": { "provider": provider, "model": target_model, "format": "responses" }
            })
        })
        .collect::<Vec<_>>();
    std::fs::write(
        auth_home.path().join("config.json"),
        json!({
            "routes": routes
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

fn codex_test_state_without_route() -> TestState {
    let codex_home = TempDir::new().unwrap();
    let auth_home = TempDir::new().unwrap();
    std::fs::write(
        codex_home.path().join("auth.json"),
        json!({
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
    TestState {
        _codex_home: codex_home,
        _auth_home: auth_home,
        state,
    }
}

async fn connect_proxy_ws(proxy: &SpawnedServer, path: &str) -> specter::WebSocket {
    specter::Client::new()
        .unwrap()
        .websocket(format!("ws://{}{}", proxy.addr, path))
        .connect()
        .await
        .unwrap()
}

async fn expect_first_frame(
    capture_rx: &mut mpsc::UnboundedReceiver<CapturedEvent>,
) -> CapturedWebSocket {
    match tokio::time::timeout(Duration::from_secs(2), capture_rx.recv())
        .await
        .unwrap()
        .unwrap()
    {
        CapturedEvent::FirstFrame(captured) => captured,
        other => panic!("expected first upstream frame, got {other:?}"),
    }
}

async fn expect_control(capture_rx: &mut mpsc::UnboundedReceiver<CapturedEvent>) -> String {
    match tokio::time::timeout(Duration::from_secs(2), capture_rx.recv())
        .await
        .unwrap()
        .unwrap()
    {
        CapturedEvent::Control(control) => control,
        other => panic!("expected upstream control event, got {other:?}"),
    }
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

async fn expect_proxy_error(ws: &mut specter::WebSocket) -> String {
    let message = tokio::time::timeout(Duration::from_secs(2), ws.next())
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    match message {
        SpecterMessage::Text(text) => {
            assert!(text.contains("\"type\":\"error\""));
            text
        }
        other => panic!("expected proxy error text, got {other:?}"),
    }
}

async fn assert_no_upstream_frame(capture_rx: &mut mpsc::UnboundedReceiver<CapturedEvent>) {
    let result = tokio::time::timeout(Duration::from_millis(200), capture_rx.recv()).await;
    assert!(result.is_err(), "upstream should not receive a frame");
}

fn header(headers: &[(String, String)], name: &str) -> String {
    headers
        .iter()
        .find(|(header_name, _)| header_name == name)
        .map(|(_, value)| value.clone())
        .unwrap_or_else(|| panic!("missing header: {name}"))
}
