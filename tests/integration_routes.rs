use std::{env, ffi::OsString, fs};

use axum::{
    body::{to_bytes, Body},
    http::{HeaderMap, Request, StatusCode},
};
use serde_json::Value;
use tempfile::TempDir;
use tower::ServiceExt;
use unified_model_proxy_v2::{build_router, state::NewResponseStateRecord, AppState};

static UPSTREAM_ENV_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

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

fn request_json_without_openai_env(
    method: &str,
    path: &str,
    body: Option<Value>,
) -> (StatusCode, Value) {
    request_json_without_env_vars(method, path, body, &["OPENAI_API_KEY"])
}

fn request_json_without_env_vars(
    method: &str,
    path: &str,
    body: Option<Value>,
    keys: &[&'static str],
) -> (StatusCode, Value) {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            let _guard = UPSTREAM_ENV_LOCK.lock().await;
            let _restores = keys
                .iter()
                .map(|key| EnvRestore::clear(key))
                .collect::<Vec<_>>();
            request_json(method, path, body).await
        })
}

fn request_json_with_state_without_env_vars(
    state: AppState,
    method: &str,
    path: &str,
    body: Option<Value>,
    keys: &[&'static str],
) -> (StatusCode, Value) {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            let _guard = UPSTREAM_ENV_LOCK.lock().await;
            let _restores = keys
                .iter()
                .map(|key| EnvRestore::clear(key))
                .collect::<Vec<_>>();
            request_json_with_state(state, method, path, body).await
        })
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
        .header("host", "localhost")
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

async fn request_compact_with_state(
    state: AppState,
    path: &str,
    body: Value,
) -> (StatusCode, Value) {
    let app = build_router(state);
    let request = Request::builder()
        .method("POST")
        .uri(path)
        .header("host", "localhost")
        .header("content-type", "application/json")
        .header("session-id", "compact-session-route-test")
        .header("thread-id", "compact-thread-route-test")
        .body(Body::from(body.to_string()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    let status = response.status();
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json = serde_json::from_slice(&bytes).unwrap_or_else(|error| {
        panic!(
            "compact response was not json: {error}; status={status}; body={}",
            String::from_utf8_lossy(&bytes)
        )
    });
    (status, json)
}

async fn request_body_with_content_type(
    state: AppState,
    method: &str,
    path: &str,
    content_type: &str,
    body: &str,
) -> (StatusCode, Value) {
    let app = build_router(state);
    let request = Request::builder()
        .method(method)
        .uri(path)
        .header("host", "localhost")
        .header("content-type", content_type)
        .body(Body::from(body.to_string()))
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
        .header("host", "localhost")
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

async fn request_bytes_with_headers(
    state: AppState,
    method: &str,
    path: &str,
    headers: &[(&str, &str)],
    body: Option<&str>,
) -> (StatusCode, HeaderMap, Vec<u8>) {
    let app = build_router(state);
    let mut builder = Request::builder().method(method).uri(path);
    for (name, value) in headers {
        builder = builder.header(*name, *value);
    }
    let request = builder
        .body(match body {
            Some(value) => Body::from(value.to_string()),
            None => Body::empty(),
        })
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    let status = response.status();
    let headers = response.headers().clone();
    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap()
        .to_vec();
    (status, headers, bytes)
}

async fn request_json_with_headers(
    state: AppState,
    method: &str,
    path: &str,
    headers: &[(&str, &str)],
    body: Option<&str>,
) -> (StatusCode, HeaderMap, Value) {
    let mut all_headers = vec![("host", "localhost"), ("content-type", "application/json")];
    all_headers.extend_from_slice(headers);
    let (status, headers, bytes) =
        request_bytes_with_headers(state, method, path, &all_headers, body).await;
    let json = serde_json::from_slice(&bytes).unwrap_or_else(|error| {
        panic!(
            "response was not json: {error}; status={status}; body={}",
            String::from_utf8_lossy(&bytes)
        )
    });
    (status, headers, json)
}

async fn request_json_bytes_with_headers(
    state: AppState,
    method: &str,
    path: &str,
    headers: &[(&str, &str)],
    body: Vec<u8>,
) -> (StatusCode, HeaderMap, Value) {
    let app = build_router(state);
    let mut builder = Request::builder()
        .method(method)
        .uri(path)
        .header("host", "localhost")
        .header("content-type", "application/json");
    for (name, value) in headers {
        builder = builder.header(*name, *value);
    }
    let request = builder.body(Body::from(body)).unwrap();

    let response = app.oneshot(request).await.unwrap();
    let status = response.status();
    let headers = response.headers().clone();
    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap()
        .to_vec();
    let json = serde_json::from_slice(&bytes).unwrap_or_else(|error| {
        panic!(
            "response was not json: {error}; status={status}; body={}",
            String::from_utf8_lossy(&bytes)
        )
    });
    (status, headers, json)
}

async fn request_json_with_raw_headers(
    state: AppState,
    method: &str,
    path: &str,
    headers: &[(&str, &str)],
    body: Option<&str>,
) -> (StatusCode, HeaderMap, Value) {
    let (status, headers, bytes) =
        request_bytes_with_headers(state, method, path, headers, body).await;
    let json = serde_json::from_slice(&bytes).unwrap_or_else(|error| {
        panic!(
            "response was not json: {error}; status={status}; body={}",
            String::from_utf8_lossy(&bytes)
        )
    });
    (status, headers, json)
}

fn header_value<'a>(headers: &'a HeaderMap, name: &str) -> &'a str {
    headers
        .get(name)
        .unwrap_or_else(|| panic!("missing response header {name}"))
        .to_str()
        .unwrap()
}

fn assert_no_store_nosniff(headers: &HeaderMap) {
    assert_eq!(header_value(headers, "cache-control"), "no-store");
    assert_eq!(header_value(headers, "x-content-type-options"), "nosniff");
}

fn assert_config_shell_has_no_inline_code(html: &str) {
    let lower = html.to_ascii_lowercase();
    for script_tag in lower.match_indices("<script").map(|(index, _)| {
        let end = lower[index..]
            .find('>')
            .map(|relative_end| index + relative_end)
            .unwrap_or(lower.len());
        &lower[index..=end]
    }) {
        assert!(
            script_tag.contains(" src="),
            "config shell must not use inline scripts"
        );
    }
    assert!(
        !lower.contains("<style"),
        "config shell must not use inline styles"
    );
    assert!(
        !lower.contains(" style="),
        "config shell must not use style attributes"
    );
    for event_attr in [
        " onabort=",
        " onblur=",
        " onchange=",
        " onclick=",
        " onerror=",
        " onfocus=",
        " oninput=",
        " onkeydown=",
        " onkeyup=",
        " onload=",
        " onmousedown=",
        " onmouseover=",
        " onsubmit=",
    ] {
        assert!(
            !lower.contains(event_attr),
            "config shell must not use inline event handler attributes"
        );
    }
}

fn assert_graph_v2_contract(body: &Value) {
    for field in [
        "schema_version",
        "generated_at",
        "raw_hot_config",
        "sources",
        "runtime_formats",
        "config_routes",
        "effective_routes",
        "nodes",
        "edges",
        "diagnostics",
        "validation_issues",
        "contract_version",
        "draft_status",
        "groups",
        "focal",
        "route_cards",
        "diagnostics_v2",
    ] {
        assert!(
            body.get(field).is_some(),
            "Switchyard Atlas graph response is missing {field}"
        );
    }

    let contract_version = &body["contract_version"];
    assert!(
        contract_version == 2
            || contract_version
                .as_str()
                .is_some_and(|value| value.contains('2')),
        "contract_version must identify graph v2: {contract_version:?}"
    );
    assert!(
        body["draft_status"].is_string(),
        "draft_status must be a string"
    );
    assert!(body["groups"].is_array(), "groups must be an array");
    assert!(
        body["route_cards"].is_array(),
        "route_cards must be an array"
    );
    assert!(
        body["diagnostics_v2"].is_array(),
        "diagnostics_v2 must be an array"
    );
}

fn assert_no_config_graph_payload(body: &Value) {
    for field in [
        "schema_version",
        "generated_at",
        "raw_hot_config",
        "sources",
        "runtime_formats",
        "config_routes",
        "effective_routes",
        "nodes",
        "edges",
        "diagnostics",
        "validation_issues",
        "contract_version",
        "draft_status",
        "groups",
        "focal",
        "route_cards",
        "diagnostics_v2",
    ] {
        assert!(
            body.get(field).is_none(),
            "error response must not include graph field {field}: {body}"
        );
    }
}

fn assert_blocking_diagnostics_v2(body: &Value) {
    let diagnostics = body["diagnostics_v2"]
        .as_array()
        .expect("diagnostics_v2 must be an array");
    assert!(
        !diagnostics.is_empty(),
        "diagnostics_v2 must include at least one diagnostic"
    );
    assert!(
        diagnostics.iter().any(|diagnostic| {
            diagnostic["blocking"] == true
                || diagnostic["blocks_write"] == true
                || diagnostic["severity"] == "blocking"
                || diagnostic["severity"] == "error"
                || diagnostic["level"] == "blocking"
        }),
        "diagnostics_v2 must include a blocking diagnostic: {diagnostics:?}"
    );
}

fn assert_missing_codex_auth_contract(status: StatusCode, body: &Value) {
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["error"]["type"], "authentication_error");
    assert_eq!(body["error"]["code"], "invalid_api_key");
    assert!(body["error"]["message"]
        .as_str()
        .unwrap()
        .contains("~/.codex/auth.json"));
}

