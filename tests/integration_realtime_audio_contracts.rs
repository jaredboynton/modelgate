use std::fs;

use axum::{
    body::{to_bytes, Body, Bytes},
    http::{header, HeaderMap, HeaderValue, Method, Request, StatusCode},
};
use serde_json::Value;
use tempfile::TempDir;
use tower::ServiceExt;
use unified_model_proxy_v2::{
    build_router,
    upstream::{openai_audio, openai_public, openai_realtime},
    AppState, UpstreamResponse,
};
use wiremock::{
    matchers::{body_bytes, header as wiremock_header, method, path},
    Mock, MockServer, ResponseTemplate,
};

const UNSUPPORTED_FEATURE_MESSAGE: &str =
    "This route family is not supported by the Codex OAuth public OpenAI facade.";
const UNSUPPORTED_ROUTE_MESSAGE: &str =
    "This OpenAI route is not implemented for the Codex OAuth public OpenAI facade.";
const MISSING_CODEX_AUTH_MESSAGE: &str = "Missing Codex OAuth credentials at ~/.codex/auth.json.";

fn test_state(codex_home: &TempDir, auth_home: &TempDir) -> AppState {
    AppState::for_tests(
        codex_home.path().to_path_buf(),
        auth_home.path().to_path_buf(),
    )
}

fn write_codex_auth(codex_home: &TempDir, access_token: &str, refresh_token: Option<&str>) {
    fs::write(
        codex_home.path().join("auth.json"),
        serde_json::json!({
            "tokens": {
                "access_token": access_token,
                "refresh_token": refresh_token,
                "account_id": "acct-test"
            }
        })
        .to_string(),
    )
    .unwrap();
}

async fn request_json(method: &str, path: &str, body: Option<Value>) -> (StatusCode, Value) {
    let codex_home = tempfile::tempdir().unwrap();
    let auth_home = tempfile::tempdir().unwrap();
    request_json_with_state(test_state(&codex_home, &auth_home), method, path, body).await
}

async fn request_json_with_state(
    state: AppState,
    method: &str,
    path: &str,
    body: Option<Value>,
) -> (StatusCode, Value) {
    let app = build_router(state);
    let request = Request::builder()
        .method(method)
        .uri(path)
        .header("content-type", "application/json")
        .body(match body {
            Some(value) => Body::from(value.to_string()),
            None => Body::empty(),
        })
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    let status = response.status();
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body = serde_json::from_slice(&bytes).unwrap_or_else(|err| {
        panic!(
            "expected JSON response for {method} {path}, got {err}: {}",
            String::from_utf8_lossy(&bytes)
        )
    });
    (status, body)
}

async fn request_bytes_with_headers(
    state: AppState,
    method: &str,
    path: &str,
    headers: &[(&str, &str)],
    body: Vec<u8>,
) -> (StatusCode, Value) {
    let app = build_router(state);
    let mut builder = Request::builder().method(method).uri(path);
    for (name, value) in headers {
        builder = builder.header(*name, *value);
    }
    let request = builder.body(Body::from(body)).unwrap();

    let response = app.oneshot(request).await.unwrap();
    let status = response.status();
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body = serde_json::from_slice(&bytes).unwrap_or_else(|err| {
        panic!(
            "expected JSON response for {method} {path}, got {err}: {}",
            String::from_utf8_lossy(&bytes)
        )
    });
    (status, body)
}

async fn upstream_json(response: UpstreamResponse) -> (StatusCode, Value) {
    let status = response.status;
    let bytes = to_bytes(response.body, usize::MAX).await.unwrap();
    let body = serde_json::from_slice(&bytes).unwrap_or_else(|err| {
        panic!(
            "expected JSON upstream response, got {err}: {}",
            String::from_utf8_lossy(&bytes)
        )
    });
    (status, body)
}

fn assert_openai_error(
    status: StatusCode,
    body: &Value,
    expected_status: StatusCode,
    expected_type: &str,
    expected_code: &str,
    expected_message: &str,
) {
    assert_eq!(status, expected_status, "{body}");
    assert_eq!(body["error"]["type"], expected_type, "{body}");
    assert_eq!(body["error"]["code"], expected_code, "{body}");
    assert!(body["error"]["param"].is_null(), "{body}");
    assert_eq!(body["error"]["message"], expected_message, "{body}");
}

