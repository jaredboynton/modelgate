use std::sync::Arc;

use bytes::Bytes;
use http::{HeaderMap, HeaderValue, Method, StatusCode};
use unified_model_proxy_v2::{
    auth::bedrock::BedrockAuth,
    error::AppError,
    upstream::{
        bedrock::{
            resolve_bedrock_runtime_model_id, runtime_forward_headers, runtime_invoke_url,
            runtime_invoke_with_response_stream_url, select_bedrock_runtime_auth,
            should_retry_status, BedrockRuntimeAuthSelection,
        },
        google::{
            build_google_generate_content_request_with_base_url,
            build_google_generate_content_request_with_headers, build_google_request,
            build_google_stream_generate_content_request_with_headers,
            forward_generate_content_direct_request, rewrite_google_path, send_google_direct,
            translate_bedrock_messages_to_google_response, translate_google_to_bedrock_messages,
            GoogleRequest,
        },
    },
    AppState,
};

fn state_with_google_key(key: Option<&str>) -> AppState {
    let codex_home = tempfile::tempdir().unwrap();
    let auth_home = tempfile::tempdir().unwrap();
    let mut state = AppState::for_tests(
        codex_home.path().to_path_buf(),
        auth_home.path().to_path_buf(),
    );
    state.google_api_key = key.map(Arc::<str>::from);
    state.bedrock_region = Arc::<str>::from("eu-west-1");
    state
}

#[test]
fn bedrock_runtime_url_model_and_auth_helpers_are_fixtureable() {
    assert_eq!(
        runtime_invoke_url("us-west-2", "global.anthropic.claude-sonnet-4-6"),
        "https://bedrock-runtime.us-west-2.amazonaws.com/model/global.anthropic.claude-sonnet-4-6/invoke"
    );
    assert_eq!(
        runtime_invoke_with_response_stream_url("us-west-2", "global.anthropic.claude-sonnet-4-6"),
        "https://bedrock-runtime.us-west-2.amazonaws.com/model/global.anthropic.claude-sonnet-4-6/invoke-with-response-stream"
    );
    assert_eq!(
        resolve_bedrock_runtime_model_id("anthropic/claude-haiku-4-5-20251001").unwrap(),
        "global.anthropic.claude-haiku-4-5-20251001-v1:0"
    );
    assert_eq!(
        resolve_bedrock_runtime_model_id("anthropic/claude-sonnet-4-6").unwrap(),
        "global.anthropic.claude-sonnet-4-6"
    );
    assert_eq!(
        resolve_bedrock_runtime_model_id("claude-opus-4-6").unwrap(),
        "global.anthropic.claude-opus-4-6-v1"
    );

    let auth = select_bedrock_runtime_auth(
        BedrockAuth::Bearer {
            token: "test-token".into(),
            source: "bearer_file",
        },
        "us-west-2",
    );
    assert_eq!(
        auth,
        BedrockRuntimeAuthSelection::Header {
            name: "authorization",
            value: "test-token".into(),
            source: "bearer_file"
        }
    );
}

#[test]
fn bedrock_forward_headers_default_anthropic_version_and_preserve_safe_headers() {
    let mut headers = axum::http::HeaderMap::new();
    headers.insert("accept", "text/event-stream".parse().unwrap());
    headers.insert("x-amzn-bedrock-trace", "true".parse().unwrap());

    // Stream case:
    let forwarded = runtime_forward_headers(&headers, true);
    assert_eq!(forwarded["accept"], "application/vnd.amazon.eventstream");
    assert_eq!(forwarded["x-amzn-bedrock-accept"], "application/json");
    assert_eq!(forwarded["x-amzn-bedrock-trace"], "true");
    assert!(forwarded.get("authorization").is_none());

    // Non-stream case:
    let forwarded_non_stream = runtime_forward_headers(&headers, false);
    assert_eq!(forwarded_non_stream["accept"], "application/json");
    assert_eq!(forwarded_non_stream["x-amzn-bedrock-trace"], "true");
    assert!(forwarded_non_stream.get("x-amzn-bedrock-accept").is_none());
}

