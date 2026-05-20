use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use axum::{
    body::{to_bytes, Body},
    http::{HeaderMap, StatusCode},
    response::Response,
    routing::post,
    Router,
};
use base64::{engine::general_purpose::STANDARD as BASE64_STANDARD, Engine as _};
use bytes::Bytes;
use futures::{stream, StreamExt};
use tempfile::TempDir;
use tokio::{net::TcpListener, sync::oneshot};
use unified_model_proxy_v2::{
    auth::bedrock::BedrockAuth,
    error::AppError,
    upstream::bedrock::{
        bedrock_retry_delay, build_runtime_invoke_request, resolve_bedrock_runtime_model_id,
        runtime_forward_headers, select_bedrock_runtime_auth, send_runtime_invoke_request,
        BedrockRetryPolicy, BedrockRuntimeInvokeRequest, DEFAULT_BEDROCK_MAX_ATTEMPTS,
    },
    AppState,
};
use wiremock::{
    matchers::{body_json, header, method, path},
    Mock, MockServer, ResponseTemplate,
};

#[test]
fn bedrock_runtime_request_removes_responses_only_fields() {
    let codex_home = TempDir::new().unwrap();
    let auth_home = TempDir::new().unwrap();
    std::fs::write(
        auth_home.path().join("auth.json"),
        serde_json::json!({ "bedrock": { "bearer": "fixture-token" } }).to_string(),
    )
    .unwrap();
    let state = AppState::for_tests(codex_home.path().into(), auth_home.path().into());

    let request = build_runtime_invoke_request(
        &state,
        serde_json::json!({
            "model": "claude-sonnet-4-6",
            "stream": false,
            "messages": [{"role": "user", "content": "ping"}],
            "max_tokens": 64
        }),
        &HeaderMap::new(),
        "global.anthropic.claude-sonnet-4-6",
    )
    .unwrap();

    assert!(request.body.get("model").is_none());
    assert!(request.body.get("stream").is_none());
    assert_eq!(request.body["anthropic_version"], "bedrock-2023-05-31");
    assert!(request
        .url
        .ends_with("/model/global.anthropic.claude-sonnet-4-6/invoke"));
    assert!(!request.stream);
}

#[test]
fn bedrock_retry_policy_defaults_to_six_attempts_with_bounded_jitter() {
    assert_eq!(
        BedrockRetryPolicy::default().max_attempts,
        DEFAULT_BEDROCK_MAX_ATTEMPTS
    );
    assert_eq!(DEFAULT_BEDROCK_MAX_ATTEMPTS, 6);
    assert_eq!(bedrock_retry_delay(1, 0), Duration::from_millis(50));
    assert_eq!(bedrock_retry_delay(2, 0), Duration::from_millis(100));
    assert!(bedrock_retry_delay(6, u64::MAX) <= Duration::from_millis(2_000));
}

#[test]
fn bedrock_runtime_stream_true_uses_invoke_with_response_stream_endpoint() {
    let codex_home = TempDir::new().unwrap();
    let auth_home = TempDir::new().unwrap();
    std::fs::write(
        auth_home.path().join("auth.json"),
        serde_json::json!({ "bedrock": { "bearer": "fixture-token" } }).to_string(),
    )
    .unwrap();
    let state = AppState::for_tests(codex_home.path().into(), auth_home.path().into());

    let request = build_runtime_invoke_request(
        &state,
        serde_json::json!({
            "model": "claude-sonnet-4-6",
            "stream": true,
            "messages": [{"role": "user", "content": "ping"}],
            "max_tokens": 64
        }),
        &HeaderMap::new(),
        "global.anthropic.claude-sonnet-4-6",
    )
    .unwrap();

    assert!(request.body.get("stream").is_none());
    assert!(
        request
            .url
            .ends_with("/model/global.anthropic.claude-sonnet-4-6/invoke-with-response-stream"),
        "Runtime stream=true must select Bedrock invoke-with-response-stream"
    );
    assert!(request.stream);
}