#[tokio::test]
async fn integration_realtime_transcription_session_success_and_refresh_retry() {
    let codex_home = tempfile::tempdir().unwrap();
    let auth_home = tempfile::tempdir().unwrap();
    write_codex_auth(&codex_home, "codex-access", Some("codex-refresh"));
    let state = test_state(&codex_home, &auth_home);
    let upstream = MockServer::start().await;
    let request_body =
        Bytes::from_static(br#"{"model":"gpt-realtime-2","input_audio_format":"pcm16"}"#);
    let response_body = serde_json::json!({
        "id": "sess_local_contract",
        "object": "realtime.transcription_session"
    });

    Mock::given(method("POST"))
        .and(path("/v1/realtime/transcription_sessions"))
        .and(wiremock_header("authorization", "Bearer codex-access"))
        .and(wiremock_header("x-request-id", "req-realtime"))
        .and(body_bytes(request_body.clone()))
        .respond_with(ResponseTemplate::new(200).set_body_json(&response_body))
        .expect(1)
        .mount(&upstream)
        .await;

    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/json"),
    );
    headers.insert("x-request-id", HeaderValue::from_static("req-realtime"));
    headers.insert(
        header::AUTHORIZATION,
        HeaderValue::from_static("Bearer caller"),
    );

    let response = openai_public::send_public_openai_http_with_refresh(
        &state,
        Method::POST,
        format!("{}/v1/realtime/transcription_sessions", upstream.uri()),
        headers,
        request_body,
    )
    .await
    .unwrap();
    let (status, body) = upstream_json(response).await;

    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body, response_body);
    assert!(openai_public::is_refresh_eligible_401(
        StatusCode::UNAUTHORIZED,
        br#"{"error":{"message":"expired bearer token"}}"#,
        false,
    ));
    assert_eq!(
        openai_realtime::realtime_transcription_sessions_url("https://api.openai.com/"),
        "https://api.openai.com/v1/realtime/transcription_sessions"
    );
}

#[tokio::test]
async fn integration_realtime_missing_scope_does_not_refresh() {
    let codex_home = tempfile::tempdir().unwrap();
    let auth_home = tempfile::tempdir().unwrap();
    write_codex_auth(&codex_home, "codex-access", Some("codex-refresh"));
    let state = test_state(&codex_home, &auth_home);
    let upstream = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/realtime/transcription_sessions"))
        .and(wiremock_header("authorization", "Bearer codex-access"))
        .respond_with(ResponseTemplate::new(403).set_body_json(serde_json::json!({
            "error": {
                "type": "permission_error",
                "code": "missing_scope",
                "message": "missing scope for realtime transcription"
            }
        })))
        .expect(1)
        .mount(&upstream)
        .await;

    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/json"),
    );
    let response = openai_public::send_public_openai_http_with_refresh(
        &state,
        Method::POST,
        format!("{}/v1/realtime/transcription_sessions", upstream.uri()),
        headers,
        Bytes::from_static(br#"{"model":"gpt-realtime-2"}"#),
    )
    .await
    .unwrap();
    let (status, body) = upstream_json(response).await;

    assert_openai_error(
        status,
        &body,
        StatusCode::FORBIDDEN,
        "permission_error",
        "missing_scope",
        openai_public::MISSING_SCOPE_MESSAGE,
    );
}

#[tokio::test]
async fn integration_audio_transcriptions_reject_non_multipart_missing_boundary_and_body_over_cap()
{
    for (content_type, expected_status, expected_code) in [
        (
            "application/json",
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            "invalid_content_type",
        ),
        (
            "multipart/form-data",
            StatusCode::BAD_REQUEST,
            "missing_multipart_boundary",
        ),
    ] {
        let codex_home = tempfile::tempdir().unwrap();
        let auth_home = tempfile::tempdir().unwrap();
        write_codex_auth(&codex_home, "codex-access", None);
        let (status, body) = request_bytes_with_headers(
            test_state(&codex_home, &auth_home),
            "POST",
            "/v1/audio/transcriptions",
            &[("content-type", content_type)],
            b"not forwarded".to_vec(),
        )
        .await;
        assert_eq!(status, expected_status, "{content_type}: {body}");
        assert_eq!(body["error"]["type"], "invalid_request_error", "{body}");
        assert_eq!(body["error"]["code"], expected_code, "{body}");
    }

    let codex_home = tempfile::tempdir().unwrap();
    let auth_home = tempfile::tempdir().unwrap();
    write_codex_auth(&codex_home, "codex-access", None);
    let over_cap = vec![b'a'; 25 * 1024 * 1024 + 1];
    let (status, body) = request_bytes_with_headers(
        test_state(&codex_home, &auth_home),
        "POST",
        "/api/provider/openai/v1/audio/transcriptions",
        &[("content-type", "multipart/form-data; boundary=audio")],
        over_cap,
    )
    .await;
    assert_eq!(status, StatusCode::PAYLOAD_TOO_LARGE, "{body}");
    assert_eq!(body["error"]["type"], "invalid_request_error", "{body}");
    assert_eq!(body["error"]["code"], "audio_body_too_large", "{body}");
}