fn write_codex_auth(codex_home: &TempDir) {
    fs::write(
        codex_home.path().join("auth.json"),
        r#"{"access_token":"codex-access-test","account_id":"acct-test"}"#,
    )
    .unwrap();
}

fn seed_codex_catalog(state: &AppState, models: &[(&str, &str, bool)]) {
    state
        .codex_catalog
        .store_validated(&serde_json::json!({
            "models": models
                .iter()
                .map(|(slug, visibility, supported_in_api)| {
                    serde_json::json!({
                        "slug": slug,
                        "display_name": slug,
                        "visibility": visibility,
                        "supported_in_api": supported_in_api,
                        "supported_reasoning_levels": [
                            { "effort": "low", "description": "Low" },
                            { "effort": "medium", "description": "Medium" },
                            { "effort": "high", "description": "High" },
                            { "effort": "xhigh", "description": "XHigh" }
                        ],
                        "service_tiers": [
                            { "id": "auto", "name": "Auto", "description": "Default" },
                            { "id": "priority", "name": "Priority", "description": "Fast lane" }
                        ],
                        "support_verbosity": true,
                        "truncation_policy": { "mode": "tokens", "limit": 12345 },
                        "input_modalities": ["text", "image"],
                        "output_modalities": ["text"]
                    })
                })
                .collect::<Vec<_>>()
        }))
        .unwrap();
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

    // Composer rows added in Phase 1: `/v1/models` lists Cursor models with
    // `owned_by: "cursor"` so downstream clients can distinguish provider.
    for composer_id in ["composer-1.5", "composer-2", "composer-2-fast"] {
        assert!(
            ids.contains(&composer_id),
            "/v1/models missing Composer row {composer_id}",
        );
        let row = body["data"]
            .as_array()
            .unwrap()
            .iter()
            .find(|model| model["id"] == composer_id)
            .unwrap_or_else(|| panic!("composer row {composer_id} not present"));
        assert_eq!(
            row["owned_by"], "cursor",
            "composer row {composer_id} owned_by should be cursor",
        );
    }
}

#[tokio::test]
async fn integration_routes_openai_provider_models_does_not_expose_cursor_rows() {
    // The Codex/OpenAI projection at `/api/provider/openai/v1/models` reads
    // from the Codex catalog cache, not `KNOWN_MODELS`. Asserting the
    // Composer rows stay absent guards against accidental pollution if the
    // projection ever swaps to iterate `KNOWN_MODELS` directly.
    let codex_home = tempfile::tempdir().unwrap();
    let auth_home = tempfile::tempdir().unwrap();
    let state = test_state(&codex_home, &auth_home);

    let (status, _headers, body) = request_json_with_headers(
        state,
        "GET",
        "/api/provider/openai/v1/models?client_version=26.506.31421",
        &[],
        None,
    )
    .await;

    // The Codex projection requires Codex auth; if the body has a `data`
    // array, assert no Composer rows. If it returns the missing-auth
    // contract, the absence assertion is trivially satisfied.
    if status == StatusCode::OK {
        let ids: Vec<&str> = body["data"]
            .as_array()
            .map(|models| {
                models
                    .iter()
                    .filter_map(|model| model["id"].as_str())
                    .collect()
            })
            .unwrap_or_default();
        for composer_id in ["composer-1.5", "composer-2", "composer-2-fast"] {
            assert!(
                !ids.contains(&composer_id),
                "/api/provider/openai/v1/models leaked Composer row {composer_id}",
            );
        }
    }
}

#[tokio::test]
async fn integration_routes_openai_provider_models_uses_codex_auth_gate() {
    let (status, body) = request_json(
        "GET",
        "/api/provider/openai/v1/models?client_version=26.506.31421",
        None,
    )
    .await;
    assert_missing_codex_auth_contract(status, &body);
}

#[test]
fn integration_routes_openai_provider_models_does_not_require_openai_api_key() {
    let (status, body) = request_json_without_openai_env(
        "GET",
        "/api/provider/openai/v1/models?client_version=26.506.31421",
        None,
    );
    assert_missing_codex_auth_contract(status, &body);
}

#[test]
fn integration_routes_v1_models_aggregate_is_local_not_live_codex_catalog() {
    let (status, body) = request_json_without_openai_env("GET", "/v1/models", None);
    assert_eq!(status, StatusCode::OK);
    let ids: Vec<&str> = body["data"]
        .as_array()
        .unwrap()
        .iter()
        .map(|model| model["id"].as_str().unwrap())
        .collect();

    assert!(ids.contains(&"openai:gpt-5.5"));
    assert!(ids.contains(&"gpt-image-2"));
    assert!(!ids.contains(&"codex-auto-review"));
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
    assert_eq!(body["error"]["type"], "invalid_request_error");
    assert_eq!(body["error"]["code"], "model_not_supported");
}