#[tokio::test]
async fn bedrock_runtime_transport_uses_bearer_auth_and_path_model_id() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/model/global.anthropic.claude-sonnet-4-6/invoke"))
        .and(header("authorization", "Bearer fixture-token"))
        .and(header("content-type", "application/json"))
        .and(header("accept", "application/json"))
        .and(body_json(serde_json::json!({
            "anthropic_version": "bedrock-2023-05-31",
            "messages": [{"role": "user", "content": "ping"}],
            "max_tokens": 64
        })))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "application/json")
                .insert_header("x-amzn-requestid", "bedrock-req-id")
                .insert_header("authorization", "must-be-stripped")
                .set_body_json(serde_json::json!({
                    "id": "msg_runtime_123",
                    "type": "message",
                    "role": "assistant",
                    "model": "claude-sonnet-4-6",
                    "content": [{"type": "text", "text": "pong"}],
                    "usage": {"input_tokens": 1, "output_tokens": 1}
                })),
        )
        .mount(&server)
        .await;

    let response = send_runtime_invoke_request(
        &reqwest::Client::new(),
        BedrockRuntimeInvokeRequest {
            url: format!(
                "{}/model/global.anthropic.claude-sonnet-4-6/invoke",
                server.uri()
            ),
            body: serde_json::json!({
                "anthropic_version": "bedrock-2023-05-31",
                "messages": [{"role": "user", "content": "ping"}],
                "max_tokens": 64
            }),
            auth: select_bedrock_runtime_auth(
                BedrockAuth::Bearer {
                    token: "fixture-token".into(),
                    source: "test",
                },
                "us-west-2",
            ),
            headers: runtime_forward_headers(&HeaderMap::new(), false),
            stream: false,
        },
        BedrockRetryPolicy { max_attempts: 1 },
    )
    .await
    .unwrap();

    assert_eq!(response.status, StatusCode::OK);
    assert_eq!(response.provider, "bedrock");
    assert_eq!(
        response.headers.get("x-amzn-requestid").unwrap(),
        "bedrock-req-id"
    );
    assert!(response.headers.get("authorization").is_none());

    let bytes = to_bytes(response.body, usize::MAX).await.unwrap();
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&bytes).unwrap()["model"],
        "claude-sonnet-4-6"
    );
}

#[tokio::test]
async fn bedrock_runtime_stream_transport_decodes_eventstream_incrementally() {
    let (server_url, release_second_chunk) =
        spawn_incremental_runtime_eventstream_server(false).await;
    let response = send_runtime_invoke_request(
        &reqwest::Client::new(),
        BedrockRuntimeInvokeRequest {
            url: format!(
                "{server_url}/model/global.anthropic.claude-sonnet-4-6/invoke-with-response-stream"
            ),
            body: serde_json::json!({
                "anthropic_version": "bedrock-2023-05-31",
                "messages": [{"role": "user", "content": "ping"}],
                "max_tokens": 64
            }),
            auth: select_bedrock_runtime_auth(
                BedrockAuth::Bearer {
                    token: "fixture-token".into(),
                    source: "test",
                },
                "us-west-2",
            ),
            headers: runtime_forward_headers(&HeaderMap::new(), true),
            stream: true,
        },
        BedrockRetryPolicy { max_attempts: 1 },
    )
    .await
    .unwrap();

    assert_eq!(response.status, StatusCode::OK);
    assert_eq!(response.headers["content-type"], "text/event-stream");
    let mut chunks = response.body.into_data_stream();
    let first = tokio::time::timeout(Duration::from_secs(1), chunks.next())
        .await
        .expect("first Runtime stream chunk should be decoded immediately")
        .expect("first Runtime stream chunk should be present")
        .unwrap();
    assert_eq!(
        first,
        Bytes::from_static(b"event: message_start\ndata: {\"type\":\"message_start\"}\n\n")
    );

    let second_before_release =
        tokio::time::timeout(Duration::from_millis(100), chunks.next()).await;
    assert!(
        second_before_release.is_err(),
        "second Runtime chunk should not appear before the mock upstream releases it"
    );

    release_second_chunk.send(()).unwrap();
    let second = tokio::time::timeout(Duration::from_secs(1), chunks.next())
        .await
        .expect("second Runtime stream chunk should arrive after release")
        .expect("second Runtime stream chunk should be present")
        .unwrap();
    assert_eq!(
        second,
        Bytes::from_static(
            b"event: content_block_delta\ndata: {\"type\":\"content_block_delta\"}\n\n"
        )
    );

    let third = tokio::time::timeout(Duration::from_secs(1), chunks.next())
        .await
        .expect("third Runtime stream chunk should arrive")
        .expect("third Runtime stream chunk should be present")
        .unwrap();
    assert_eq!(
        third,
        Bytes::from_static(b"event: message_stop\ndata: {\"type\":\"message_stop\"}\n\n")
    );
}

#[tokio::test]
async fn bedrock_runtime_stream_premature_eof_fails() {
    let (server_url, release_second_chunk) =
        spawn_incremental_runtime_eventstream_server(true).await; // true = premature EOF
    let response = send_runtime_invoke_request(
        &reqwest::Client::new(),
        BedrockRuntimeInvokeRequest {
            url: format!(
                "{server_url}/model/global.anthropic.claude-sonnet-4-6/invoke-with-response-stream"
            ),
            body: serde_json::json!({
                "anthropic_version": "bedrock-2023-05-31",
                "messages": [{"role": "user", "content": "ping"}],
                "max_tokens": 64
            }),
            auth: select_bedrock_runtime_auth(
                BedrockAuth::Bearer {
                    token: "fixture-token".into(),
                    source: "test",
                },
                "us-west-2",
            ),
            headers: runtime_forward_headers(&HeaderMap::new(), true),
            stream: true,
        },
        BedrockRetryPolicy { max_attempts: 1 },
    )
    .await
    .unwrap();

    assert_eq!(response.status, StatusCode::OK);
    let mut chunks = response.body.into_data_stream();
    let first = chunks.next().await.unwrap().unwrap();
    assert_eq!(
        first,
        Bytes::from_static(b"event: message_start\ndata: {\"type\":\"message_start\"}\n\n")
    );

    release_second_chunk.send(()).unwrap();
    // Next chunk should be an error because the server terminates without emitting message_stop event
    let second = chunks.next().await.unwrap();
    assert!(second.is_err());
    let err = second.unwrap_err();
    assert!(err
        .to_string()
        .contains("premature EOF before message_stop event"));
}

