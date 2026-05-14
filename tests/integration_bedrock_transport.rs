use axum::{
    body::to_bytes,
    http::{HeaderMap, StatusCode},
};
use tempfile::TempDir;
use unified_model_proxy_v2::{
    auth::bedrock::BedrockAuth,
    upstream::bedrock::{
        build_runtime_invoke_request, mantle_forward_headers, runtime_forward_headers,
        select_mantle_auth, send_mantle_messages_request, send_runtime_invoke_request,
        MantleMessagesRequest, MantleRetryPolicy, RuntimeInvokeRequest, MANTLE_MESSAGES_PATH,
    },
    AppState,
};
use wiremock::{
    matchers::{body_json, header, method, path},
    Mock, MockServer, ResponseTemplate,
};

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
            "stream": true,
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