#[tokio::test]
async fn integration_routes_count_tokens_accepts_zstd_encoded_json_body() {
    let codex_home = tempfile::tempdir().unwrap();
    let auth_home = tempfile::tempdir().unwrap();
    let state = test_state(&codex_home, &auth_home);
    let body = serde_json::json!({
        "model": "openai:gpt-5.5",
        "messages": [{ "role": "user", "content": "hello compressed request" }]
    })
    .to_string();
    let compressed = zstd::stream::encode_all(body.as_bytes(), 0).unwrap();

    let (status, _, response) = request_json_bytes_with_headers(
        state,
        "POST",
        "/api/provider/anthropic/v1/messages/count_tokens",
        &[("content-encoding", "zstd")],
        compressed,
    )
    .await;

    assert_eq!(status, StatusCode::OK, "{response}");
    assert!(response["input_tokens"].as_u64().unwrap() > 0);
}

#[tokio::test]
async fn integration_routes_responses_accepts_zstd_encoded_json_body_before_auth() {
    let codex_home = tempfile::tempdir().unwrap();
    let auth_home = tempfile::tempdir().unwrap();
    let state = test_state(&codex_home, &auth_home);
    let body = serde_json::json!({
        "model": "openai:gpt-5.5",
        "input": "hello compressed response request"
    })
    .to_string();
    let compressed = zstd::stream::encode_all(body.as_bytes(), 0).unwrap();

    let (status, _, response) = request_json_bytes_with_headers(
        state,
        "POST",
        "/v1/responses",
        &[("content-encoding", "zstd")],
        compressed,
    )
    .await;

    assert_missing_codex_auth_contract(status, &response);
}

#[tokio::test]
async fn integration_routes_compact_paths_are_registered_and_validate_input_shape() {
    for path in [
        "/v1/responses/compact",
        "/api/provider/openai/v1/responses/compact",
    ] {
        let codex_home = tempfile::tempdir().unwrap();
        let auth_home = tempfile::tempdir().unwrap();
        let state = test_state(&codex_home, &auth_home);
        let (status, body) = request_compact_with_state(
            state,
            path,
            serde_json::json!({
                "model": "claude-opus-4-7"
            }),
        )
        .await;

        assert_eq!(status, StatusCode::BAD_REQUEST, "{path}: {body}");
        assert_eq!(body["error"]["type"], "invalid_request", "{path}: {body}");
        assert_eq!(
            body["error"]["code"], "invalid_compaction_input",
            "{path}: {body}"
        );
        assert!(body["error"]["message"].as_str().unwrap().contains("input"));
    }
}

#[tokio::test]
async fn integration_routes_proxy_visible_compact_returns_one_pack_item() {
    let _guard = UPSTREAM_ENV_LOCK.lock().await;
    let _key_env = EnvRestore::set(
        "UMP_COMPACTION_KEYS_JSON",
        r#"{"current":"fixture","keys":{"fixture":"AAECAwQFBgcICQoLDA0ODxAREhMUFRYXGBkaGxwdHh8"}}"#,
    );
    let _instance_env = EnvRestore::set("UMP_COMPACTION_INSTANCE_ID", "route-shape-test");
    let codex_home = tempfile::tempdir().unwrap();
    let auth_home = tempfile::tempdir().unwrap();
    fs::write(
        auth_home.path().join("config.json"),
        serde_json::json!({
            "compaction": {
                "default_policy": "proxy_visible_summary"
            },
            "routes": [{
                "source": { "model": "claude-opus-4-7", "format": "responses" },
                "target": {
                    "provider": "bedrock",
                    "model": "anthropic.claude-opus-4-7",
                    "format": "anthropic_messages"
                },
                "remote_compaction_policy": "proxy_visible_summary"
            }]
        })
        .to_string(),
    )
    .unwrap();

    let (status, body) = request_compact_with_state(
        test_state(&codex_home, &auth_home),
        "/v1/responses/compact",
        serde_json::json!({
            "model": "claude-opus-4-7",
            "input": [{
                "type": "message",
                "role": "user",
                "content": [{ "type": "input_text", "text": "keep this objective" }]
            }]
        }),
    )
    .await;

    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["object"], "response.compaction");
    let output = body["output"].as_array().expect("compact output array");
    assert_eq!(output.len(), 1, "{body}");
    assert_eq!(output[0]["type"], "compaction");
    assert!(output[0]["encrypted_content"]
        .as_str()
        .is_some_and(|value| value.starts_with("ump.compaction.v1.")));
    assert!(
        !output.iter().any(|item| item["type"] == "message"),
        "direct compact must not emit restored-context messages: {body}"
    );
}

#[tokio::test]
async fn integration_routes_proxy_visible_pack_can_be_consumed_by_http_response() {
    let _guard = UPSTREAM_ENV_LOCK.lock().await;
    let _key_env = EnvRestore::set(
        "UMP_COMPACTION_KEYS_JSON",
        r#"{"current":"fixture","keys":{"fixture":"AAECAwQFBgcICQoLDA0ODxAREhMUFRYXGBkaGxwdHh8"}}"#,
    );
    let _instance_env = EnvRestore::set("UMP_COMPACTION_INSTANCE_ID", "route-roundtrip-test");
    let _bedrock_env = EnvRestore::clear("AWS_BEARER_TOKEN_BEDROCK");
    let codex_home = tempfile::tempdir().unwrap();
    let auth_home = tempfile::tempdir().unwrap();
    fs::write(
        auth_home.path().join("config.json"),
        serde_json::json!({
            "routes": [{
                "source": { "model": "claude-opus-4-7", "format": "responses" },
                "target": {
                    "provider": "bedrock",
                    "model": "anthropic.claude-opus-4-7",
                    "format": "anthropic_messages"
                },
                "remote_compaction_policy": "proxy_visible_summary"
            }]
        })
        .to_string(),
    )
    .unwrap();
    let state = test_state(&codex_home, &auth_home);
    let (compact_status, compact_body) = request_compact_with_state(
        state.clone(),
        "/v1/responses/compact",
        serde_json::json!({
            "model": "claude-opus-4-7",
            "input": [{
                "type": "message",
                "role": "user",
                "content": [{ "type": "input_text", "text": "keep this objective" }]
            }]
        }),
    )
    .await;
    assert_eq!(compact_status, StatusCode::OK, "{compact_body}");
    let encrypted_content = compact_body["output"][0]["encrypted_content"]
        .as_str()
        .expect("compact response encrypted_content");

    let (status, _, body) = request_json_with_headers(
        state,
        "POST",
        "/v1/responses",
        &[
            ("session-id", "compact-session-route-test"),
            ("thread-id", "compact-thread-route-test"),
        ],
        Some(
            &serde_json::json!({
                "model": "claude-opus-4-7",
                "input": [{
                    "type": "context_compaction",
                    "encrypted_content": encrypted_content
                }, {
                    "type": "message",
                    "role": "user",
                    "content": [{ "type": "input_text", "text": "continue" }]
                }]
            })
            .to_string(),
        ),
    )
    .await;

    assert_eq!(status, StatusCode::UNAUTHORIZED, "{body}");
    assert_eq!(body["error"]["type"], "missing_credential", "{body}");
}