#[tokio::test]
async fn integration_audio_transcriptions_preserve_multipart_bytes_metadata_and_header_policy() {
    let codex_home = tempfile::tempdir().unwrap();
    let auth_home = tempfile::tempdir().unwrap();
    write_codex_auth(&codex_home, "codex-access", None);
    let state = test_state(&codex_home, &auth_home);
    let upstream = MockServer::start().await;
    let boundary = "audio-contract";
    let multipart = Bytes::from(format!(
        "--{boundary}\r\n\
         Content-Disposition: form-data; name=\"model\"\r\n\r\n\
         gpt-4o-transcribe\r\n\
         --{boundary}\r\n\
         Content-Disposition: form-data; name=\"file\"; filename=\"clip.wav\"\r\n\
         Content-Type: audio/wav\r\n\r\n\
         \u{0000}\u{0001}raw-audio-bytes\r\n\
         --{boundary}--\r\n"
    ));

    Mock::given(method("POST"))
        .and(path("/v1/audio/transcriptions"))
        .and(wiremock_header("authorization", "Bearer codex-access"))
        .and(wiremock_header(
            "content-type",
            format!("multipart/form-data; boundary={boundary}"),
        ))
        .and(wiremock_header("x-request-id", "req-audio"))
        .and(body_bytes(multipart.clone()))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "text": "local transcript"
        })))
        .expect(1)
        .mount(&upstream)
        .await;

    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_str(&format!("multipart/form-data; boundary={boundary}")).unwrap(),
    );
    headers.insert("x-request-id", HeaderValue::from_static("req-audio"));
    headers.insert(
        header::AUTHORIZATION,
        HeaderValue::from_static("Bearer caller"),
    );
    headers.insert(header::COOKIE, HeaderValue::from_static("secret=true"));
    headers.insert("OpenAI-Project", HeaderValue::from_static("proj-secret"));

    let request = openai_audio::build_audio_transcriptions_request(&state, &headers).unwrap();
    assert_eq!(
        request
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok()),
        Some("Bearer codex-access")
    );
    let expected_content_type = format!("multipart/form-data; boundary={boundary}");
    assert_eq!(
        request
            .headers
            .get(header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok()),
        Some(expected_content_type.as_str())
    );
    assert_eq!(
        request
            .headers
            .get("x-request-id")
            .and_then(|value| value.to_str().ok()),
        Some("req-audio")
    );
    assert!(!request.headers.contains_key(header::COOKIE));
    assert!(!request.headers.contains_key("OpenAI-Project"));

    let response = openai_public::send_public_openai_http_with_refresh(
        &state,
        Method::POST,
        format!("{}/v1/audio/transcriptions", upstream.uri()),
        headers,
        multipart,
    )
    .await
    .unwrap();
    let (status, body) = upstream_json(response).await;

    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["text"], "local transcript");
}

#[tokio::test]
async fn integration_openai_realtime_audio_route_matrix_unsupported_no_outbound() {
    let known_feature_routes = [
        ("POST", "/v1/realtime/client_secrets"),
        ("POST", "/api/provider/openai/v1/realtime/client_secrets"),
        ("POST", "/v1/realtime/calls"),
        ("POST", "/api/provider/openai/v1/realtime/calls"),
        ("POST", "/v1/audio/speech"),
        ("POST", "/api/provider/openai/v1/audio/speech"),
        ("POST", "/v1/audio/translations"),
        ("POST", "/api/provider/openai/v1/audio/translations"),
        ("POST", "/transcribe"),
    ];

    for (method, path) in known_feature_routes {
        let (status, body) = request_json(
            method,
            path,
            Some(serde_json::json!({ "model": "gpt-realtime-2" })),
        )
        .await;
        assert_openai_error(
            status,
            &body,
            StatusCode::NOT_IMPLEMENTED,
            "invalid_request_error",
            "unsupported_feature",
            UNSUPPORTED_FEATURE_MESSAGE,
        );
    }

    let unknown_routes = [
        ("POST", "/v1/realtime/calls/call_123/accept"),
        (
            "POST",
            "/api/provider/openai/v1/realtime/calls/call_123/accept",
        ),
        ("POST", "/v1/realtime/calls/call_123/reject"),
        (
            "POST",
            "/api/provider/openai/v1/realtime/calls/call_123/reject",
        ),
        ("POST", "/v1/realtime/calls/call_123/hangup"),
        (
            "POST",
            "/api/provider/openai/v1/realtime/calls/call_123/hangup",
        ),
        ("POST", "/v1/realtime/calls/call_123/refer"),
        (
            "POST",
            "/api/provider/openai/v1/realtime/calls/call_123/refer",
        ),
        ("POST", "/v1/realtime/calls/call_123/escalate"),
        (
            "POST",
            "/api/provider/openai/v1/realtime/calls/call_123/escalate",
        ),
        ("POST", "/v1/realtime/unknown_child"),
        ("POST", "/api/provider/openai/v1/realtime/unknown_child"),
        ("POST", "/v1/audio/unknown_child"),
        ("POST", "/api/provider/openai/v1/audio/unknown_child"),
    ];

    for (method, path) in unknown_routes {
        let (status, body) = request_json(method, path, Some(serde_json::json!({}))).await;
        assert_openai_error(
            status,
            &body,
            StatusCode::NOT_IMPLEMENTED,
            "invalid_request_error",
            "unsupported_route",
            UNSUPPORTED_ROUTE_MESSAGE,
        );
    }
}

