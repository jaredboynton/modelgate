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
    upstream::bedrock::{
        build_runtime_invoke_request, mantle_forward_headers, mantle_retry_delay,
        runtime_forward_headers, select_mantle_auth, send_mantle_messages_request,
        send_runtime_invoke_request, MantleMessagesRequest, MantleRetryPolicy,
        RuntimeInvokeRequest, DEFAULT_MANTLE_MAX_ATTEMPTS, MANTLE_MESSAGES_PATH,
    },
    AppState,
};
use wiremock::{
    matchers::{body_json, header, method, path},
    Mock, MockServer, ResponseTemplate,
};

#[tokio::test]
async fn bedrock_mantle_transport_emits_chunks_before_upstream_body_completes() {
    let (server_url, release_second_chunk) = spawn_incremental_mantle_server().await;
    let response = tokio::time::timeout(
        Duration::from_secs(1),
        send_mantle_messages_request(
            &reqwest::Client::new(),
            MantleMessagesRequest {
                url: format!("{server_url}{MANTLE_MESSAGES_PATH}"),
                path: MANTLE_MESSAGES_PATH,
                body: serde_json::json!({
                    "model": "anthropic.claude-haiku-4-5",
                    "messages": [{"role": "user", "content": "ping"}],
                    "max_tokens": 64
                }),
                auth: select_mantle_auth(
                    BedrockAuth::Bearer {
                        token: "fixture-token".into(),
                        source: "test",
                    },
                    "us-east-1",
                ),
                headers: mantle_forward_headers(&HeaderMap::new()),
            },
            MantleRetryPolicy { max_attempts: 1 },
        ),
    )
    .await
    .expect("Mantle transport must return once response headers and first bytes are available")
    .unwrap();

    assert_eq!(response.status, StatusCode::OK);
    let mut chunks = response.body.into_data_stream();
    let first = tokio::time::timeout(Duration::from_secs(1), chunks.next())
        .await
        .expect("first Mantle chunk should be emitted without waiting for the full body")
        .expect("first Mantle chunk should be present")
        .unwrap();
    assert_eq!(first, Bytes::from_static(b"{\"type\":\"message_start\"}\n"));

    let second_before_release =
        tokio::time::timeout(Duration::from_millis(100), chunks.next()).await;
    assert!(
        second_before_release.is_err(),
        "second Mantle chunk should not appear before the mock upstream releases it"
    );

    release_second_chunk.send(()).unwrap();
    let second = tokio::time::timeout(Duration::from_secs(1), chunks.next())
        .await
        .expect("second Mantle chunk should arrive after release")
        .expect("second Mantle chunk should be present")
        .unwrap();
    assert_eq!(
        second,
        Bytes::from_static(b"{\"type\":\"content_block_delta\"}\n")
    );
}

#[tokio::test]
async fn bedrock_mantle_transport_forwards_bearer_and_streams_response_parts() {
    let server = MockServer::start().await;
    let response = ResponseTemplate::new(202)
        .insert_header("content-type", "application/json")
        .insert_header("anthropic-request-id", "req_bedrock_123")
        .insert_header("authorization", "must-not-return")
        .set_body_raw(
            r#"{"id":"msg_123","content":[{"text":"pong"}]}"#,
            "application/json",
        );

    Mock::given(method("POST"))
        .and(path(MANTLE_MESSAGES_PATH))
        .and(header("x-api-key", "fixture-token"))
        .and(header("anthropic-version", "2023-06-01"))
        .and(body_json(serde_json::json!({
            "model": "anthropic.claude-haiku-4-5",
            "messages": [{"role": "user", "content": "ping"}],
            "max_tokens": 64
        })))
        .respond_with(response)
        .mount(&server)
        .await;

    let response = send_mantle_messages_request(
        &reqwest::Client::new(),
        MantleMessagesRequest {
            url: format!("{}{}", server.uri(), MANTLE_MESSAGES_PATH),
            path: MANTLE_MESSAGES_PATH,
            body: serde_json::json!({
                "model": "anthropic.claude-haiku-4-5",
                "messages": [{"role": "user", "content": "ping"}],
                "max_tokens": 64
            }),
            auth: select_mantle_auth(
                BedrockAuth::Bearer {
                    token: "fixture-token".into(),
                    source: "test",
                },
                "us-east-1",
            ),
            headers: mantle_forward_headers(&HeaderMap::new()),
        },
        MantleRetryPolicy { max_attempts: 1 },
    )
    .await
    .unwrap();

    assert_eq!(response.status, StatusCode::ACCEPTED);
    assert_eq!(response.provider, "bedrock");
    assert_eq!(response.headers["content-type"], "application/json");
    assert_eq!(response.headers["anthropic-request-id"], "req_bedrock_123");
    assert!(response.headers.get("authorization").is_none());

    let bytes = to_bytes(response.body, usize::MAX).await.unwrap();
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&bytes).unwrap(),
        serde_json::json!({"id":"msg_123","content":[{"text":"pong"}]})
    );
}

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
        "us.anthropic.claude-sonnet-4-6",
    )
    .unwrap();

    assert!(request.body.get("model").is_none());
    assert!(request.body.get("stream").is_none());
    assert_eq!(request.body["anthropic_version"], "bedrock-2023-05-31");
    assert!(request
        .url
        .ends_with("/model/us.anthropic.claude-sonnet-4-6/invoke"));
    assert!(!request.stream);
}