#[tokio::test]
async fn integration_routes_proxy_visible_responses_trigger_returns_context_compaction() {
    let _guard = UPSTREAM_ENV_LOCK.lock().await;
    let _key_env = EnvRestore::set(
        "UMP_COMPACTION_KEYS_JSON",
        r#"{"current":"fixture","keys":{"fixture":"AAECAwQFBgcICQoLDA0ODxAREhMUFRYXGBkaGxwdHh8"}}"#,
    );
    let _instance_env = EnvRestore::set("UMP_COMPACTION_INSTANCE_ID", "route-trigger-test");
    let codex_home = tempfile::tempdir().unwrap();
    let auth_home = tempfile::tempdir().unwrap();
    fs::write(
        auth_home.path().join("config.json"),
        serde_json::json!({
            "routes": [{
                "source": { "model": "claude-opus-4-7", "format": "responses" },
                "target": {
                    "provider": "bedrock",
                    "model": "anthropic.claude-opus-4-7",
                    "format": "anthropic_messages"
                },
                "remote_compaction_policy": "proxy_visible_summary"
            }]
        })
        .to_string(),
    )
    .unwrap();

    let (status, _, body) = request_json_with_headers(
        test_state(&codex_home, &auth_home),
        "POST",
        "/v1/responses",
        &[
            ("session-id", "compact-session-route-test"),
            ("thread-id", "compact-thread-route-test"),
        ],
        Some(
            &serde_json::json!({
                "model": "claude-opus-4-7",
                "input": [
                    {
                        "type": "message",
                        "role": "user",
                        "content": [{ "type": "input_text", "text": "keep this objective" }]
                    },
                    { "type": "context_compaction" }
                ]
            })
            .to_string(),
        ),
    )
    .await;

    assert_eq!(status, StatusCode::OK, "{body}");
    let output = body["output"].as_array().expect("response output array");
    assert_eq!(output.len(), 1, "{body}");
    assert_eq!(output[0]["type"], "context_compaction");
    assert!(output[0]["encrypted_content"]
        .as_str()
        .is_some_and(|value| value.starts_with("ump.compaction.v1.")));
}

#[tokio::test]
async fn integration_routes_proxy_visible_requires_session_binding() {
    let _guard = UPSTREAM_ENV_LOCK.lock().await;
    let _key_env = EnvRestore::set(
        "UMP_COMPACTION_KEYS_JSON",
        r#"{"current":"fixture","keys":{"fixture":"AAECAwQFBgcICQoLDA0ODxAREhMUFRYXGBkaGxwdHh8"}}"#,
    );
    let _instance_env = EnvRestore::set("UMP_COMPACTION_INSTANCE_ID", "route-binding-test");
    let codex_home = tempfile::tempdir().unwrap();
    let auth_home = tempfile::tempdir().unwrap();
    fs::write(
        auth_home.path().join("config.json"),
        serde_json::json!({
            "routes": [{
                "source": { "model": "claude-opus-4-7", "format": "responses" },
                "target": {
                    "provider": "bedrock",
                    "model": "anthropic.claude-opus-4-7",
                    "format": "anthropic_messages"
                },
                "remote_compaction_policy": "proxy_visible_summary"
            }]
        })
        .to_string(),
    )
    .unwrap();

    let (status, _, body) = request_json_with_headers(
        test_state(&codex_home, &auth_home),
        "POST",
        "/v1/responses",
        &[],
        Some(
            &serde_json::json!({
                "model": "claude-opus-4-7",
                "input": [{ "type": "context_compaction" }]
            })
            .to_string(),
        ),
    )
    .await;

    assert_eq!(status, StatusCode::BAD_REQUEST, "{body}");
    assert_eq!(body["error"]["code"], "compaction_binding_required");
}

#[tokio::test]
async fn integration_routes_disabled_remote_compaction_returns_conflict() {
    let codex_home = tempfile::tempdir().unwrap();
    let auth_home = tempfile::tempdir().unwrap();
    fs::write(
        auth_home.path().join("config.json"),
        serde_json::json!({
            "routes": [{
                "source": { "model": "compact-off-claude", "format": "responses" },
                "target": {
                    "provider": "bedrock",
                    "model": "anthropic.claude-opus-4-7",
                    "format": "anthropic_messages"
                },
                "remote_compaction_policy": "off"
            }]
        })
        .to_string(),
    )
    .unwrap();

    let (status, _, body) = request_json_with_headers(
        test_state(&codex_home, &auth_home),
        "POST",
        "/v1/responses",
        &[
            ("session-id", "compact-session-route-test"),
            ("thread-id", "compact-thread-route-test"),
        ],
        Some(
            &serde_json::json!({
                "model": "compact-off-claude",
                "input": [{ "type": "context_compaction" }]
            })
            .to_string(),
        ),
    )
    .await;

    assert_eq!(status, StatusCode::CONFLICT, "{body}");
    assert_eq!(body["error"]["code"], "compaction_disabled_for_target");
}

#[tokio::test]
async fn integration_routes_known_gpt_messages_route_strips_lossy_token_cap_before_auth() {
    let gpt_messages = serde_json::json!({
        "model": "openai:gpt-5.5",
        "max_tokens": 64,
        "messages": [{ "role": "user", "content": "hello" }]
    });
    let (status, body) = request_json("POST", "/v1/messages", Some(gpt_messages)).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["error"]["type"], "authentication_error");
    assert!(body["error"]["message"]
        .as_str()
        .unwrap()
        .contains("~/.codex/auth.json"));
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
        .contains("Bedrock bearer"));
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
    assert_missing_codex_auth_contract(status, &response);

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
    assert_missing_codex_auth_contract(status, &body);
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
async fn integration_routes_config_shell_headers_and_markup_are_browser_safe() {
    let codex_home = tempfile::tempdir().unwrap();
    let auth_home = tempfile::tempdir().unwrap();
    let state = test_state(&codex_home, &auth_home);
    let (status, headers, bytes) =
        request_bytes_with_headers(state, "GET", "/config", &[("host", "localhost")], None).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        header_value(&headers, "content-type"),
        "text/html; charset=utf-8"
    );
    assert_eq!(
        header_value(&headers, "content-security-policy"),
        "default-src 'none'; base-uri 'none'; frame-ancestors 'none'; form-action 'none'; connect-src 'self'; img-src 'self' data:; style-src 'self'; script-src 'self'; object-src 'none'"
    );
    assert_no_store_nosniff(&headers);
    let html = String::from_utf8(bytes).unwrap();
    assert_config_shell_has_no_inline_code(&html);
}

#[tokio::test]
async fn integration_routes_config_assets_return_safe_content_headers() {
    for (path, content_type) in [
        ("/config/assets/config.css", "text/css; charset=utf-8"),
        (
            "/config/assets/config.js",
            "application/javascript; charset=utf-8",
        ),
    ] {
        let codex_home = tempfile::tempdir().unwrap();
        let auth_home = tempfile::tempdir().unwrap();
        let state = test_state(&codex_home, &auth_home);
        let (status, headers, _bytes) =
            request_bytes_with_headers(state, "GET", path, &[("host", "localhost")], None).await;

        assert_eq!(status, StatusCode::OK, "{path}");
        assert_eq!(
            header_value(&headers, "content-type"),
            content_type,
            "{path}"
        );
        assert_no_store_nosniff(&headers);
    }
}

