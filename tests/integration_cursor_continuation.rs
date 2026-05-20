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
    cursor_agent::{CursorClientProfile, CursorContinuationKey, CursorRoute, CursorToolCall},
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
    assert_eq!(parsed.0, StatusCode::UNAUTHORIZED, "{:#?}", parsed.1);
    assert_eq!(error_type(&parsed.1), "missing_credential");
}

#[tokio::test]
async fn cursor_droid_replayed_tool_result_without_previous_response_id_reaches_auth_gate() {
    let homes = common::TestHomes::new();
    prime_cursor_continuation_with_pending(
        &homes.state,
        "resp_prior",
        "conv_prior",
        "composer-2-fast",
        CursorClientProfile::Droid,
        vec![CursorToolCall {
            id: "call_lookup".into(),
            name: "Grep".into(),
            arguments: json!({ "pattern": "needle" }),
        }],
    );
    let app = build_router(homes.state.clone());
    let body = json!({
        "model": "composer-2-fast",
        "store": false,
        "input": [
            { "type": "message", "role": "user", "content": "search the repo" },
            {
                "type": "function_call",
                "call_id": "call_lookup",
                "name": "Grep",
                "arguments": "{\"pattern\":\"needle\"}"
            },
            {
                "type": "function_call_output",
                "call_id": "call_lookup",
                "output": "needle found"
            }
        ],
    });

    let parsed = post_responses_with_user_agent(app, body, "factory-cli/0.129.0").await;
    assert_eq!(parsed.0, StatusCode::UNAUTHORIZED, "{:#?}", parsed.1);
    assert_eq!(error_type(&parsed.1), "missing_credential");
}

#[tokio::test]
async fn cursor_droid_replayed_multiple_read_results_for_fast_alias_reaches_auth_gate() {
    let homes = common::TestHomes::new();
    prime_cursor_continuation_with_pending_for_request_model(
        &homes.state,
        "resp_readme",
        "conv_readme",
        "composer-2.5-fast",
        "composer-2.5",
        CursorClientProfile::Droid,
        vec![
            CursorToolCall {
                id: "call_readme".into(),
                name: "Read".into(),
                arguments: json!({ "path": "README.md" }),
            },
            CursorToolCall {
                id: "call_missing".into(),
                name: "Read".into(),
                arguments: json!({ "path": "DOES_NOT_EXIST.md" }),
            },
        ],
    );
    let app = build_router(homes.state.clone());
    let body = json!({
        "model": "composer-2.5-fast",
        "store": false,
        "input": [
            { "type": "message", "role": "user", "content": "raed the readme" },
            {
                "type": "function_call",
                "call_id": "call_readme",
                "name": "Read",
                "arguments": "{\"path\":\"README.md\"}"
            },
            {
                "type": "function_call_output",
                "call_id": "call_readme",
                "output": "Unified Model Proxy v2"
            },
            {
                "type": "function_call",
                "call_id": "call_missing",
                "name": "Read",
                "arguments": "{\"path\":\"DOES_NOT_EXIST.md\"}"
            },
            {
                "type": "function_call_output",
                "call_id": "call_missing",
                "output": "file not found",
                "error": "not_found"
            }
        ],
    });

    let parsed = post_responses_with_user_agent(app, body, "factory-cli/0.129.0").await;
    assert_eq!(parsed.0, StatusCode::UNAUTHORIZED);
    assert_eq!(error_type(&parsed.1), "missing_credential");
}

#[tokio::test]
async fn cursor_droid_replayed_live_prefixed_read_result_reaches_auth_gate() {
    let homes = common::TestHomes::new();
    prime_cursor_continuation_with_pending_and_stable_fields(
        &homes.state,
        "resp_readme",
        "conv_readme",
        "composer-2.5",
        CursorClientProfile::Droid,
        json!({
            "model": "composer-2.5-fast",
            "instructions": "stable Droid instructions",
        }),
        vec![CursorToolCall {
            id: "46d82b32-d981-4f82-bb91-523a18ebeab4".into(),
            name: "Read".into(),
            arguments: json!({ "file_path": "/private/tmp/ump-droid-live-capture/README.md" }),
        }],
    );
    let app = build_router(homes.state.clone());
    let body = json!({
        "model": "composer-2.5-fast",
        "store": false,
        "stream": true,
        "instructions": "stable Droid instructions",
        "input": [
            { "role": "user", "content": "Read README.md using tools" },
            {
                "type": "function_call",
                "call_id": "call_46d82b32-d981-4f82-bb91-_0",
                "name": "Read",
                "arguments": "{\"file_path\":\"/private/tmp/ump-droid-live-capture/README.md\"}"
            },
            {
                "type": "function_call_output",
                "call_id": "call_46d82b32-d981-4f82-bb91-_0",
                "output": "alpha README\n"
            }
        ],
    });

    let parsed = post_responses_with_user_agent(app, body, "factory-cli/0.129.0").await;
    assert_eq!(parsed.0, StatusCode::UNAUTHORIZED);
    assert_eq!(error_type(&parsed.1), "missing_credential");
}

