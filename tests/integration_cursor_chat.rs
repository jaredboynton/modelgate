//! End-to-end Cursor Chat Completions adapter tests.
//!
//! Mirrors the Responses suite in `integration_cursor_responses.rs` but
//! targets `/v1/chat/completions`. Lanes B/G have landed; without Cursor
//! credentials in the test environment, the route returns 401
//! missing_credential. The Lane G success branches stay aspirational and
//! flip on once a real Cursor backend is wired into the test rig.

mod common;

use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
};
use serde_json::json;
use tower::ServiceExt;
use unified_model_proxy_v2::build_router;

const COMPOSER_MODELS: &[&str] = &["composer-1.5", "composer-2", "composer-2-fast"];

fn lane_g_landed() -> bool {
    std::env::var_os("UMP_LANE_G_CURSOR_CHAT").is_some()
}

#[tokio::test]
async fn cursor_chat_route_returns_phase1_model_not_supported_for_each_composer() {
    for model in COMPOSER_MODELS {
        let homes = common::TestHomes::new();
        let app = build_router(homes.state.clone());
        let body = json!({
            "model": model,
            "messages": [
                { "role": "user", "content": "hello cursor" }
            ],
            "stream": false,
        });
        let request = Request::builder()
            .method("POST")
            .uri("/v1/chat/completions")
            .header("host", "localhost")
            .header("content-type", "application/json")
            .body(Body::from(body.to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        let status = response.status();
        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let parsed: serde_json::Value =
            serde_json::from_slice(&bytes).unwrap_or_else(|_| json!({}));

        if lane_g_landed() {
            assert!(status.is_success(), "Lane G chat: {model} should succeed");
            assert_eq!(parsed["object"], "chat.completion");
            assert!(parsed["choices"].is_array());
        } else {
            assert_eq!(
                status,
                StatusCode::UNAUTHORIZED,
                "Lanes B/G landed: chat completion for {model} returns 401 when missing credentials",
            );
            let error_type = parsed
                .pointer("/error/type")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            assert_eq!(error_type, "missing_credential");
        }
    }
}

#[tokio::test]
async fn cursor_chat_streaming_request_returns_phase1_400_until_lane_g_lands() {
    // Lane G post-condition: chunk envelope with `delta.role`,
    // `delta.content`, `delta.reasoning_content` (Composer 2-family),
    // optional `delta.tool_calls`, finish_reason, and a final usage block
    // per ralplan Section 6.
    let homes = common::TestHomes::new();
    let app = build_router(homes.state.clone());
    let body = json!({
        "model": "composer-2-fast",
        "messages": [
            { "role": "user", "content": "stream me" }
        ],
        "stream": true,
    });
    let request = Request::builder()
        .method("POST")
        .uri("/v1/chat/completions")
        .header("host", "localhost")
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    let status = response.status();

    if lane_g_landed() {
        assert!(status.is_success());
        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body = String::from_utf8_lossy(&bytes);
        assert!(
            body.contains("\"object\":\"chat.completion.chunk\""),
            "Lane G stream must emit chat.completion.chunk envelopes: {body}",
        );
    } else {
        assert_eq!(status, StatusCode::UNAUTHORIZED);
        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let parsed: serde_json::Value =
            serde_json::from_slice(&bytes).unwrap_or_else(|_| json!({}));
        let error_type = parsed
            .pointer("/error/type")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        assert_eq!(error_type, "missing_credential");
    }
}

#[tokio::test]
async fn cursor_chat_reasoning_content_pinned_for_composer_2_family() {
    // Composer 2-family models emit `delta.reasoning_content` when Lane G
    // lands. Phase 1 returns 400.
    for model in &["composer-2", "composer-2-fast"] {
        let homes = common::TestHomes::new();
        let app = build_router(homes.state.clone());
        let body = json!({
            "model": model,
            "messages": [
                { "role": "user", "content": "think first" }
            ],
            "stream": true,
        });
        let request = Request::builder()
            .method("POST")
            .uri("/v1/chat/completions")
            .header("host", "localhost")
            .header("content-type", "application/json")
            .body(Body::from(body.to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        let status = response.status();

        if lane_g_landed() {
            let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
            let body = String::from_utf8_lossy(&bytes);
            assert!(
                body.contains("reasoning_content"),
                "Lane G chat reasoning evidence required for {model}: {body}",
            );
        } else {
            assert_eq!(status, StatusCode::UNAUTHORIZED);
            let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
            let parsed: serde_json::Value =
                serde_json::from_slice(&bytes).unwrap_or_else(|_| json!({}));
            let error_type = parsed
                .pointer("/error/type")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            assert_eq!(error_type, "missing_credential");
        }
    }
}

#[tokio::test]
async fn cursor_chat_tool_calls_round_trip_pinned_for_lane_g() {
    // Lane G must emit `delta.tool_calls[*].function.arguments` in chat
    // streaming chunks and accept tool messages back in a follow-up call.
    let homes = common::TestHomes::new();
    let app = build_router(homes.state.clone());
    let body = json!({
        "model": "composer-2-fast",
        "messages": [
            { "role": "user", "content": "look up the weather" }
        ],
        "tools": [{
            "type": "function",
            "function": {
                "name": "get_weather",
                "parameters": { "type": "object", "properties": {} }
            }
        }],
        "stream": false,
    });
    let request = Request::builder()
        .method("POST")
        .uri("/v1/chat/completions")
        .header("host", "localhost")
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    let status = response.status();

    if lane_g_landed() {
        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let parsed: serde_json::Value =
            serde_json::from_slice(&bytes).unwrap_or_else(|_| json!({}));
        let choices = parsed["choices"].as_array().expect("Lane G choices array");
        let tool_calls = choices[0]
            .pointer("/message/tool_calls")
            .and_then(|v| v.as_array())
            .expect("Lane G must populate tool_calls when the model invokes a tool");
        assert!(!tool_calls.is_empty());
    } else {
        assert_eq!(status, StatusCode::UNAUTHORIZED);
        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let parsed: serde_json::Value =
            serde_json::from_slice(&bytes).unwrap_or_else(|_| json!({}));
        let error_type = parsed
            .pointer("/error/type")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        assert_eq!(error_type, "missing_credential");
    }
}