#[test]
fn bedrock_retry_policy_defaults_to_six_attempts_with_bounded_jitter() {
    assert_eq!(
        MantleRetryPolicy::default().max_attempts,
        DEFAULT_MANTLE_MAX_ATTEMPTS
    );
    assert_eq!(DEFAULT_MANTLE_MAX_ATTEMPTS, 6);
    assert_eq!(mantle_retry_delay(1, 0), Duration::from_millis(50));
    assert_eq!(mantle_retry_delay(2, 0), Duration::from_millis(100));
    assert!(mantle_retry_delay(6, u64::MAX) <= Duration::from_millis(2_000));
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
        "us.anthropic.claude-sonnet-4-6",
    )
    .unwrap();

    assert!(request.body.get("stream").is_none());
    assert!(
        request
            .url
            .ends_with("/model/us.anthropic.claude-sonnet-4-6/invoke-with-response-stream"),
        "Runtime stream=true must select Bedrock invoke-with-response-stream"
    );
    assert!(request.stream);
}

#[tokio::test]
async fn bedrock_runtime_transport_uses_bearer_auth_and_path_model_id() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/model/us.anthropic.claude-sonnet-4-6/invoke"))
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
        RuntimeInvokeRequest {
            url: format!(
                "{}/model/us.anthropic.claude-sonnet-4-6/invoke",
                server.uri()
            ),
            body: serde_json::json!({
                "anthropic_version": "bedrock-2023-05-31",
                "messages": [{"role": "user", "content": "ping"}],
                "max_tokens": 64
            }),
            auth: select_mantle_auth(
                BedrockAuth::Bearer {
                    token: "fixture-token".into(),
                    source: "test",
                },
                "us-east-1",
            ),
            headers: runtime_forward_headers(&HeaderMap::new()),
            stream: false,
        },
        MantleRetryPolicy { max_attempts: 1 },
    )
    .await
    .unwrap();

    assert_eq!(response.status, StatusCode::OK);
    assert_eq!(response.provider, "bedrock");
    let bytes = to_bytes(response.body, usize::MAX).await.unwrap();
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&bytes).unwrap()["model"],
        "claude-sonnet-4-6"
    );
}

#[tokio::test]
async fn bedrock_runtime_stream_transport_decodes_eventstream_incrementally() {
    let (server_url, release_second_chunk) = spawn_incremental_runtime_eventstream_server().await;
    let response = send_runtime_invoke_request(
        &reqwest::Client::new(),
        RuntimeInvokeRequest {
            url: format!(
                "{server_url}/model/us.anthropic.claude-sonnet-4-6/invoke-with-response-stream"
            ),
            body: serde_json::json!({
                "anthropic_version": "bedrock-2023-05-31",
                "messages": [{"role": "user", "content": "ping"}],
                "max_tokens": 64
            }),
            auth: select_mantle_auth(
                BedrockAuth::Bearer {
                    token: "fixture-token".into(),
                    source: "test",
                },
                "us-east-1",
            ),
            headers: runtime_forward_headers(&HeaderMap::new()),
            stream: true,
        },
        MantleRetryPolicy { max_attempts: 1 },
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
        Bytes::from_static(b"data: {\"type\":\"message_start\"}\n\n")
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
        Bytes::from_static(b"data: {\"type\":\"content_block_delta\"}\n\n")
    );
}

async fn spawn_incremental_mantle_server() -> (String, oneshot::Sender<()>) {
    let (release_tx, release_rx) = oneshot::channel();
    let release_rx = Arc::new(Mutex::new(Some(release_rx)));
    let app = Router::new().route(
        MANTLE_MESSAGES_PATH,
        post(move || {
            let release_rx = release_rx.clone();
            async move {
                let release_rx = release_rx
                    .lock()
                    .expect("release receiver mutex")
                    .take()
                    .expect("incremental test server should receive one request");
                let body = stream::once(async {
                    Ok::<_, std::io::Error>(Bytes::from_static(b"{\"type\":\"message_start\"}\n"))
                })
                .chain(stream::once(async move {
                    let _ = release_rx.await;
                    Ok::<_, std::io::Error>(Bytes::from_static(
                        b"{\"type\":\"content_block_delta\"}\n",
                    ))
                }));
                Response::builder()
                    .status(StatusCode::OK)
                    .header("content-type", "application/x-ndjson")
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

async fn spawn_incremental_runtime_eventstream_server() -> (String, oneshot::Sender<()>) {
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
                        Ok::<_, std::io::Error>(Bytes::from(aws_event_stream_chunk(
                            br#"{"type":"content_block_delta"}"#,
                        )))
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

#[tokio::test]
async fn bedrock_mantle_transport_retries_transient_status_before_streaming_final_body() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path(MANTLE_MESSAGES_PATH))
        .respond_with(ResponseTemplate::new(503).set_body_string("try again"))
        .up_to_n_times(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path(MANTLE_MESSAGES_PATH))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"ok": true})))
        .mount(&server)
        .await;

    let response = send_mantle_messages_request(
        &reqwest::Client::new(),
        MantleMessagesRequest {
            url: format!("{}{}", server.uri(), MANTLE_MESSAGES_PATH),
            path: MANTLE_MESSAGES_PATH,
            body: serde_json::json!({"model": "anthropic.claude-haiku-4-5"}),
            auth: select_mantle_auth(
                BedrockAuth::Bearer {
                    token: "fixture-token".into(),
                    source: "test",
                },
                "us-east-1",
            ),
            headers: HeaderMap::new(),
        },
        MantleRetryPolicy { max_attempts: 2 },
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