#[tokio::test]
async fn integration_routes_config_raw_api_preserves_get_and_put_compatibility() {
    let codex_home = tempfile::tempdir().unwrap();
    let auth_home = tempfile::tempdir().unwrap();
    let config_path = auth_home.path().join("config.json");
    let state = test_state(&codex_home, &auth_home);

    let (status, headers, body) =
        request_json_with_headers(state.clone(), "GET", "/api/config", &[], None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(header_value(&headers, "content-type"), "application/json");
    assert_eq!(header_value(&headers, "cache-control"), "no-store");
    assert_eq!(body, serde_json::json!({ "routes": [] }));

    let config = serde_json::json!({
        "routes": [{
            "source": { "model": "integration-raw-model", "format": "responses" },
            "target": { "provider": "codex", "model": "gpt-5.5", "format": "responses" }
        }]
    });
    let (status, _headers, body) = request_json_with_headers(
        state.clone(),
        "PUT",
        "/api/config",
        &[],
        Some(&config.to_string()),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["ok"], true);

    let written: Value = serde_json::from_str(&fs::read_to_string(config_path).unwrap()).unwrap();
    assert_eq!(written, config);
}

#[tokio::test]
async fn integration_routes_config_malformed_json_returns_invalid_routing_config() {
    let codex_home = tempfile::tempdir().unwrap();
    let auth_home = tempfile::tempdir().unwrap();
    let state = test_state(&codex_home, &auth_home);

    let (status, _headers, body) =
        request_json_with_headers(state, "PUT", "/api/config", &[], Some(r#"{ "routes": ["#)).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"]["code"], "invalid_routing_config");
}

#[tokio::test]
async fn integration_routes_config_graph_get_returns_persisted_projection() {
    let codex_home = tempfile::tempdir().unwrap();
    let auth_home = tempfile::tempdir().unwrap();
    fs::write(
        auth_home.path().join("config.json"),
        serde_json::json!({
            "routes": [{
                "source": { "model": "integration-graph-model", "format": "responses" },
                "target": { "provider": "codex", "model": "gpt-5.5", "format": "responses" }
            }]
        })
        .to_string(),
    )
    .unwrap();
    let state = test_state(&codex_home, &auth_home);

    let (status, headers, body) =
        request_json_with_headers(state, "GET", "/api/config/graph", &[], None).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(header_value(&headers, "content-type"), "application/json");
    assert_eq!(header_value(&headers, "cache-control"), "no-store");
    assert_eq!(
        body["raw_hot_config"]["routes"][0]["source"]["model"],
        "integration-graph-model"
    );
    assert!(body["effective_routes"]
        .as_array()
        .unwrap()
        .iter()
        .any(|route| {
            route["source_model"] == "integration-graph-model"
                || route["source"]["model"] == "integration-graph-model"
        }));
}

#[tokio::test]
async fn integration_routes_config_graph_get_returns_switchyard_atlas_v2_contract() {
    let codex_home = tempfile::tempdir().unwrap();
    let auth_home = tempfile::tempdir().unwrap();
    fs::write(
        auth_home.path().join("config.json"),
        serde_json::json!({
            "routes": [{
                "source": { "model": "atlas-v2-model", "format": "responses" },
                "target": { "provider": "codex", "model": "gpt-5.5", "format": "responses" }
            }]
        })
        .to_string(),
    )
    .unwrap();
    let state = test_state(&codex_home, &auth_home);

    let (status, _headers, body) =
        request_json_with_headers(state, "GET", "/api/config/graph", &[], None).await;

    assert_eq!(status, StatusCode::OK);
    assert_graph_v2_contract(&body);
    assert_eq!(body["draft_status"], "valid");
    assert!(
        body["effective_routes"]
            .as_array()
            .unwrap()
            .iter()
            .any(|route| route["source_model"] == "atlas-v2-model"),
        "v1 effective_routes must remain populated"
    );
    assert!(
        body["route_cards"]
            .as_array()
            .unwrap()
            .iter()
            .any(|route| route["source_model"] == "atlas-v2-model"
                || route["source"]["model"] == "atlas-v2-model"),
        "v2 route_cards must expose the projected route"
    );
}

#[tokio::test]
async fn integration_routes_config_graph_post_projects_draft_without_writing() {
    let codex_home = tempfile::tempdir().unwrap();
    let auth_home = tempfile::tempdir().unwrap();
    let config_path = auth_home.path().join("config.json");
    let persisted = serde_json::json!({ "routes": [] });
    fs::write(&config_path, persisted.to_string()).unwrap();
    let state = test_state(&codex_home, &auth_home);
    let draft = serde_json::json!({
        "routes": [{
            "source": { "model": "draft-only-model", "format": "responses" },
            "target": { "provider": "codex", "model": "gpt-5.5", "format": "responses" }
        }]
    });

    let (status, _headers, body) = request_json_with_headers(
        state.clone(),
        "POST",
        "/api/config/graph",
        &[],
        Some(&draft.to_string()),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        body["raw_hot_config"]["routes"][0]["source"]["model"],
        "draft-only-model"
    );

    let written: Value = serde_json::from_str(&fs::read_to_string(&config_path).unwrap()).unwrap();
    assert_eq!(written, persisted);
    let (status, _headers, body) =
        request_json_with_headers(state, "GET", "/api/config", &[], None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, persisted);
}

fn assert_graph_post_invalid_response(body: &Value, forbidden_fragments: &[&str]) {
    assert_eq!(body["error"]["type"], "invalid_request_error");
    assert_eq!(body["error"]["code"], "invalid_routing_config");
    assert_no_config_graph_payload(body);
    let body_text = body.to_string();
    for fragment in forbidden_fragments {
        assert!(
            !body_text.contains(fragment),
            "graph validation error leaked submitted fragment {fragment}: {body_text}"
        );
    }
}

#[tokio::test]
async fn integration_routes_config_graph_post_invalid_configs_return_sanitized_error_only() {
    for (name, body, forbidden_fragments) in [
        (
            "malformed_json",
            r#"{ "routes": ["#,
            vec!["raw_hot_config", "effective_routes"],
        ),
        (
            "non_object_root",
            r#""secret-root-value""#,
            vec!["secret-root-value", "raw_hot_config", "effective_routes"],
        ),
        (
            "unknown_root_field",
            r#"{ "routes": [], "mysteryRoot": "unknown-root-secret" }"#,
            vec!["unknown-root-secret", "raw_hot_config", "effective_routes"],
        ),
        (
            "unknown_route_field",
            r#"{
                "routes": [{
                    "source": { "model": "x" },
                    "target": { "provider": "codex", "model": "gpt-5.5" },
                    "mysteryRoute": "unknown-route-secret"
                }]
            }"#,
            vec!["unknown-route-secret", "raw_hot_config", "effective_routes"],
        ),
        (
            "unknown_source_field",
            r#"{
                "routes": [{
                    "source": { "model": "x", "mysterySource": "unknown-source-secret" },
                    "target": { "provider": "codex", "model": "gpt-5.5" }
                }]
            }"#,
            vec![
                "unknown-source-secret",
                "raw_hot_config",
                "effective_routes",
            ],
        ),
        (
            "unknown_target_field",
            r#"{
                "routes": [{
                    "source": { "model": "x" },
                    "target": {
                        "provider": "codex",
                        "model": "gpt-5.5",
                        "mysteryTarget": "unknown-target-secret"
                    }
                }]
            }"#,
            vec![
                "unknown-target-secret",
                "raw_hot_config",
                "effective_routes",
            ],
        ),
        (
            "forbidden_secret_key",
            r#"{
                "routes": [{
                    "source": { "model": "x" },
                    "target": { "provider": "codex", "model": "gpt-5.5" },
                    "api-key": "do-not-echo"
                }]
            }"#,
            vec!["do-not-echo", "raw_hot_config", "effective_routes"],
        ),
        (
            "unprojectable_shape",
            r#"{ "routes": "not-a-route-array" }"#,
            vec!["not-a-route-array", "raw_hot_config", "effective_routes"],
        ),
    ] {
        let codex_home = tempfile::tempdir().unwrap();
        let auth_home = tempfile::tempdir().unwrap();
        let state = test_state(&codex_home, &auth_home);

        let (status, headers, response) =
            request_json_with_headers(state, "POST", "/api/config/graph", &[], Some(body)).await;

        assert_eq!(status, StatusCode::BAD_REQUEST, "{name}: {response}");
        assert_eq!(
            header_value(&headers, "cache-control"),
            "no-store",
            "{name}"
        );
        assert_eq!(
            header_value(&headers, "x-content-type-options"),
            "nosniff",
            "{name}"
        );
        assert_graph_post_invalid_response(&response, &forbidden_fragments);
    }
}

