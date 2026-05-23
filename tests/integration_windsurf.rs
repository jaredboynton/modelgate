use axum::{
    body::{to_bytes, Bytes},
    http::{header, HeaderMap, HeaderValue, StatusCode},
};
use serde_json::{json, Value};
use std::fs;
use unified_model_proxy_v2::{
    route::{
        chat::chat_completions,
        responses_executor::{execute_responses_request, ExecuteResponsesOptions},
    },
    state::NewResponseStateRecord,
    upstream::windsurf::{connect_envelope, encode_string},
};
use wiremock::{
    matchers::{method, path},
    Mock, MockServer, ResponseTemplate,
};

#[tokio::test]
async fn windsurf_chat_non_stream_posts_connect_proto_and_returns_openai_chat() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/exa.api_server_pb.ApiServerService/GetChatMessage"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(windsurf_text_frame("hello")))
        .expect(1)
        .mount(&server)
        .await;
    let (_temp, state) = windsurf_state(&server);

    let response = chat_completions(
        axum::extract::State(state),
        HeaderMap::new(),
        Bytes::from(
            json!({
                "model": "swe-1.6",
                "messages": [{ "role": "user", "content": "hello" }],
                "stream": false
            })
            .to_string(),
        ),
    )
    .await
    .unwrap();

    assert_eq!(response.provider, "windsurf");
    assert_eq!(response.status, StatusCode::OK);
    let body: Value =
        serde_json::from_slice(&to_bytes(response.body, usize::MAX).await.unwrap()).unwrap();
    assert_eq!(body["model"], "swe-1.6");
    assert_eq!(body["choices"][0]["message"]["content"], "hello");

    let requests = server.received_requests().await.unwrap();
    let body = String::from_utf8_lossy(&requests[0].body);
    assert!(body.contains("fake_windsurf_key"));
    assert!(body.contains("swe-1-6"));
}

#[tokio::test]
async fn windsurf_chat_stream_returns_sse_chunks_and_done() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/exa.api_server_pb.ApiServerService/GetChatMessage"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(windsurf_text_frame("streamed")))
        .expect(1)
        .mount(&server)
        .await;
    let (_temp, state) = windsurf_state(&server);

    let response = chat_completions(
        axum::extract::State(state),
        HeaderMap::new(),
        Bytes::from(
            json!({
                "model": "swe-1.6",
                "messages": [{ "role": "user", "content": "hello" }],
                "stream": true
            })
            .to_string(),
        ),
    )
    .await
    .unwrap();

    let body =
        String::from_utf8(to_bytes(response.body, usize::MAX).await.unwrap().to_vec()).unwrap();
    assert!(body.contains(r#""object":"chat.completion.chunk""#));
    assert!(body.contains(r#""content":"streamed""#));
    assert!(body.contains("data: [DONE]"));
}

#[tokio::test]
async fn windsurf_chat_non_stream_converts_droid_style_tool_tags_to_tool_calls() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/exa.api_server_pb.ApiServerService/GetChatMessage"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(windsurf_text_frame(
            r#"<tool_call>Read{"file_path":"/tmp/README.md"}<tool_call>LS{"directory_path":"/tmp"}"#,
        )))
        .expect(1)
        .mount(&server)
        .await;
    let (_temp, state) = windsurf_state(&server);

    let response = chat_completions(
        axum::extract::State(state),
        HeaderMap::new(),
        Bytes::from(
            json!({
                "model": "swe-1.6",
                "messages": [{ "role": "user", "content": "read the readme" }],
                "tools": [
                    {
                        "type": "function",
                        "function": {
                            "name": "Read",
                            "parameters": { "type": "object" }
                        }
                    },
                    {
                        "type": "function",
                        "function": {
                            "name": "LS",
                            "parameters": { "type": "object" }
                        }
                    }
                ],
                "stream": false
            })
            .to_string(),
        ),
    )
    .await
    .unwrap();

    let body: Value =
        serde_json::from_slice(&to_bytes(response.body, usize::MAX).await.unwrap()).unwrap();
    assert_eq!(body["choices"][0]["finish_reason"], "tool_calls");
    let calls = body["choices"][0]["message"]["tool_calls"]
        .as_array()
        .unwrap();
    assert_eq!(calls[0]["function"]["name"], "Read");
    assert_eq!(
        calls[0]["function"]["arguments"],
        r#"{"file_path":"/tmp/README.md"}"#
    );
    assert_eq!(calls[1]["function"]["name"], "LS");
    assert_eq!(
        calls[1]["function"]["arguments"],
        r#"{"directory_path":"/tmp"}"#
    );
}