#[tokio::test]
async fn bedrock_runtime_transport_retries_transient_status() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/model/global.anthropic.claude-sonnet-4-6/invoke"))
        .respond_with(ResponseTemplate::new(503).set_body_string("try again"))
        .up_to_n_times(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/model/global.anthropic.claude-sonnet-4-6/invoke"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"ok": true})))
        .mount(&server)
        .await;

    let response = send_runtime_invoke_request(
        &reqwest::Client::new(),
        BedrockRuntimeInvokeRequest {
            url: format!(
                "{}/model/global.anthropic.claude-sonnet-4-6/invoke",
                server.uri()
            ),
            body: serde_json::json!({"anthropic_version": "bedrock-2023-05-31"}),
            auth: select_bedrock_runtime_auth(
                BedrockAuth::Bearer {
                    token: "fixture-token".into(),
                    source: "test",
                },
                "us-west-2",
            ),
            headers: HeaderMap::new(),
            stream: false,
        },
        BedrockRetryPolicy { max_attempts: 2 },
    )
    .await
    .unwrap();

    assert_eq!(response.status, StatusCode::OK);
    let bytes = to_bytes(response.body, usize::MAX).await.unwrap();
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&bytes).unwrap(),
        serde_json::json!({"ok": true})
    );
}

#[test]
fn bedrock_runtime_unsupported_model_fails_closed() {
    let err = resolve_bedrock_runtime_model_id("unsupported-claude-model").unwrap_err();
    assert!(matches!(err, AppError::ModelNotSupported(_)));
}

async fn spawn_incremental_runtime_eventstream_server(
    premature_eof: bool,
) -> (String, oneshot::Sender<()>) {
    let (release_tx, release_rx) = oneshot::channel();
    let release_rx = Arc::new(Mutex::new(Some(release_rx)));
    let app = Router::new().route(
        "/model/:model/invoke-with-response-stream",
        post(move || {
            let release_rx = release_rx.clone();
            async move {
                let release_rx = release_rx
                    .lock()
                    .expect("release receiver mutex")
                    .take()
                    .expect("incremental Runtime test server should receive one request");
                let first = aws_event_stream_chunk(br#"{"type":"message_start"}"#);
                let body = stream::once(async { Ok::<_, std::io::Error>(Bytes::from(first)) })
                    .chain(stream::once(async move {
                        let _ = release_rx.await;
                        if premature_eof {
                            // Do not send terminal chunk, just finish the stream
                            Ok::<_, std::io::Error>(Bytes::new())
                        } else {
                            let second =
                                aws_event_stream_chunk(br#"{"type":"content_block_delta"}"#);
                            let third = aws_event_stream_chunk(br#"{"type":"message_stop"}"#);
                            let mut combined = second;
                            combined.extend_from_slice(&third);
                            Ok::<_, std::io::Error>(Bytes::from(combined))
                        }
                    }));
                Response::builder()
                    .status(StatusCode::OK)
                    .header("content-type", "application/vnd.amazon.eventstream")
                    .body(Body::from_stream(body))
                    .unwrap()
            }
        }),
    );
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    (format!("http://{addr}"), release_tx)
}

fn aws_event_stream_chunk(payload: &[u8]) -> Vec<u8> {
    let encoded = BASE64_STANDARD.encode(payload);
    let payload = serde_json::to_vec(&serde_json::json!({ "bytes": encoded })).unwrap();
    let headers = aws_event_stream_string_header(":message-type", "event")
        .into_iter()
        .chain(aws_event_stream_string_header(":event-type", "chunk"))
        .collect::<Vec<_>>();
    let total_len = 12 + headers.len() + payload.len() + 4;
    let mut message = Vec::with_capacity(total_len);
    message.extend_from_slice(&(total_len as u32).to_be_bytes());
    message.extend_from_slice(&(headers.len() as u32).to_be_bytes());
    message.extend_from_slice(&0_u32.to_be_bytes());
    message.extend_from_slice(&headers);
    message.extend_from_slice(&payload);
    message.extend_from_slice(&0_u32.to_be_bytes());
    message
}

fn aws_event_stream_string_header(name: &str, value: &str) -> Vec<u8> {
    let mut out = Vec::new();
    out.push(name.len() as u8);
    out.extend_from_slice(name.as_bytes());
    out.push(7);
    out.extend_from_slice(&(value.len() as u16).to_be_bytes());
    out.extend_from_slice(value.as_bytes());
    out
}