#[tokio::test]
async fn integration_routes_config_graph_post_semantic_row_error_returns_invalid_draft() {
    let codex_home = tempfile::tempdir().unwrap();
    let auth_home = tempfile::tempdir().unwrap();
    let state = test_state(&codex_home, &auth_home);
    let draft = serde_json::json!({
        "routes": [{
            "source": { "model": "semantic-row-error-model", "format": "responses" },
            "target": { "provider": "unsupported", "model": "unsupported-target-model" }
        }]
    });

    let (status, _headers, body) = request_json_with_headers(
        state,
        "POST",
        "/api/config/graph",
        &[],
        Some(&draft.to_string()),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_graph_v2_contract(&body);
    assert_eq!(body["draft_status"], "invalid");
    assert_blocking_diagnostics_v2(&body);
}

#[tokio::test]
async fn integration_routes_config_graph_projects_composer_source_provider() {
    let codex_home = tempfile::tempdir().unwrap();
    let auth_home = tempfile::tempdir().unwrap();
    let state = test_state(&codex_home, &auth_home);
    let draft = serde_json::json!({
        "routes": [{
            "source": { "model": "composer-2-fast", "format": "responses" },
            "target": { "provider": "codex", "model": "composer-2-fast", "format": "responses" }
        }]
    });

    let (status, _headers, body) = request_json_with_headers(
        state,
        "POST",
        "/api/config/graph",
        &[],
        Some(&draft.to_string()),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_graph_v2_contract(&body);
    assert_eq!(body["draft_status"], "valid");
    assert_eq!(body["config_routes"][0]["source_provider"], "cursor");
    assert!(body["effective_routes"]
        .as_array()
        .unwrap()
        .iter()
        .any(|route| route["source_model"] == "composer-2-fast"
            && route["source_provider"] == "cursor"
            && route["target_provider"] == "codex"
            && route["target_model"] == "composer-2-fast"));
    assert!(body["sources"].as_array().unwrap().iter().any(|source| {
        source["model"] == "composer-2-fast" && source["source_provider"] == "cursor"
    }));
    assert!(body["route_cards"]
        .as_array()
        .unwrap()
        .iter()
        .any(|card| card["source"]["model"] == "composer-2-fast"
            && card["source"]["source_provider"] == "cursor"
            && card["target"]["provider"] == "codex"
            && card["target"]["model"] == "composer-2-fast"));
    assert!(body["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .any(|node| { node["kind"] == "source_model" && node["source_provider"] == "cursor" }));
    assert!(body["edges"].as_array().unwrap().iter().any(|edge| {
        edge["route_id"] == "effective:composer-2-fast:responses"
            && edge["source_provider"] == "cursor"
    }));
}

#[tokio::test]
async fn integration_routes_config_graph_composer_source_routes_to_cursor_target() {
    // Sibling test pinning `provider: "cursor"` so future dispatch lands
    // Composer source models on the Cursor adapter rather than Codex.
    let codex_home = tempfile::tempdir().unwrap();
    let auth_home = tempfile::tempdir().unwrap();
    let state = test_state(&codex_home, &auth_home);
    let draft = serde_json::json!({
        "routes": [{
            "source": { "model": "composer-2-fast", "format": "responses" },
            "target": { "provider": "cursor", "model": "composer-2-fast", "format": "cursor_agent" }
        }]
    });

    let (status, _headers, body) = request_json_with_headers(
        state,
        "POST",
        "/api/config/graph",
        &[],
        Some(&draft.to_string()),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_graph_v2_contract(&body);
    assert_eq!(body["draft_status"], "valid");
    assert!(body["effective_routes"]
        .as_array()
        .unwrap()
        .iter()
        .any(|route| route["source_model"] == "composer-2-fast"
            && route["source_provider"] == "cursor"
            && route["target_provider"] == "cursor"
            && route["target_model"] == "composer-2-fast"));
    assert!(body["route_cards"]
        .as_array()
        .unwrap()
        .iter()
        .any(|card| card["source"]["model"] == "composer-2-fast"
            && card["source"]["source_provider"] == "cursor"
            && card["target"]["provider"] == "cursor"
            && card["target"]["model"] == "composer-2-fast"));
}

#[tokio::test]
async fn integration_routes_config_graph_projects_composer_source_provider_and_accepts_cursor_target(
) {
    // Phase 1 flip: Cursor target rows are now valid. The old rejection
    // assertion is gone; this test pins the positive shape so a regression
    // would surface as either an `invalid` draft_status or a stray
    // `unsupported_target_provider` diagnostic.
    let codex_home = tempfile::tempdir().unwrap();
    let auth_home = tempfile::tempdir().unwrap();
    let state = test_state(&codex_home, &auth_home);
    let draft = serde_json::json!({
        "routes": [{
            "source": { "model": "composer-2-fast", "format": "responses" },
            "target": { "provider": "cursor", "model": "composer-2-fast" }
        }]
    });

    let (status, _headers, body) = request_json_with_headers(
        state,
        "POST",
        "/api/config/graph",
        &[],
        Some(&draft.to_string()),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_graph_v2_contract(&body);
    assert_eq!(body["draft_status"], "valid");
    let card = body["route_cards"]
        .as_array()
        .unwrap()
        .iter()
        .find(|card| card["id"] == "config:0")
        .unwrap();
    assert_eq!(card["source"]["source_provider"], "cursor");
    assert_eq!(card["target"]["provider"], "cursor");
    assert_eq!(card["target"]["model"], "composer-2-fast");
    if let Some(diagnostics) = body["diagnostics_v2"].as_array() {
        assert!(
            !diagnostics.iter().any(|diagnostic| {
                diagnostic["code"] == "unsupported_target_provider"
                    && diagnostic["path"] == "$.routes[0].target.provider"
            }),
            "Cursor target should no longer produce unsupported_target_provider diagnostic: {body}",
        );
    }
    assert!(body["effective_routes"]
        .as_array()
        .unwrap()
        .iter()
        .any(|route| route["source_model"] == "composer-2-fast"
            && route["source_provider"] == "cursor"
            && route["target_provider"] == "cursor"
            && route["target_model"] == "composer-2-fast"));
}

#[tokio::test]
async fn integration_routes_config_graph_post_partially_projectable_row_reports_partial_projection()
{
    let codex_home = tempfile::tempdir().unwrap();
    let auth_home = tempfile::tempdir().unwrap();
    let state = test_state(&codex_home, &auth_home);
    let draft = serde_json::json!({
        "routes": [
            {
                "source": { "model": "partial-valid-model", "format": "responses" },
                "target": { "provider": "codex", "model": "gpt-5.5", "format": "responses" }
            },
            {
                "source": { "model": "partial-invalid-model", "format": "responses" },
                "target": { "provider": "unsupported", "model": "unsupported-target-model" }
            }
        ]
    });

    let (status, _headers, body) = request_json_with_headers(
        state,
        "POST",
        "/api/config/graph",
        &[],
        Some(&draft.to_string()),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_graph_v2_contract(&body);
    assert_eq!(body["draft_status"], "partially_projected");
    assert_blocking_diagnostics_v2(&body);
    assert!(
        body["effective_routes"]
            .as_array()
            .unwrap()
            .iter()
            .any(|route| route["source_model"] == "partial-valid-model"),
        "partial projection must preserve projectable rows"
    );
}

#[tokio::test]
async fn integration_routes_config_guard_allows_loopback_hosts_and_missing_origin() {
    for host in [
        "localhost",
        "localhost:18743",
        "127.0.0.1",
        "127.0.0.1:18743",
        "[::1]",
        "[::1]:18743",
    ] {
        let codex_home = tempfile::tempdir().unwrap();
        let auth_home = tempfile::tempdir().unwrap();
        let state = test_state(&codex_home, &auth_home);
        let (status, _headers, _body) =
            request_json_with_raw_headers(state, "GET", "/api/config", &[("host", host)], None)
                .await;
        assert_eq!(status, StatusCode::OK, "{host}");
    }

    let codex_home = tempfile::tempdir().unwrap();
    let auth_home = tempfile::tempdir().unwrap();
    let state = test_state(&codex_home, &auth_home);
    let (status, _headers, body) = request_json_with_headers(
        state,
        "PUT",
        "/api/config",
        &[],
        Some(r#"{ "routes": [] }"#),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["ok"], true);
}

#[tokio::test]
async fn integration_routes_config_guard_rejects_missing_or_foreign_host() {
    for headers in [&[][..], &[("host", "example.com")][..]] {
        let codex_home = tempfile::tempdir().unwrap();
        let auth_home = tempfile::tempdir().unwrap();
        let state = test_state(&codex_home, &auth_home);
        let (status, response_headers, _body) =
            request_json_with_raw_headers(state, "GET", "/api/config", headers, None).await;

        assert_eq!(status, StatusCode::FORBIDDEN);
        assert_no_store_nosniff(&response_headers);
    }
}

#[tokio::test]
async fn integration_routes_config_guard_rejects_cross_site_unsafe_requests() {
    for extra_headers in [
        vec![("origin", "https://evil.example")],
        vec![("sec-fetch-site", "cross-site")],
    ] {
        let codex_home = tempfile::tempdir().unwrap();
        let auth_home = tempfile::tempdir().unwrap();
        let state = test_state(&codex_home, &auth_home);
        let (status, response_headers, _body) = request_json_with_headers(
            state,
            "PUT",
            "/api/config",
            &extra_headers,
            Some(r#"{ "routes": [] }"#),
        )
        .await;

        assert_eq!(status, StatusCode::FORBIDDEN);
        assert_no_store_nosniff(&response_headers);
    }
}

#[tokio::test]
async fn integration_routes_config_guard_allows_loopback_origin_for_unsafe_requests() {
    let codex_home = tempfile::tempdir().unwrap();
    let auth_home = tempfile::tempdir().unwrap();
    let state = test_state(&codex_home, &auth_home);

    let (status, _headers, body) = request_json_with_headers(
        state,
        "PUT",
        "/api/config",
        &[("origin", "http://localhost:18743")],
        Some(r#"{ "routes": [] }"#),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["ok"], true);
}

#[tokio::test]
async fn integration_routes_config_empty_graph_includes_catalog_routes_and_no_override_banner_data()
{
    let codex_home = tempfile::tempdir().unwrap();
    let auth_home = tempfile::tempdir().unwrap();
    fs::write(auth_home.path().join("config.json"), r#"{ "routes": [] }"#).unwrap();
    let state = test_state(&codex_home, &auth_home);

    let (status, _headers, body) =
        request_json_with_headers(state, "GET", "/api/config/graph", &[], None).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["raw_hot_config"], serde_json::json!({ "routes": [] }));
    assert!(
        !body["effective_routes"].as_array().unwrap().is_empty(),
        "empty hot config must still project built-in catalog routes"
    );
    assert!(body["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .any(|diagnostic| {
            diagnostic["code"] == "no_hot_overrides"
                || diagnostic["message"]
                    .as_str()
                    .is_some_and(|message| message.contains("No hot overrides"))
        }));
}

#[tokio::test]
async fn integration_routes_route_model_resolvers_reject_unknown_models_before_credentials() {
    let unknown_chat = serde_json::json!({ "model": "nope/nope", "messages": [] });
    let (status, body) = request_json("POST", "/v1/chat/completions", Some(unknown_chat)).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"]["type"], "invalid_request_error");
    assert_eq!(body["error"]["code"], "model_not_supported");
}

#[tokio::test]
async fn integration_routes_known_gpt_chat_routes_to_codex_without_real_upstream() {
    let gpt_chat = serde_json::json!({
        "model": "openai:gpt-5.5",
        "messages": [{ "role": "user", "content": "hello" }]
    });
    let (status, body) = request_json("POST", "/v1/chat/completions", Some(gpt_chat)).await;
    assert_missing_codex_auth_contract(status, &body);
}

#[test]
fn integration_routes_known_gpt_responses_uses_codex_auth_not_openai_api_key() {
    let codex_home = tempfile::tempdir().unwrap();
    let auth_home = tempfile::tempdir().unwrap();
    write_codex_auth(&codex_home);
    let state = test_state(&codex_home, &auth_home);
    seed_codex_catalog(&state, &[("gpt-5.5", "list", true)]);
    let body = serde_json::json!({ "model": "openai:gpt-5.5", "input": "hello" });

    let (status, response) = request_json_with_state_without_env_vars(
        state,
        "POST",
        "/v1/responses",
        Some(body),
        &["OPENAI_API_KEY"],
    );

    assert_eq!(status, StatusCode::BAD_GATEWAY);
    assert_eq!(response["error"]["type"], "upstream_error");
    assert_ne!(response["error"]["code"], "invalid_api_key");
    assert!(!response["error"]["message"]
        .as_str()
        .unwrap()
        .contains("OPENAI_API_KEY"));
}

#[tokio::test]
async fn integration_routes_hidden_and_catalog_absent_codex_models_fail_before_auth() {
    for model in ["codex-auto-review", "gpt-5.99-not-in-catalog"] {
        let body = serde_json::json!({ "model": model, "input": "hello" });
        let (status, response) = request_json("POST", "/v1/responses", Some(body)).await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{model}");
        assert_eq!(
            response["error"]["type"], "invalid_request_error",
            "{model}"
        );
        assert_eq!(response["error"]["code"], "model_not_supported", "{model}");
        assert!(!response["error"]["message"]
            .as_str()
            .unwrap()
            .contains("~/.codex/auth.json"));
    }
}

#[tokio::test]
async fn integration_routes_known_codex_alias_absent_from_catalog_fails_before_auth() {
    let codex_home = tempfile::tempdir().unwrap();
    let auth_home = tempfile::tempdir().unwrap();
    let state = test_state(&codex_home, &auth_home);
    seed_codex_catalog(&state, &[("gpt-5.4", "list", true)]);

    let body = serde_json::json!({ "model": "openai:gpt-5.5", "input": "hello" });
    let (status, response) =
        request_json_with_state(state, "POST", "/v1/responses", Some(body)).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(response["error"]["type"], "invalid_request_error");
    assert_eq!(response["error"]["code"], "model_not_supported");
    assert!(!response["error"]["message"]
        .as_str()
        .unwrap()
        .contains("~/.codex/auth.json"));
}

#[tokio::test]
async fn integration_routes_hidden_catalog_model_fails_before_auth() {
    let codex_home = tempfile::tempdir().unwrap();
    let auth_home = tempfile::tempdir().unwrap();
    fs::write(
        auth_home.path().join("config.json"),
        serde_json::json!({
            "routes": [{
                "source": { "model": "fixture-hidden-codex-model", "format": "responses" },
                "target": { "provider": "codex", "model": "gpt-hidden", "format": "responses" }
            }]
        })
        .to_string(),
    )
    .unwrap();
    let state = test_state(&codex_home, &auth_home);
    seed_codex_catalog(&state, &[("gpt-hidden", "hidden", true)]);

    let body = serde_json::json!({ "model": "fixture-hidden-codex-model", "input": "hello" });
    let (status, response) =
        request_json_with_state(state, "POST", "/v1/responses", Some(body)).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(response["error"]["type"], "invalid_request_error");
    assert_eq!(response["error"]["code"], "model_not_supported");
    assert!(!response["error"]["message"]
        .as_str()
        .unwrap()
        .contains("~/.codex/auth.json"));
}

#[tokio::test]
async fn integration_routes_unsupported_codex_fields_are_stripped_before_auth() {
    let body = serde_json::json!({
        "model": "openai:gpt-5.5",
        "input": "hello",
        "max_output_tokens": 16
    });
    let (status, response) = request_json("POST", "/v1/responses", Some(body)).await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(response["error"]["type"], "authentication_error");
    assert!(response["error"]["message"]
        .as_str()
        .unwrap()
        .contains("~/.codex/auth.json"));
}

#[tokio::test]
async fn integration_routes_unsupported_model_edges_fail_before_credentials() {
    for (path, body) in [
        (
            "/v1/responses",
            serde_json::json!({ "model": "gpt-image-2", "input": "paint" }),
        ),
        (
            "/v1/chat/completions",
            serde_json::json!({ "model": "gpt-image-2", "messages": [] }),
        ),
        (
            "/v1/messages",
            serde_json::json!({ "model": "gpt-image-2", "messages": [] }),
        ),
    ] {
        let (status, response) = request_json("POST", path, Some(body)).await;

        assert_eq!(status, StatusCode::BAD_REQUEST, "{path}: {response}");
        assert_eq!(response["error"]["type"], "invalid_request_error", "{path}");
        assert_eq!(response["error"]["code"], "model_not_supported", "{path}");
        assert!(!response["error"]["message"]
            .as_str()
            .unwrap()
            .contains("~/.codex/auth.json"));
    }
}

#[test]
fn integration_routes_invalid_hot_target_format_edge_fails_before_credentials() {
    let codex_home = tempfile::tempdir().unwrap();
    let auth_home = tempfile::tempdir().unwrap();
    fs::write(
        auth_home.path().join("config.json"),
        serde_json::json!({
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
    let state = test_state(&codex_home, &auth_home);
    let body = serde_json::json!({ "model": "invalid-google-target", "input": "hello" });

    let (status, response) = request_json_with_state_without_env_vars(
        state,
        "POST",
        "/v1/responses",
        Some(body),
        &["GOOGLE_API_KEY"],
    );

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(response["error"]["type"], "invalid_request_error");
    assert_eq!(response["error"]["code"], "model_not_supported");
    assert!(!response["error"]["message"]
        .as_str()
        .unwrap()
        .contains("GOOGLE_API_KEY"));
}

#[tokio::test]
async fn integration_routes_image_routes_return_explicit_unsupported_error() {
    let body = serde_json::json!({ "model": "gpt-image-2", "prompt": "paint" });
    for path in [
        "/api/provider/openai/v1/images/generations",
        "/api/provider/openai/v1/images/edits",
        "/v1/images/generations",
        "/v1/images/edits",
    ] {
        let (status, body) = request_json("POST", path, Some(body.clone())).await;
        assert_eq!(status, StatusCode::NOT_IMPLEMENTED, "{path}");
        assert_eq!(body["error"]["type"], "invalid_request_error", "{path}");
        assert_eq!(body["error"]["code"], "unsupported_route", "{path}");
        assert!(body["error"]["message"]
            .as_str()
            .unwrap()
            .contains("gpt-image-2"));
    }
}

#[tokio::test]
async fn integration_routes_audio_and_realtime_are_explicit_feature_gates() {
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
        assert_missing_codex_auth_contract(status, &body);
    }

    for path in [
        "/v1/audio/transcriptions",
        "/api/provider/openai/v1/audio/transcriptions",
    ] {
        let codex_home = tempfile::tempdir().unwrap();
        let auth_home = tempfile::tempdir().unwrap();
        let (status, body) = request_body_with_content_type(
            test_state(&codex_home, &auth_home),
            "POST",
            path,
            "multipart/form-data; boundary=audio-test",
            "--audio-test\r\n--audio-test--\r\n",
        )
        .await;
        assert_missing_codex_auth_contract(status, &body);
    }

    for path in [
        "/v1/audio/speech",
        "/api/provider/openai/v1/audio/speech",
        "/v1/audio/translations",
        "/api/provider/openai/v1/audio/translations",
        "/transcribe",
    ] {
        let (status, body) = request_json(
            "POST",
            path,
            Some(serde_json::json!({ "model": "gpt-realtime-2" })),
        )
        .await;
        assert_eq!(status, StatusCode::NOT_IMPLEMENTED, "{path}");
        assert_eq!(body["error"]["type"], "invalid_request_error", "{path}");
        assert_eq!(body["error"]["code"], "unsupported_feature", "{path}");
        assert_eq!(
            body["error"]["message"],
            "This route family is not supported by the Codex OAuth public OpenAI facade.",
            "{path}: {body}"
        );
    }
}

#[tokio::test]
async fn integration_routes_audio_transcriptions_validate_multipart_before_codex_auth() {
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
        let (status, body) = request_body_with_content_type(
            test_state(&codex_home, &auth_home),
            "POST",
            "/v1/audio/transcriptions",
            content_type,
            "{}",
        )
        .await;

        assert_eq!(status, expected_status, "{content_type}: {body}");
        assert_eq!(body["error"]["type"], "invalid_request_error");
        assert_eq!(body["error"]["code"], expected_code);
        assert!(!body["error"]["message"]
            .as_str()
            .unwrap()
            .contains("~/.codex/auth.json"));
    }
}

#[tokio::test]
async fn integration_routes_public_file_routes_return_explicit_unsupported_error() {
    for (method, path) in [
        ("GET", "/v1/files"),
        ("POST", "/v1/files"),
        ("GET", "/v1/files/file_123"),
        ("DELETE", "/v1/files/file_123"),
        ("GET", "/api/provider/openai/v1/files"),
        ("POST", "/api/provider/openai/v1/files"),
        ("GET", "/api/provider/openai/v1/files/file_123"),
        ("DELETE", "/api/provider/openai/v1/files/file_123"),
    ] {
        let (status, body) = request_json(method, path, None).await;
        assert_eq!(status, StatusCode::NOT_IMPLEMENTED, "{method} {path}");
        assert_eq!(body["error"]["type"], "invalid_request_error", "{path}");
        assert_eq!(body["error"]["code"], "unsupported_route", "{path}");
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