#[tokio::test]
async fn windsurf_chat_stream_converts_droid_style_tool_tags_to_tool_calls() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/exa.api_server_pb.ApiServerService/GetChatMessage"))
        .respond_with(
            ResponseTemplate::new(200).set_body_bytes(windsurf_text_frame(
                r#"<tool_call>Read{"file_path":"/tmp/README.md"}"#,
            )),
        )
        .expect(1)
        .mount(&server)
        .await;
    let (_temp, state) = windsurf_state(&server);

    let response = chat_completions(
        axum::extract::State(state),
        HeaderMap::new(),
        Bytes::from(
            json!({
                "model": "swe-1.6",
                "messages": [{ "role": "user", "content": "read the readme" }],
                "tools": [{
                    "type": "function",
                    "function": {
                        "name": "Read",
                        "parameters": { "type": "object" }
                    }
                }],
                "stream": true
            })
            .to_string(),
        ),
    )
    .await
    .unwrap();

    let body =
        String::from_utf8(to_bytes(response.body, usize::MAX).await.unwrap().to_vec()).unwrap();
    assert!(body.contains(r#""tool_calls""#));
    assert!(body.contains(r#""name":"Read""#));
    assert!(body.contains(r#""finish_reason":"tool_calls""#));
    assert!(body.contains("data: [DONE]"));
}

#[tokio::test]
async fn windsurf_chat_droid_profile_converts_assistant_tool_calls_block_to_tool_calls() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/exa.api_server_pb.ApiServerService/GetChatMessage"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(windsurf_text_frame(
            r#"ASSISTANT TOOL_CALLS: [{"id":"call_1","type":"function","function":{"name":"Execute","arguments":"{\"command\":\"git status --short\"}"}}]"#,
        )))
        .expect(1)
        .mount(&server)
        .await;
    let (_temp, state) = windsurf_state(&server);
    let mut headers = HeaderMap::new();
    headers.insert(
        header::USER_AGENT,
        HeaderValue::from_static("factory-cli/0.130.0"),
    );

    let response = chat_completions(
        axum::extract::State(state),
        headers,
        Bytes::from(
            json!({
                "model": "swe-1.6",
                "messages": [{ "role": "user", "content": "check status" }],
                "tools": [{
                    "type": "function",
                    "function": {
                        "name": "Execute",
                        "parameters": {
                            "type": "object",
                            "properties": { "command": { "type": "string" } },
                            "required": ["command"]
                        }
                    }
                }],
                "stream": false
            })
            .to_string(),
        ),
    )
    .await
    .unwrap();

    let body: Value =
        serde_json::from_slice(&to_bytes(response.body, usize::MAX).await.unwrap()).unwrap();
    assert_eq!(body["choices"][0]["finish_reason"], "tool_calls");
    assert_eq!(body["choices"][0]["message"]["content"], "");
    let call = &body["choices"][0]["message"]["tool_calls"][0];
    assert_eq!(call["function"]["name"], "Execute");
    let arguments: Value = serde_json::from_str(call["function"]["arguments"].as_str().unwrap())
        .expect("tool arguments must be JSON");
    assert_eq!(arguments["command"], "git status --short");
    assert_eq!(arguments["riskLevel"], "medium");
    assert_eq!(arguments["riskLevelReason"], "automated proxy invocation");
    assert!(!body.to_string().contains("ASSISTANT TOOL_CALLS"));
}

#[tokio::test]
async fn windsurf_responses_droid_profile_converts_assistant_tool_calls_block_to_function_call() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/exa.api_server_pb.ApiServerService/GetChatMessage"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(windsurf_text_frame(
            r#"ASSISTANT TOOL_CALLS: [{"id":"call_1","type":"function","function":{"name":"Execute","arguments":"{\"command\":\"git status --short\"}"}}]"#,
        )))
        .expect(1)
        .mount(&server)
        .await;
    let (_temp, state) = windsurf_state(&server);
    let mut headers = HeaderMap::new();
    headers.insert(
        header::USER_AGENT,
        HeaderValue::from_static("factory-cli/0.130.0"),
    );

    let response = execute_responses_request(
        &state,
        headers,
        json!({
            "model": "swe-1.6",
            "input": "check status",
            "tools": [{
                "type": "function",
                "name": "Execute",
                "parameters": {
                    "type": "object",
                    "properties": { "command": { "type": "string" } },
                    "required": ["command"]
                }
            }]
        }),
        ExecuteResponsesOptions::default(),
    )
    .await
    .unwrap();

    let body: Value =
        serde_json::from_slice(&to_bytes(response.body, usize::MAX).await.unwrap()).unwrap();
    assert_eq!(body["output"][0]["type"], "function_call");
    assert_eq!(body["output"][0]["name"], "Execute");
    let arguments: Value = serde_json::from_str(body["output"][0]["arguments"].as_str().unwrap())
        .expect("function call arguments must be JSON");
    assert_eq!(arguments["command"], "git status --short");
    assert_eq!(arguments["riskLevel"], "medium");
    assert_eq!(arguments["riskLevelReason"], "automated proxy invocation");
    assert!(!body.to_string().contains("ASSISTANT TOOL_CALLS"));
}

