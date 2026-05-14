use std::{env, ffi::OsString, fs, sync::Mutex};

use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
};
use serde_json::Value;
use tempfile::TempDir;
use tower::ServiceExt;
use unified_model_proxy_v2::{build_router, state::NewResponseStateRecord, AppState};

static UPSTREAM_ENV_LOCK: Mutex<()> = Mutex::new(());

struct EnvRestore {
    key: &'static str,
    previous: Option<OsString>,
}

impl EnvRestore {
    fn clear(key: &'static str) -> Self {
        let previous = env::var_os(key);
        env::remove_var(key);
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

fn test_state(codex_home: &TempDir, auth_home: &TempDir) -> AppState {
    AppState::for_tests(
        codex_home.path().to_path_buf(),
        auth_home.path().to_path_buf(),
    )
}

async fn request_json(method: &str, path: &str, body: Option<Value>) -> (StatusCode, Value) {
    let codex_home = tempfile::tempdir().unwrap();
    let auth_home = tempfile::tempdir().unwrap();
    request_json_with_state(test_state(&codex_home, &auth_home), method, path, body).await
}

fn request_json_without_bedrock_env(
    method: &str,
    path: &str,
    body: Option<Value>,
) -> (StatusCode, Value) {
    request_json_without_env_vars(method, path, body, &["AWS_BEARER_TOKEN_BEDROCK"])
}

fn request_json_without_google_env(
    method: &str,
    path: &str,
    body: Option<Value>,
) -> (StatusCode, Value) {
    request_json_without_env_vars(method, path, body, &["GOOGLE_API_KEY"])
}

fn request_json_without_env_vars(
    method: &str,
    path: &str,
    body: Option<Value>,
    keys: &[&'static str],
) -> (StatusCode, Value) {
    let _guard = UPSTREAM_ENV_LOCK.lock().unwrap();
    let _restores = keys
        .iter()
        .map(|key| EnvRestore::clear(key))
        .collect::<Vec<_>>();
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(request_json(method, path, body))
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
    let json = serde_json::from_slice(&bytes).unwrap();
    (status, json)
}

async fn request_text_with_state(
    state: AppState,
    method: &str,
    path: &str,
    body: Option<&str>,
) -> (StatusCode, String) {
    let app = build_router(state);
    let request = Request::builder()
        .method(method)
        .uri(path)
        .body(match body {
            Some(value) => Body::from(value.to_string()),
            None => Body::empty(),
        })
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    let status = response.status();
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let text = String::from_utf8(bytes.to_vec()).unwrap();
    (status, text)
}

#[tokio::test]
async fn integration_routes_health_route_reports_ok() {
    let (status, body) = request_json("GET", "/health", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"], "ok");
}

#[tokio::test]
async fn integration_routes_models_route_returns_stable_known_models() {
    let (status, body) = request_json("GET", "/v1/models", None).await;
    assert_eq!(status, StatusCode::OK);
    let ids: Vec<&str> = body["data"]
        .as_array()
        .unwrap()
        .iter()
        .map(|model| model["id"].as_str().unwrap())
        .collect();
    assert!(ids.contains(&"anthropic/claude-sonnet-4-6"));
    assert!(ids.contains(&"claude-opus-4-7"));
    assert!(ids.contains(&"claude-sonnet-4-6"));
    assert!(ids.contains(&"claude-sonnet-4-6-max"));
    assert!(ids.contains(&"gemini-3.1-pro-preview"));
    assert!(ids.contains(&"openai:gpt-5.5"));
    assert!(ids.contains(&"gpt-image-2"));
}

#[tokio::test]
async fn integration_routes_openai_provider_models_uses_codex_auth_gate() {
    let (status, body) = request_json(
        "GET",
        "/api/provider/openai/v1/models?client_version=26.506.31421",
        None,
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["error"]["type"], "missing_credential");
    assert!(body["error"]["message"]
        .as_str()
        .unwrap()
        .contains("~/.codex/auth.json"));
}

#[tokio::test]
async fn integration_routes_response_retrieve_returns_404_for_store_false_continuation_state() {
    let codex_home = tempfile::tempdir().unwrap();
    let auth_home = tempfile::tempdir().unwrap();
    let state = test_state(&codex_home, &auth_home);
    state.remember_response_for_continuation(response_record(
        "resp_ump_store_false",
        "resp_upstream_store_false",
        serde_json::json!({
            "id": "resp_ump_store_false",
            "store": false,
            "output": []
        }),
    ));

    let (status, body) = request_json_with_state(
        state.clone(),
        "GET",
        "/v1/responses/resp_ump_store_false",
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["error"]["type"], "not_found");

    let (status, body) = request_json_with_state(
        state,
        "GET",
        "/v1/responses/resp_ump_store_false/input_items",
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["error"]["type"], "not_found");
}

#[tokio::test]
async fn integration_routes_response_retrieve_uses_adapter_public_storage_for_store_true() {
    let codex_home = tempfile::tempdir().unwrap();
    let auth_home = tempfile::tempdir().unwrap();
    let state = test_state(&codex_home, &auth_home);
    state.store_public_response(response_record(
        "resp_ump_store_true",
        "resp_upstream_store_true",
        serde_json::json!({
            "id": "resp_ump_store_true",
            "store": true,
            "output": [{ "id": "item_public", "type": "message" }]
        }),
    ));

    let (status, body) = request_json_with_state(
        state.clone(),
        "GET",
        "/v1/responses/resp_ump_store_true",
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["id"], "resp_ump_store_true");
    assert_eq!(body["store"], true);

    let (status, body) = request_json_with_state(
        state,
        "GET",
        "/v1/responses/resp_ump_store_true/input_items",
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"][0]["id"], "item_public");
}

#[tokio::test]
async fn integration_routes_amp_startup_internal_routes_return_local_stubs() {
    let (status, body) = request_json(
        "POST",
        "/api/internal?getUserInfo",
        Some(serde_json::json!({ "method": "getUserInfo", "params": {} })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["ok"], true);
    assert_eq!(body["result"]["email"], "jared@ampcode.com");

    let (status, body) = request_json(
        "POST",
        "/api/internal?loadPlugins",
        Some(serde_json::json!({ "method": "loadPlugins", "params": {} })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, serde_json::json!({ "ok": true, "result": [] }));

    let (status, body) = request_json(
        "POST",
        "/api/internal?getUserFreeTierStatus",
        Some(serde_json::json!({
            "method": "getUserFreeTierStatus",
            "params": {}
        })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["result"]["canUseAmpFree"], false);
}

#[tokio::test]
async fn integration_routes_telemetry_route_is_accepted_without_body() {
    let codex_home = tempfile::tempdir().unwrap();
    let auth_home = tempfile::tempdir().unwrap();
    let app = build_router(test_state(&codex_home, &auth_home));
    let request = Request::builder()
        .method("POST")
        .uri("/api/telemetry")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn integration_routes_count_tokens_is_local_stub_and_rejects_unknown_models() {
    let known = serde_json::json!({
        "model": "anthropic/claude-sonnet-4-6",
        "messages": [{ "role": "user", "content": "hello world" }]
    });
    let (status, body) = request_json(
        "POST",
        "/api/provider/anthropic/v1/messages/count_tokens",
        Some(known),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(body["input_tokens"].as_u64().unwrap() > 0);

    let codex = serde_json::json!({
        "model": "openai:gpt-5.5",
        "messages": [{ "role": "user", "content": "hello world" }]
    });
    let (status, body) = request_json(
        "POST",
        "/api/provider/anthropic/v1/messages/count_tokens",
        Some(codex),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(body["input_tokens"].as_u64().unwrap() > 0);

    let unknown = serde_json::json!({ "model": "nope/nope", "messages": [] });
    let (status, body) = request_json(
        "POST",
        "/api/provider/anthropic/v1/messages/count_tokens",
        Some(unknown),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"]["type"], "model_not_supported");
}

#[tokio::test]
async fn integration_routes_known_gpt_messages_route_rejects_lossy_token_cap_before_auth() {
    let gpt_messages = serde_json::json!({
        "model": "openai:gpt-5.5",
        "max_tokens": 64,
        "messages": [{ "role": "user", "content": "hello" }]
    });
    let (status, body) = request_json("POST", "/v1/messages", Some(gpt_messages)).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"]["type"], "invalid_request");
    let message = body["error"]["message"].as_str().unwrap();
    assert!(message.contains("max_tokens") || message.contains("max_output_tokens"));
}

#[test]
fn integration_routes_anthropic_messages_path_reaches_bedrock_credential_gate() {
    let body = serde_json::json!({
        "model": "anthropic/claude-sonnet-4-6",
        "max_tokens": 64,
        "messages": [{ "role": "user", "content": "hello" }]
    });
    let (status, body) =
        request_json_without_bedrock_env("POST", "/api/provider/anthropic/v1/messages", Some(body));
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["error"]["type"], "missing_credential");
    assert!(body["error"]["message"]
        .as_str()
        .unwrap()
        .contains("Bedrock bearer/profile"));
}

#[test]
fn integration_routes_anthropic_responses_routes_reach_bedrock_credential_gate() {
    let body = serde_json::json!({
        "model": "claude-opus-4-7",
        "input": "hello"
    });
    let (status, body) = request_json_without_bedrock_env("POST", "/v1/responses", Some(body));
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["error"]["type"], "missing_credential");
}

#[test]
fn integration_routes_sonnet_responses_routes_reach_bedrock_credential_gate() {
    let body = serde_json::json!({
        "model": "claude-sonnet-4-6",
        "input": "hello"
    });
    let (status, body) = request_json_without_bedrock_env("POST", "/v1/responses", Some(body));
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["error"]["type"], "missing_credential");
}

#[test]
fn integration_routes_provider_openai_anthropic_responses_reach_bedrock_credential_gate() {
    let body = serde_json::json!({
        "model": "anthropic/claude-opus-4-7",
        "input": "hello"
    });
    let (status, body) =
        request_json_without_bedrock_env("POST", "/api/provider/openai/v1/responses", Some(body));
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["error"]["type"], "missing_credential");
}

#[test]
fn integration_routes_google_responses_routes_reach_google_credential_gate() {
    let body = serde_json::json!({
        "model": "gemini-3.1-flash-lite",
        "input": "hello"
    });
    let (status, body) = request_json_without_google_env("POST", "/v1/responses", Some(body));
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["error"]["type"], "missing_credential");
    assert!(body["error"]["message"]
        .as_str()
        .unwrap()
        .contains("GOOGLE_API_KEY"));
}

#[test]
fn integration_routes_google_responses_stream_reaches_google_credential_gate() {
    let body = serde_json::json!({
        "model": "gemini-3.1-flash-lite",
        "input": "hello",
        "stream": true
    });
    let (status, body) = request_json_without_google_env("POST", "/v1/responses", Some(body));
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["error"]["type"], "missing_credential");
    assert!(body["error"]["message"]
        .as_str()
        .unwrap()
        .contains("GOOGLE_API_KEY"));
}

#[test]
fn integration_routes_direct_google_generate_content_reaches_google_credential_gate() {
    let body = serde_json::json!({
        "contents": [{ "role": "user", "parts": [{ "text": "hello" }] }]
    });
    let (status, body) = request_json_without_google_env(
        "POST",
        "/v1beta/models/gemini-3.1-flash-lite:generateContent",
        Some(body),
    );
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["error"]["type"], "missing_credential");
}

#[test]
fn integration_routes_direct_google_stream_reaches_google_credential_gate() {
    let body = serde_json::json!({
        "contents": [{ "role": "user", "parts": [{ "text": "hello" }] }]
    });
    let (status, body) = request_json_without_google_env(
        "POST",
        "/v1beta/models/gemini-3.1-flash-lite:streamGenerateContent?alt=sse",
        Some(body),
    );
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["error"]["type"], "missing_credential");
}

#[test]
fn integration_routes_vertex_stream_reaches_google_credential_gate() {
    let body = serde_json::json!({
        "contents": [{ "role": "user", "parts": [{ "text": "hello" }] }]
    });
    let (status, body) = request_json_without_google_env(
        "POST",
        "/v1/projects/proj/locations/us-central1/publishers/google/models/gemini-3.1-flash-lite:streamGenerateContent",
        Some(body),
    );
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["error"]["type"], "missing_credential");
}

#[test]
fn integration_routes_provider_openai_google_responses_reach_google_credential_gate() {
    let body = serde_json::json!({
        "model": "vertexai/gemini-3.1-flash-lite",
        "input": "hello"
    });
    let (status, body) =
        request_json_without_google_env("POST", "/api/provider/openai/v1/responses", Some(body));
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["error"]["type"], "missing_credential");
    assert!(body["error"]["message"]
        .as_str()
        .unwrap()
        .contains("GOOGLE_API_KEY"));
}

#[tokio::test]
async fn integration_routes_hot_config_reloads_model_routes_between_requests() {
    let codex_home = tempfile::tempdir().unwrap();
    let auth_home = tempfile::tempdir().unwrap();
    let config_path = auth_home.path().join("config.json");
    let state = test_state(&codex_home, &auth_home);
    let body = serde_json::json!({
        "model": "gemini-3.1-flash-lite",
        "input": "hello"
    });

    fs::write(
        &config_path,
        serde_json::json!({
            "routes": [{
                "source": { "model": "gemini-3.1-flash-lite", "format": "responses" },
                "target": { "provider": "codex", "model": "gpt-5.5", "format": "responses" }
            }]
        })
        .to_string(),
    )
    .unwrap();

    let (status, response) =
        request_json_with_state(state.clone(), "POST", "/v1/responses", Some(body.clone())).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert!(response["error"]["message"]
        .as_str()
        .unwrap()
        .contains("~/.codex/auth.json"));

    fs::write(
        &config_path,
        serde_json::json!({
            "routes": [{
                "source": { "model": "gemini-3.1-flash-lite", "format": "responses" },
                "target": { "provider": "google", "model": "gemini-3.1-flash-lite", "format": "google_generate_content" }
            }]
        })
        .to_string(),
    )
    .unwrap();

    let (status, response) =
        request_json_with_state(state, "POST", "/v1/responses", Some(body)).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert!(response["error"]["message"]
        .as_str()
        .unwrap()
        .contains("GOOGLE_API_KEY"));
}

#[tokio::test]
async fn integration_routes_hot_config_uses_source_format_for_same_model() {
    let codex_home = tempfile::tempdir().unwrap();
    let auth_home = tempfile::tempdir().unwrap();
    fs::write(
        auth_home.path().join("config.json"),
        serde_json::json!({
            "routes": [
                {
                    "source": { "model": "same-model", "format": "responses" },
                    "target": { "provider": "google", "model": "gemini-3.1-flash-lite", "format": "google_generate_content" }
                },
                {
                    "source": { "model": "same-model", "format": "chat_completions" },
                    "target": { "provider": "codex", "model": "gpt-5.5", "format": "responses" }
                }
            ]
        })
        .to_string(),
    )
    .unwrap();
    let state = test_state(&codex_home, &auth_home);

    let (status, body) = request_json_with_state(
        state.clone(),
        "POST",
        "/v1/responses",
        Some(serde_json::json!({ "model": "same-model", "input": "hello" })),
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert!(body["error"]["message"]
        .as_str()
        .unwrap()
        .contains("GOOGLE_API_KEY"));

    let (status, body) = request_json_with_state(
        state,
        "POST",
        "/v1/chat/completions",
        Some(serde_json::json!({
            "model": "same-model",
            "messages": [{ "role": "user", "content": "hello" }]
        })),
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert!(body["error"]["message"]
        .as_str()
        .unwrap()
        .contains("~/.codex/auth.json"));
}

#[tokio::test]
async fn integration_routes_config_ui_reads_and_writes_hot_config() {
    let codex_home = tempfile::tempdir().unwrap();
    let auth_home = tempfile::tempdir().unwrap();
    let config_path = auth_home.path().join("config.json");
    let state = test_state(&codex_home, &auth_home);

    let (status, html) = request_text_with_state(state.clone(), "GET", "/config", None).await;
    assert_eq!(status, StatusCode::OK);
    assert!(html.contains("Unified Model Proxy v2"));
    assert!(html.contains("/api/config"));

    let (status, body) = request_json_with_state(state.clone(), "GET", "/api/config", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, serde_json::json!({ "routes": [] }));

    let config = serde_json::json!({
        "routes": [{
            "source": { "model": "gemini-3.1-flash-lite", "format": "responses" },
            "target": { "provider": "codex", "model": "gpt-5.5", "format": "responses" }
        }]
    });
    let (status, body) =
        request_json_with_state(state.clone(), "PUT", "/api/config", Some(config.clone())).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["ok"], true);

    let written: Value = serde_json::from_str(&fs::read_to_string(&config_path).unwrap()).unwrap();
    assert_eq!(written, config);

    let (status, body) = request_json_with_state(state, "GET", "/api/config", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, config);
}

#[tokio::test]
async fn integration_routes_route_model_resolvers_reject_unknown_models_before_credentials() {
    let unknown_chat = serde_json::json!({ "model": "nope/nope", "messages": [] });
    let (status, body) = request_json("POST", "/v1/chat/completions", Some(unknown_chat)).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"]["type"], "model_not_supported");
}

#[tokio::test]
async fn integration_routes_known_gpt_chat_routes_to_codex_without_real_upstream() {
    let gpt_chat = serde_json::json!({
        "model": "openai:gpt-5.5",
        "messages": [{ "role": "user", "content": "hello" }]
    });
    let (status, body) = request_json("POST", "/v1/chat/completions", Some(gpt_chat)).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["error"]["type"], "missing_credential");
}

#[tokio::test]
async fn integration_routes_image_routes_return_explicit_unsupported_error() {
    let body = serde_json::json!({ "model": "gpt-image-2", "prompt": "paint" });
    let (status, body) = request_json(
        "POST",
        "/api/provider/openai/v1/images/generations",
        Some(body),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"]["type"], "model_not_supported");
    assert!(body["error"]["message"]
        .as_str()
        .unwrap()
        .contains("gpt-image-2"));
}

#[tokio::test]
async fn integration_routes_audio_and_realtime_are_explicit_feature_gates() {
    for (path, marker) in [
        (
            "/v1/realtime/transcription_sessions",
            "realtime transcription sessions",
        ),
        (
            "/api/provider/openai/v1/realtime/transcription_sessions",
            "realtime transcription sessions",
        ),
        ("/v1/audio/speech", "audio speech"),
        ("/api/provider/openai/v1/audio/speech", "audio speech"),
        ("/transcribe", "dictation transcription"),
    ] {
        let (status, body) = request_json(
            "POST",
            path,
            Some(serde_json::json!({ "model": "gpt-realtime-2" })),
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{path}");
        assert_eq!(body["error"]["type"], "unsupported_route", "{path}");
        assert!(
            body["error"]["message"].as_str().unwrap().contains(marker),
            "{path}: {body}"
        );
    }
}

fn response_record(
    adapter_response_id: &str,
    upstream_response_id: &str,
    raw_response: Value,
) -> NewResponseStateRecord {
    NewResponseStateRecord {
        route: "responses".to_string(),
        provider: "codex".to_string(),
        upstream_model: "gpt-5.5".to_string(),
        upstream_response_id: upstream_response_id.to_string(),
        adapter_response_id: adapter_response_id.to_string(),
        conversation_id: None,
        raw_response,
        raw_input_items: serde_json::json!({
            "object": "list",
            "data": [{ "id": "item_public", "type": "message" }]
        }),
        upstream_codex_minted: true,
    }
}