#[tokio::test]
async fn cursor_droid_replayed_sequential_read_round_uses_latest_tool_result() {
    let homes = common::TestHomes::new();
    prime_cursor_continuation_with_pending_and_stable_fields(
        &homes.state,
        "resp_second_read",
        "conv_readme",
        "composer-2.5",
        CursorClientProfile::Droid,
        json!({
            "model": "composer-2.5-fast",
            "instructions": "stable Droid instructions",
        }),
        vec![CursorToolCall {
            id: "f68a98e7-14ad-4780-8259-7ebd63e60d06".into(),
            name: "Read".into(),
            arguments: json!({ "file_path": "/private/tmp/ump-droid-two-read-live/README.md" }),
        }],
    );
    let app = build_router(homes.state.clone());
    let body = json!({
        "model": "composer-2.5-fast",
        "store": false,
        "stream": true,
        "instructions": "stable Droid instructions",
        "input": [
            { "role": "user", "content": "Read README.md using tools" },
            {
                "type": "function_call",
                "call_id": "call_9c43ea9c-bc9e-4c6a-9fae-_0",
                "name": "Read",
                "arguments": "{\"file_path\":\"/private/tmp/ump-droid-two-read-live/README.md\"}"
            },
            {
                "type": "function_call_output",
                "call_id": "call_9c43ea9c-bc9e-4c6a-9fae-_0",
                "output": "alpha README\n"
            },
            {
                "type": "function_call",
                "call_id": "call_f68a98e7-14ad-4780-8259-_1",
                "name": "Read",
                "arguments": "{\"file_path\":\"/private/tmp/ump-droid-two-read-live/README.md\"}"
            },
            {
                "type": "function_call_output",
                "call_id": "call_f68a98e7-14ad-4780-8259-_1",
                "output": "alpha README\n"
            }
        ],
    });

    let parsed = post_responses_with_user_agent(app, body, "factory-cli/0.129.0").await;
    assert_eq!(parsed.0, StatusCode::UNAUTHORIZED, "{:#?}", parsed.1);
    assert_eq!(error_type(&parsed.1), "missing_credential");
}

#[tokio::test]
async fn cursor_droid_historical_tool_result_before_new_user_message_is_not_continuation() {
    let homes = common::TestHomes::new();
    let app = build_router(homes.state.clone());
    let body = json!({
        "model": "composer-2.5-fast",
        "store": false,
        "stream": true,
        "instructions": "stable Droid instructions",
        "input": [
            { "role": "user", "content": "Read README.md using tools" },
            {
                "type": "function_call",
                "call_id": "call_46d82b32-d981-4f82-bb91-_0",
                "name": "Read",
                "arguments": "{\"file_path\":\"/private/tmp/ump-droid-live-capture/README.md\"}"
            },
            {
                "type": "function_call_output",
                "call_id": "call_46d82b32-d981-4f82-bb91-_0",
                "output": "alpha README\n"
            },
            { "role": "user", "content": "Reply exactly: followup-ok. Do not use tools." }
        ],
    });

    let parsed = post_responses_with_user_agent(app, body, "factory-cli/0.129.0").await;
    assert_eq!(parsed.0, StatusCode::UNAUTHORIZED);
    assert_eq!(error_type(&parsed.1), "missing_credential");
}

#[tokio::test]
async fn cursor_generic_replayed_tool_result_without_previous_response_id_stays_strict() {
    let homes = common::TestHomes::new();
    prime_cursor_continuation_with_pending(
        &homes.state,
        "resp_prior",
        "conv_prior",
        "composer-2-fast",
        CursorClientProfile::Droid,
        vec![CursorToolCall {
            id: "call_lookup".into(),
            name: "Grep".into(),
            arguments: json!({}),
        }],
    );
    let app = build_router(homes.state.clone());
    let body = json!({
        "model": "composer-2-fast",
        "input": [
            {
                "type": "function_call",
                "call_id": "call_lookup",
                "name": "Grep",
                "arguments": "{}"
            },
            {
                "type": "function_call_output",
                "call_id": "call_lookup",
                "output": "ok"
            }
        ],
    });

    let parsed = post_responses(app, body).await;
    assert_eq!(parsed.0, StatusCode::BAD_REQUEST);
    assert_eq!(error_code(&parsed.1), "unknown_previous_response_id");
}