#[tokio::test]
async fn integration_public_realtime_transcription_sessions_missing_auth() {
    let (status, body) = request_json(
        "POST",
        "/v1/realtime/transcription_sessions",
        Some(serde_json::json!({ "model": "gpt-realtime-2" })),
    )
    .await;
    assert_openai_error(
        status,
        &body,
        StatusCode::UNAUTHORIZED,
        "authentication_error",
        "invalid_api_key",
        MISSING_CODEX_AUTH_MESSAGE,
    );
}

#[tokio::test]
async fn integration_provider_realtime_transcription_sessions_missing_auth() {
    let (status, body) = request_json(
        "POST",
        "/api/provider/openai/v1/realtime/transcription_sessions",
        Some(serde_json::json!({ "model": "gpt-realtime-2" })),
    )
    .await;
    assert_openai_error(
        status,
        &body,
        StatusCode::UNAUTHORIZED,
        "authentication_error",
        "invalid_api_key",
        MISSING_CODEX_AUTH_MESSAGE,
    );
}

#[tokio::test]
async fn integration_public_audio_transcriptions_missing_auth() {
    let codex_home = tempfile::tempdir().unwrap();
    let auth_home = tempfile::tempdir().unwrap();
    let (status, body) = request_bytes_with_headers(
        test_state(&codex_home, &auth_home),
        "POST",
        "/v1/audio/transcriptions",
        &[("content-type", "multipart/form-data; boundary=audio")],
        b"--audio\r\n--audio--\r\n".to_vec(),
    )
    .await;
    assert_openai_error(
        status,
        &body,
        StatusCode::UNAUTHORIZED,
        "authentication_error",
        "invalid_api_key",
        MISSING_CODEX_AUTH_MESSAGE,
    );
}

#[tokio::test]
async fn integration_provider_audio_transcriptions_missing_auth() {
    let codex_home = tempfile::tempdir().unwrap();
    let auth_home = tempfile::tempdir().unwrap();
    let (status, body) = request_bytes_with_headers(
        test_state(&codex_home, &auth_home),
        "POST",
        "/api/provider/openai/v1/audio/transcriptions",
        &[("content-type", "multipart/form-data; boundary=audio")],
        b"--audio\r\n--audio--\r\n".to_vec(),
    )
    .await;
    assert_openai_error(
        status,
        &body,
        StatusCode::UNAUTHORIZED,
        "authentication_error",
        "invalid_api_key",
        MISSING_CODEX_AUTH_MESSAGE,
    );
}

#[tokio::test]
async fn integration_realtime_and_audio_missing_codex_auth_openai_401_no_outbound() {
    for path in [
        "/v1/realtime/transcription_sessions",
        "/api/provider/openai/v1/realtime/transcription_sessions",
    ] {
        let (status, body) = request_json(
            "POST",
            path,
            Some(serde_json::json!({ "model": "gpt-realtime-2" })),
        )
        .await;
        assert_openai_error(
            status,
            &body,
            StatusCode::UNAUTHORIZED,
            "authentication_error",
            "invalid_api_key",
            MISSING_CODEX_AUTH_MESSAGE,
        );
    }

    for path in [
        "/v1/audio/transcriptions",
        "/api/provider/openai/v1/audio/transcriptions",
    ] {
        let codex_home = tempfile::tempdir().unwrap();
        let auth_home = tempfile::tempdir().unwrap();
        let (status, body) = request_bytes_with_headers(
            test_state(&codex_home, &auth_home),
            "POST",
            path,
            &[("content-type", "multipart/form-data; boundary=audio")],
            b"--audio\r\n--audio--\r\n".to_vec(),
        )
        .await;
        assert_openai_error(
            status,
            &body,
            StatusCode::UNAUTHORIZED,
            "authentication_error",
            "invalid_api_key",
            MISSING_CODEX_AUTH_MESSAGE,
        );
    }
}