#[tokio::test]
async fn windsurf_responses_tool_call_then_tool_result_continues_with_previous_response_id() {
    let first_server = MockServer::start().await;
    let second_server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/exa.api_server_pb.ApiServerService/GetChatMessage"))
        .respond_with(
            ResponseTemplate::new(200).set_body_bytes(windsurf_text_frame(
                r#"{"action":"tool_call","tool_calls":[{"name":"lookup","arguments":{"q":"x"}}]}"#,
            )),
        )
        .expect(1)
        .mount(&first_server)
        .await;
    Mock::given(method("POST"))
        .and(path("/exa.api_server_pb.ApiServerService/GetChatMessage"))
        .respond_with(
            ResponseTemplate::new(200).set_body_bytes(windsurf_text_frame(
                r#"{"action":"final","content":"done"}"#,
            )),
        )
        .expect(1)
        .mount(&second_server)
        .await;
    let (_temp, mut state) = windsurf_state(&first_server);

    let first = execute_responses_request(
        &state,
        HeaderMap::new(),
        json!({
            "model": "swe-1.6",
            "input": "lookup x",
            "tools": [{
                "type": "function",
                "name": "lookup",
                "parameters": { "type": "object" }
            }]
        }),
        ExecuteResponsesOptions::default(),
    )
    .await
    .unwrap();
    let first_body: Value =
        serde_json::from_slice(&to_bytes(first.body, usize::MAX).await.unwrap()).unwrap();
    let response_id = first_body["id"].as_str().unwrap();
    let call_id = first_body["output"][0]["call_id"].as_str().unwrap();
    assert_eq!(first_body["output"][0]["type"], "function_call");
    assert!(state.continuation_response(response_id).is_some());

    std::sync::Arc::make_mut(&mut state.runtime).windsurf_cloud_base_url = second_server.uri();
    let second = execute_responses_request(
        &state,
        HeaderMap::new(),
        json!({
            "model": "swe-1.6",
            "previous_response_id": response_id,
            "input": [{
                "type": "function_call_output",
                "call_id": call_id,
                "output": "found x"
            }]
        }),
        ExecuteResponsesOptions::default(),
    )
    .await
    .unwrap();
    let second_body: Value =
        serde_json::from_slice(&to_bytes(second.body, usize::MAX).await.unwrap()).unwrap();
    assert_eq!(
        second_body["output"][0]["content"][0]["text"], "done",
        "tool output should continue through the Windsurf tool-planning loop"
    );

    let requests = second_server.received_requests().await.unwrap();
    let second_request = String::from_utf8_lossy(&requests[0].body);
    assert!(second_request.contains("ASSISTANT TOOL_CALLS"));
    assert!(second_request.contains("TOOL RESULT"));
}

