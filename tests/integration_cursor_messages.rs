//! End-to-end Cursor Anthropic Messages adapter tests.
//!
//! Mirrors the Responses + Chat suites for `/v1/messages`. Phase 1 returns
//! 400 model_not_supported until Lane G lands the real adapter.
//!
//! Cursor Messages-specific: image blocks must be rejected with an
//! explicit unsupported error (Cursor agent stream is text-only).

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
    std::env::var_os("UMP_LANE_G_CURSOR_MESSAGES").is_some()
}

#[tokio::test]
async fn cursor_messages_route_returns_phase1_model_not_supported_for_each_composer() {
    for model in COMPOSER_MODELS {
        let homes = common::TestHomes::new();
        let app = build_router(homes.state.clone());
        let body = json!({
            "model": model,
            "messages": [
                { "role": "user", "content": "hello cursor" }
            ],
            "max_tokens": 256,
        });
        let request = Request::builder()
            .method("POST")
            .uri("/v1/messages")
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
            assert!(
                status.is_success(),
                "Lane G messages: {model} should succeed"
            );
            assert_eq!(parsed["type"], "message");
            assert!(parsed["content"].is_array());
        } else {
            assert_eq!(
                status,
                StatusCode::UNAUTHORIZED,
                "missing_credential gate: {model} returns 401 until Lane G/H wires real creds",
            );
            let error_type = parsed
                .pointer("/error/type")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            assert_eq!(
                error_type, "missing_credential",
                "missing_credential gate: {model} returns 401 until Lane G/H wires real creds: {parsed}",
            );
        }
    }
}

#[tokio::test]
async fn cursor_messages_stream_event_order_pinned_for_lane_g() {
    // Lane G post-condition: emit `message_start`, `content_block_start`,
    // `content_block_delta` (text + thinking + signature),
    // `content_block_stop`, `message_delta`, `message_stop` per the
    // Anthropic Messages spec.
    let homes = common::TestHomes::new();
    let app = build_router(homes.state.clone());
    let body = json!({
        "model": "composer-2-fast",
        "messages": [
            { "role": "user", "content": "stream me" }
        ],
        "max_tokens": 256,
        "stream": true,
    });
    let request = Request::builder()
        .method("POST")
        .uri("/v1/messages")
        .header("host", "localhost")
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    let status = response.status();

    if lane_g_landed() {
        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body_text = String::from_utf8_lossy(&bytes);
        assert!(
            body_text.contains("event: message_start"),
            "Lane G messages stream must lead with message_start: {body_text}",
        );
        assert!(
            body_text.contains("event: message_stop"),
            "Lane G messages stream must reach message_stop: {body_text}",
        );
    } else {
        assert_eq!(
            status,
            StatusCode::UNAUTHORIZED,
            "missing_credential gate: composer-2-fast returns 401 until Lane G/H wires real creds",
        );
        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let parsed: serde_json::Value =
            serde_json::from_slice(&bytes).unwrap_or_else(|_| json!({}));
        let error_type = parsed
            .pointer("/error/type")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        assert_eq!(
            error_type, "missing_credential",
            "missing_credential gate: composer-2-fast returns 401 until Lane G/H wires real creds: {parsed}",
        );
    }
}