#[tokio::test]
async fn integration_responses_rejects_audio_file_inputs_without_realtime_audio_fallback() {
    for (path, input_type) in [
        ("/v1/responses", "input_audio"),
        ("/v1/responses", "input_file"),
        ("/api/provider/openai/v1/responses", "input_audio"),
        ("/api/provider/openai/v1/responses", "input_file"),
    ] {
        let body = serde_json::json!({
            "model": "openai:gpt-5.5",
            "input": [{
                "role": "user",
                "content": [{ "type": input_type }]
            }]
        });
        let (status, response) = request_json("POST", path, Some(body)).await;

        assert_eq!(
            status,
            StatusCode::BAD_REQUEST,
            "{path} {input_type}: {response}"
        );
        assert_eq!(
            response["error"]["type"], "invalid_request_error",
            "{response}"
        );
        assert_eq!(
            response["error"]["code"], "unsupported_feature",
            "{response}"
        );
        assert!(response["error"]["param"].is_null(), "{response}");
        assert!(
            response["error"]["message"]
                .as_str()
                .unwrap()
                .contains(input_type),
            "{response}"
        );
        assert!(
            !response["error"]["message"]
                .as_str()
                .unwrap()
                .contains("~/.codex/auth.json"),
            "{response}"
        );
    }
}

#[tokio::test]
async fn integration_responses_rejects_audio_transcription_models_without_audio_route_fallback() {
    for path in ["/v1/responses", "/api/provider/openai/v1/responses"] {
        let (status, response) = request_json(
            "POST",
            path,
            Some(serde_json::json!({
                "model": "gpt-4o-transcribe",
                "input": "transcribe this"
            })),
        )
        .await;

        assert_eq!(status, StatusCode::BAD_REQUEST, "{path}: {response}");
        assert_eq!(
            response["error"]["type"], "invalid_request_error",
            "{response}"
        );
        assert_eq!(
            response["error"]["code"], "model_not_supported",
            "{response}"
        );
        assert!(!response["error"]["message"]
            .as_str()
            .unwrap()
            .contains("~/.codex/auth.json"));
    }
}

#[tokio::test]
async fn integration_provider_openai_models_preserves_existing_boundary_without_public_audio_proxy()
{
    let (status, body) = request_json(
        "GET",
        "/api/provider/openai/v1/models?client_version=26.506.31421",
        None,
    )
    .await;

    assert_eq!(status, StatusCode::UNAUTHORIZED, "{body}");
    assert_eq!(body["error"]["type"], "authentication_error", "{body}");
    assert_eq!(body["error"]["code"], "invalid_api_key", "{body}");
    assert!(body["error"]["param"].is_null(), "{body}");
    assert_ne!(body["error"]["code"], "unsupported_feature");
    assert_ne!(body["error"]["code"], "unsupported_route");
}

#[tokio::test]
async fn integration_provider_prefixed_realtime_models_do_not_cross_responses_boundary() {
    for path in [
        "/api/provider/openai/v1/responses",
        "/v1/responses",
        "/api/provider/openai/v1/chat/completions",
        "/v1/chat/completions",
    ] {
        let body = if path.ends_with("chat/completions") {
            serde_json::json!({
                "model": "gpt-realtime-2",
                "messages": [{ "role": "user", "content": "hello" }]
            })
        } else {
            serde_json::json!({
                "model": "gpt-realtime-2",
                "input": "hello"
            })
        };
        let (status, response) = request_json("POST", path, Some(body)).await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{path}: {response}");
        assert_eq!(
            response["error"]["type"], "invalid_request_error",
            "{response}"
        );
        assert_eq!(
            response["error"]["code"], "model_not_supported",
            "{response}"
        );
        assert!(response["error"]["param"].is_null(), "{response}");
        assert!(
            !response["error"]["message"]
                .as_str()
                .unwrap()
                .contains("~/.codex/auth.json"),
            "{response}"
        );
    }
}