#[test]
fn bedrock_retry_policy_marks_transient_statuses_only() {
    assert!(should_retry_status(
        axum::http::StatusCode::TOO_MANY_REQUESTS
    ));
    assert!(should_retry_status(
        axum::http::StatusCode::INTERNAL_SERVER_ERROR
    ));
    assert!(should_retry_status(axum::http::StatusCode::BAD_GATEWAY));
    assert!(!should_retry_status(axum::http::StatusCode::BAD_REQUEST));
    assert!(!should_retry_status(axum::http::StatusCode::UNAUTHORIZED));
}

#[test]
fn google_request_rewrites_amp_path_and_requires_api_key_header() {
    let path = "/api/provider/google/v1beta1/publishers/google/models/gemini-3-flash-preview:generateContent?alt=sse";
    assert_eq!(
        rewrite_google_path(path).unwrap(),
        "https://generativelanguage.googleapis.com/v1beta/models/gemini-3-flash-preview:generateContent?alt=sse"
    );

    let missing = build_google_request(&state_with_google_key(None), path).unwrap_err();
    assert!(matches!(
        missing,
        AppError::MissingCredential("GOOGLE_API_KEY")
    ));

    let request = build_google_request(&state_with_google_key(Some("fixture-key")), path).unwrap();
    assert_eq!(
        request.url,
        "https://generativelanguage.googleapis.com/v1beta/models/gemini-3-flash-preview:generateContent?alt=sse"
    );
    assert_eq!(
        request.headers.get("x-goog-api-key").unwrap(),
        "fixture-key"
    );
}

#[test]
fn google_generate_content_request_builder_targets_native_endpoint() {
    let request = build_google_generate_content_request_with_headers(
        "gemini-3.1-flash-lite",
        HeaderMap::new(),
        "fixture-key".to_string(),
    )
    .unwrap();

    assert_eq!(
        request.url,
        "https://generativelanguage.googleapis.com/v1beta/models/gemini-3.1-flash-lite:generateContent"
    );
    assert_eq!(request.headers["x-goog-api-key"], "fixture-key");
    assert!(request.url.contains(":generateContent"));
    assert!(!request.url.contains("anthropic"));
}

#[test]
fn google_stream_generate_content_request_builder_targets_sse_endpoint() {
    let mut headers = HeaderMap::new();
    headers.insert(
        "authorization",
        HeaderValue::from_static("Bearer caller-secret"),
    );
    headers.insert("content-type", HeaderValue::from_static("application/json"));

    let request = build_google_stream_generate_content_request_with_headers(
        "gemini-3.1-flash-lite",
        headers,
        "fixture-key".to_string(),
    )
    .unwrap();

    assert_eq!(
        request.url,
        "https://generativelanguage.googleapis.com/v1beta/models/gemini-3.1-flash-lite:streamGenerateContent?alt=sse"
    );
    assert_eq!(request.headers["x-goog-api-key"], "fixture-key");
    assert!(request.headers.get("authorization").is_none());
    assert!(!request.url.contains("aiplatform.googleapis.com"));
}