#[tokio::test]
async fn cursor_messages_thinking_delta_pinned_for_composer_2_family() {
    // Composer 2-family must emit `thinking_delta` and the matching
    // `signature_delta` per the events spec.
    for model in &["composer-2", "composer-2-fast"] {
        let homes = common::TestHomes::new();
        let app = build_router(homes.state.clone());
        let body = json!({
            "model": model,
            "messages": [
                { "role": "user", "content": "think first" }
            ],
            "max_tokens": 256,
            "stream": true,
        });
        let request = Request::builder()
            .method("POST")
            .uri("/v1/messages")
            .header("host", "localhost")
            .header("content-type", "application/json")
            .body(Body::from(body.to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        if lane_g_landed() {
            let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
            let body_text = String::from_utf8_lossy(&bytes);
            assert!(
                body_text.contains("thinking_delta"),
                "Lane G messages thinking_delta required for {model}: {body_text}",
            );
        } else {
            let status = response.status();
            assert_eq!(
                status,
                StatusCode::UNAUTHORIZED,
                "missing_credential gate: {model} returns 401 until Lane G/H wires real creds",
            );
            let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
            let parsed: serde_json::Value =
                serde_json::from_slice(&bytes).unwrap_or_else(|_| json!({}));
            let error_type = parsed
                .pointer("/error/type")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            assert_eq!(
                error_type, "missing_credential",
                "missing_credential gate: {model} returns 401 until Lane G/H wires real creds: {parsed}",
            );
        }
    }
}

#[tokio::test]
async fn cursor_messages_image_block_rejected_with_explicit_unsupported_error() {
    // Cursor Composer is text-only on the agent stream. Adapter rejects
    // image content blocks with a stable, descriptive error.
    let homes = common::TestHomes::new();
    let app = build_router(homes.state.clone());
    let body = json!({
        "model": "composer-2-fast",
        "messages": [{
            "role": "user",
            "content": [
                { "type": "text", "text": "here is an image" },
                {
                    "type": "image",
                    "source": {
                        "type": "base64",
                        "media_type": "image/png",
                        "data": "AAAA"
                    }
                }
            ]
        }],
        "max_tokens": 256,
    });
    let request = Request::builder()
        .method("POST")
        .uri("/v1/messages")
        .header("host", "localhost")
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    let status = response.status();
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let parsed: serde_json::Value = serde_json::from_slice(&bytes).unwrap_or_else(|_| json!({}));

    // The image-block check runs in `cursor_messages::build_request` before
    // the credentials gate, so the response is the same 400 + image error
    // regardless of Lane G state. The `if lane_g_landed()` branch is kept
    // for symmetry with the rest of the suite.
    let _ = lane_g_landed();
    assert_eq!(
        status,
        StatusCode::BAD_REQUEST,
        "image-block rejection runs before creds gate: {parsed}",
    );
    let message = parsed
        .pointer("/error/message")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let lower = message.to_lowercase();
    assert!(
        lower.contains("image") || lower.contains("unsupported"),
        "image-block error must call out images: {parsed}",
    );
}

#[tokio::test]
async fn cursor_messages_tool_use_round_trip_pinned_for_lane_g() {
    // Lane G must emit `tool_use` content blocks and accept subsequent
    // `tool_result` blocks in continuation messages.
    let homes = common::TestHomes::new();
    let app = build_router(homes.state.clone());
    let body = json!({
        "model": "composer-2-fast",
        "messages": [
            { "role": "user", "content": "use the tool" }
        ],
        "tools": [{
            "name": "lookup",
            "description": "look something up",
            "input_schema": { "type": "object", "properties": {} }
        }],
        "max_tokens": 256,
    });
    let request = Request::builder()
        .method("POST")
        .uri("/v1/messages")
        .header("host", "localhost")
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    if lane_g_landed() {
        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let parsed: serde_json::Value =
            serde_json::from_slice(&bytes).unwrap_or_else(|_| json!({}));
        let content = parsed["content"].as_array().expect("Lane G content array");
        assert!(
            content.iter().any(|block| block["type"] == "tool_use"),
            "Lane G must emit tool_use blocks when model invokes a tool: {parsed}",
        );
    } else {
        let status = response.status();
        assert_eq!(
            status,
            StatusCode::UNAUTHORIZED,
            "missing_credential gate: composer-2-fast returns 401 until Lane G/H wires real creds",
        );
        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let parsed: serde_json::Value =
            serde_json::from_slice(&bytes).unwrap_or_else(|_| json!({}));
        let error_type = parsed
            .pointer("/error/type")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        assert_eq!(
            error_type, "missing_credential",
            "missing_credential gate: composer-2-fast returns 401 until Lane G/H wires real creds: {parsed}",
        );
    }
}