#[tokio::test]
async fn cursor_droid_replayed_tool_result_without_unique_pending_call_fails_closed() {
    let homes = common::TestHomes::new();
    prime_cursor_continuation_with_pending(
        &homes.state,
        "resp_prior_a",
        "conv_prior_a",
        "composer-2-fast",
        CursorClientProfile::Droid,
        vec![CursorToolCall {
            id: "call_lookup".into(),
            name: "Grep".into(),
            arguments: json!({}),
        }],
    );
    prime_cursor_continuation_with_pending(
        &homes.state,
        "resp_prior_b",
        "conv_prior_b",
        "composer-2-fast",
        CursorClientProfile::Droid,
        vec![CursorToolCall {
            id: "call_lookup".into(),
            name: "Grep".into(),
            arguments: json!({}),
        }],
    );
    let app = build_router(homes.state.clone());
    let body = replayed_tool_result_body("call_lookup");

    let parsed = post_responses_with_user_agent(app, body, "factory-cli/0.129.0").await;
    assert_eq!(parsed.0, StatusCode::BAD_REQUEST);
    assert_eq!(error_code(&parsed.1), "unknown_previous_response_id");
}

#[tokio::test]
async fn cursor_droid_replayed_tool_result_without_matching_pending_call_fails_closed() {
    let homes = common::TestHomes::new();
    prime_cursor_continuation_with_pending(
        &homes.state,
        "resp_prior",
        "conv_prior",
        "composer-2-fast",
        CursorClientProfile::Droid,
        vec![CursorToolCall {
            id: "call_lookup".into(),
            name: "Grep".into(),
            arguments: json!({}),
        }],
    );
    let app = build_router(homes.state.clone());
    let body = replayed_tool_result_body("call_other");

    let parsed = post_responses_with_user_agent(app, body, "factory-cli/0.129.0").await;
    assert_eq!(parsed.0, StatusCode::BAD_REQUEST);
    assert_eq!(error_code(&parsed.1), "unknown_previous_response_id");
}

fn prime_cursor_continuation(
    state: &unified_model_proxy_v2::AppState,
    response_id: &str,
    conversation_id: &str,
    upstream_model: &str,
) {
    prime_cursor_continuation_with_pending(
        state,
        response_id,
        conversation_id,
        upstream_model,
        CursorClientProfile::GenericOpenAi,
        Vec::new(),
    );
}

fn prime_cursor_continuation_with_pending(
    state: &unified_model_proxy_v2::AppState,
    response_id: &str,
    conversation_id: &str,
    upstream_model: &str,
    client_profile: CursorClientProfile,
    pending_tool_calls: Vec<CursorToolCall>,
) {
    prime_cursor_continuation_with_pending_for_request_model(
        state,
        response_id,
        conversation_id,
        upstream_model,
        upstream_model,
        client_profile,
        pending_tool_calls,
    );
}

fn prime_cursor_continuation_with_pending_for_request_model(
    state: &unified_model_proxy_v2::AppState,
    response_id: &str,
    conversation_id: &str,
    requested_model: &str,
    upstream_model: &str,
    client_profile: CursorClientProfile,
    pending_tool_calls: Vec<CursorToolCall>,
) {
    let stable = json!({ "model": requested_model });
    prime_cursor_continuation_with_pending_and_stable_fields(
        state,
        response_id,
        conversation_id,
        upstream_model,
        client_profile,
        stable,
        pending_tool_calls,
    );
}

fn prime_cursor_continuation_with_pending_and_stable_fields(
    state: &unified_model_proxy_v2::AppState,
    response_id: &str,
    conversation_id: &str,
    upstream_model: &str,
    client_profile: CursorClientProfile,
    stable: Value,
    pending_tool_calls: Vec<CursorToolCall>,
) {
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
            pending_tool_calls,
            last_access: Instant::now(),
            route: CursorRoute::Responses,
            provider: Provider::Cursor,
            upstream_model: upstream_model.to_string(),
            target_format: TargetFormat::CursorAgent,
            client_profile,
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

fn replayed_tool_result_body(call_id: &str) -> Value {
    json!({
        "model": "composer-2-fast",
        "input": [
            {
                "type": "function_call",
                "call_id": call_id,
                "name": "Grep",
                "arguments": "{}"
            },
            {
                "type": "function_call_output",
                "call_id": call_id,
                "output": "ok"
            }
        ],
    })
}

async fn post_responses(app: axum::Router, body: Value) -> (StatusCode, Value) {
    post_responses_with_user_agent(app, body, "openai-rust/0.1.0").await
}

async fn post_responses_with_user_agent(
    app: axum::Router,
    body: Value,
    user_agent: &str,
) -> (StatusCode, Value) {
    let request = Request::builder()
        .method("POST")
        .uri("/v1/responses")
        .header("host", "localhost")
        .header("content-type", "application/json")
        .header("user-agent", user_agent)
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