#[tokio::test]
async fn google_direct_transport_preserves_url_headers_and_body() {
    use wiremock::{
        matchers::{body_string_contains, header, method, path},
        Mock, MockServer, ResponseTemplate,
    };

    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(
            "/v1beta/models/gemini-3-flash-preview:generateContent",
        ))
        .and(header("x-goog-api-key", "fixture-key"))
        .and(header("content-type", "application/json"))
        .and(body_string_contains("hello"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "application/json")
                .set_body_raw(r#"{"candidates":[]}"#, "application/json"),
        )
        .mount(&server)
        .await;

    let body = Bytes::from_static(br#"{"contents":[{"parts":[{"text":"hello"}]}]}"#);
    let mut headers = HeaderMap::new();
    headers.insert("x-goog-api-key", HeaderValue::from_static("fixture-key"));
    headers.insert("content-type", HeaderValue::from_static("application/json"));

    let response = send_google_direct(
        &state_with_google_key(Some("fixture-key")),
        Method::POST,
        GoogleRequest {
            url: format!(
                "{}/v1beta/models/gemini-3-flash-preview:generateContent",
                server.uri()
            ),
            headers,
        },
        body.clone(),
        false,
    )
    .await
    .unwrap();
    assert_eq!(response.status, StatusCode::OK);
}

#[tokio::test]
async fn google_generate_content_direct_helper_does_not_fallback_on_google_error() {
    use axum::body::to_bytes;
    use wiremock::{
        matchers::{body_string_contains, header, method, path},
        Mock, MockServer, ResponseTemplate,
    };

    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1beta/models/gemini-3.1-flash-lite:generateContent"))
        .and(header("x-goog-api-key", "fixture-key"))
        .and(body_string_contains("hello"))
        .respond_with(
            ResponseTemplate::new(503)
                .insert_header("content-type", "application/json")
                .set_body_raw(
                    r#"{"error":{"status":"UNAVAILABLE","message":"google down"}}"#,
                    "application/json",
                ),
        )
        .mount(&server)
        .await;

    let request = build_google_generate_content_request_with_base_url(
        &server.uri(),
        "gemini-3.1-flash-lite",
        HeaderMap::new(),
        "fixture-key".to_string(),
    )
    .unwrap();
    let response = forward_generate_content_direct_request(
        &state_with_google_key(Some("fixture-key")),
        request,
        Bytes::from_static(br#"{"contents":[{"parts":[{"text":"hello"}]}]}"#),
    )
    .await
    .unwrap();

    assert_eq!(response.provider, "google");
    assert_eq!(response.status, StatusCode::SERVICE_UNAVAILABLE);
    let body = to_bytes(response.body, usize::MAX).await.unwrap();
    assert_eq!(
        body,
        Bytes::from_static(br#"{"error":{"status":"UNAVAILABLE","message":"google down"}}"#)
    );
}

#[test]
fn google_body_can_be_translated_to_minimal_bedrock_messages_shape() {
    let google = serde_json::json!({
        "systemInstruction": {
            "parts": [{"text": "stay concise"}]
        },
        "contents": [
            {
                "role": "user",
                "parts": [{"text": "ping"}, {"inlineData": {"mimeType": "image/png"}}]
            },
            {
                "role": "model",
                "parts": [{"text": "pong"}]
            }
        ],
        "generationConfig": {
            "maxOutputTokens": 77
        }
    });

    let bedrock =
        translate_google_to_bedrock_messages(google, "anthropic.claude-haiku-4-5").unwrap();
    assert_eq!(bedrock["model"], "anthropic.claude-haiku-4-5");
    assert_eq!(bedrock["max_tokens"], 77);
    assert_eq!(bedrock["system"], "stay concise");
    assert_eq!(bedrock["messages"][0]["role"], "user");
    assert_eq!(bedrock["messages"][0]["content"][0]["text"], "ping");
    assert_eq!(bedrock["messages"][1]["role"], "assistant");
    assert_eq!(bedrock["messages"][1]["content"][0]["text"], "pong");
}

#[test]
fn bedrock_messages_response_can_be_translated_to_gemini_shape() {
    let bedrock = serde_json::json!({
        "content": [
            { "type": "text", "text": "hello" },
            { "type": "text", "text": "world" }
        ],
        "stop_reason": "max_tokens",
        "usage": {
            "input_tokens": 9,
            "output_tokens": 11
        }
    });

    let google = translate_bedrock_messages_to_google_response(bedrock).unwrap();
    assert_eq!(google["candidates"][0]["content"]["role"], "model");
    assert_eq!(
        google["candidates"][0]["content"]["parts"][0]["text"],
        "hello"
    );
    assert_eq!(google["candidates"][0]["finishReason"], "MAX_TOKENS");
    assert_eq!(google["usageMetadata"]["promptTokenCount"], 9);
    assert_eq!(google["usageMetadata"]["candidatesTokenCount"], 11);
}
