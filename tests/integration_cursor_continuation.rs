//! Cursor `previous_response_id` continuation policy tests.

mod common;

use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
};
use serde_json::{json, Value};
use std::{collections::HashMap, time::Instant};
use tower::ServiceExt;
use unified_model_proxy_v2::{
    build_router,
    cursor_agent::{CursorContinuationKey, CursorRoute},
    model_alias::{Provider, TargetFormat},
    state::NewResponseStateRecord,
    upstream::cursor::session::ConversationState,
};

#[tokio::test]
async fn cursor_continuation_unknown_previous_response_id_returns_400() {
    let homes = common::TestHomes::new();
    let app = build_router(homes.state.clone());
    let body = json!({
        "model": "composer-2-fast",
        "input": "follow-up",
        "previous_response_id": "resp_missing_0001",
    });

    let parsed = post_responses(app, body).await;
    assert_eq!(parsed.0, StatusCode::BAD_REQUEST);
    assert_eq!(error_code(&parsed.1), "unknown_previous_response_id");
}

#[tokio::test]
async fn cursor_continuation_model_drift_rejected_before_credentials() {
    let homes = common::TestHomes::new();
    prime_cursor_continuation(&homes.state, "resp_prior", "conv_prior", "composer-2-fast");
    let app = build_router(homes.state.clone());
    let body = json!({
        "model": "composer-2",
        "input": "follow-up",
        "previous_response_id": "resp_prior",
    });

    let parsed = post_responses(app, body).await;
    assert_eq!(parsed.0, StatusCode::BAD_REQUEST);
    assert_eq!(error_code(&parsed.1), "previous_response_model_mismatch");
}

#[tokio::test]
async fn cursor_continuation_happy_path_reaches_cursor_auth_gate() {
    let homes = common::TestHomes::new();
    prime_cursor_continuation(&homes.state, "resp_prior", "conv_prior", "composer-2-fast");
    let app = build_router(homes.state.clone());
    let body = json!({
        "model": "composer-2-fast",
        "input": "follow-up",
        "previous_response_id": "resp_prior",
    });

    let parsed = post_responses(app, body).await;
    assert_eq!(parsed.0, StatusCode::UNAUTHORIZED);
    assert_eq!(error_type(&parsed.1), "missing_credential");
}

fn prime_cursor_continuation(
    state: &unified_model_proxy_v2::AppState,
    response_id: &str,
    conversation_id: &str,
    upstream_model: &str,
) {
    let stable = json!({ "model": upstream_model });
    let key = CursorContinuationKey {
        route: CursorRoute::Responses,
        provider: Provider::Cursor,
        upstream_model: upstream_model.to_string(),
        target_format: TargetFormat::CursorAgent,
        stable_request_fields: stable,
        response_id: response_id.to_string(),
        conversation_id: conversation_id.to_string(),
    };
    state.cursor_sessions.store_continuation(
        &key,
        ConversationState {
            checkpoint: None,
            pending_tool_calls: Vec::new(),
            last_access: Instant::now(),
            route: CursorRoute::Responses,
            provider: Provider::Cursor,
            upstream_model: upstream_model.to_string(),
            target_format: TargetFormat::CursorAgent,
            stable_field_hash: [0; 32],
            response_id: response_id.to_string(),
            conversation_id: conversation_id.to_string(),
            blob_store: HashMap::new(),
        },
    );
    state.remember_response_for_continuation(NewResponseStateRecord {
        route: "responses".into(),
        provider: "cursor".into(),
        upstream_model: upstream_model.to_string(),
        upstream_response_id: response_id.to_string(),
        adapter_response_id: response_id.to_string(),
        conversation_id: Some(conversation_id.to_string()),
        raw_response: json!({ "id": response_id, "output": [] }),
        raw_input_items: Value::Null,
        upstream_codex_minted: false,
    });
}

async fn post_responses(app: axum::Router, body: Value) -> (StatusCode, Value) {
    let request = Request::builder()
        .method("POST")
        .uri("/v1/responses")
        .header("host", "localhost")
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    let status = response.status();
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let parsed = serde_json::from_slice(&bytes).unwrap_or_else(|_| json!({}));
    (status, parsed)
}

fn error_code(value: &Value) -> &str {
    value
        .pointer("/error/code")
        .and_then(Value::as_str)
        .unwrap_or("")
}

fn error_type(value: &Value) -> &str {
    value
        .pointer("/error/type")
        .and_then(Value::as_str)
        .unwrap_or("")
}