#[tokio::test]
async fn windsurf_responses_converts_droid_style_tool_tags_to_function_call() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/exa.api_server_pb.ApiServerService/GetChatMessage"))
        .respond_with(
            ResponseTemplate::new(200).set_body_bytes(windsurf_text_frame(
                r#"<tool_call>Read{"file_path":"/tmp/README.md"}"#,
            )),
        )
        .expect(1)
        .mount(&server)
        .await;
    let (_temp, state) = windsurf_state(&server);

    let response = execute_responses_request(
        &state,
        HeaderMap::new(),
        json!({
            "model": "swe-1.6",
            "input": "read the readme",
            "tools": [{
                "type": "function",
                "name": "Read",
                "parameters": { "type": "object" }
            }]
        }),
        ExecuteResponsesOptions::default(),
    )
    .await
    .unwrap();

    let body: Value =
        serde_json::from_slice(&to_bytes(response.body, usize::MAX).await.unwrap()).unwrap();
    assert_eq!(body["output"][0]["type"], "function_call");
    assert_eq!(body["output"][0]["name"], "Read");
    assert_eq!(
        body["output"][0]["arguments"],
        r#"{"file_path":"/tmp/README.md"}"#
    );
    let response_id = body["id"].as_str().unwrap();
    assert!(state.continuation_response(response_id).is_some());
}

#[tokio::test]
async fn windsurf_responses_rejects_cross_provider_previous_response_id_before_network() {
    let server = MockServer::start().await;
    let (_temp, state) = windsurf_state(&server);
    state.remember_response_for_continuation(NewResponseStateRecord {
        route: "responses".into(),
        provider: "cursor".into(),
        upstream_model: "composer-2-fast".into(),
        upstream_response_id: "resp_cursor".into(),
        adapter_response_id: "resp_cursor".into(),
        conversation_id: None,
        raw_response: json!({ "id": "resp_cursor", "output": [] }),
        raw_input_items: json!("prior"),
        upstream_codex_minted: false,
    });

    let error = match execute_responses_request(
        &state,
        HeaderMap::new(),
        json!({
            "model": "swe-1.6",
            "previous_response_id": "resp_cursor",
            "input": "continue"
        }),
        ExecuteResponsesOptions::default(),
    )
    .await
    {
        Ok(_) => panic!("cross-provider previous_response_id must fail"),
        Err(error) => error,
    };

    assert_eq!(
        error.code(),
        Some("previous_response_target_format_mismatch")
    );
    assert!(server.received_requests().await.unwrap().is_empty());
}

fn windsurf_state(server: &MockServer) -> (tempfile::TempDir, unified_model_proxy_v2::AppState) {
    let temp = tempfile::tempdir().unwrap();
    let auth_home = temp.path().join("ump");
    fs::create_dir_all(&auth_home).unwrap();
    fs::write(
        auth_home.join("auth.json"),
        r#"{ "windsurf": { "api_key": "fake_windsurf_key" } }"#,
    )
    .unwrap();
    let mut state =
        unified_model_proxy_v2::AppState::for_tests(temp.path().join("codex"), auth_home);
    std::sync::Arc::make_mut(&mut state.runtime).windsurf_cloud_base_url = server.uri();
    (temp, state)
}

fn windsurf_text_frame(text: &str) -> Vec<u8> {
    connect_envelope(&encode_string(3, text))
}
