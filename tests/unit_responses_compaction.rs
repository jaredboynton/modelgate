mod common;

use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
};
use serde_json::{json, Value};
use tower::ServiceExt;
use unified_model_proxy_v2::build_router;

async fn post_responses(body: Value) -> (StatusCode, Value) {
    let homes = common::TestHomes::new();
    let app = build_router(homes.state);
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
    let json = serde_json::from_slice(&bytes).unwrap_or_else(|error| {
        panic!(
            "response was not json: {error}; status={status}; body={}",
            String::from_utf8_lossy(&bytes)
        )
    });
    (status, json)
}

fn opaque_compaction_item() -> Value {
    json!({
        "type": "compaction",
        "encrypted_content": "openai-native-opaque-compaction"
    })
}

fn ump_marked_pack_with_size(encrypted_content_bytes: usize) -> String {
    format!(
        "ump.compaction.v1.{}.nonce.ciphertext",
        "a".repeat(encrypted_content_bytes)
    )
}

fn assert_compaction_error(
    status: StatusCode,
    body: &Value,
    expected_status: StatusCode,
    expected_code: &str,
) {
    assert_eq!(status, expected_status, "{body}");
    assert_eq!(body["error"]["type"], "invalid_request", "{body}");
    assert_eq!(body["error"]["code"], expected_code, "{body}");
}

#[tokio::test]
async fn opaque_openai_compaction_fails_closed_before_anthropic_adapter() {
    let (status, body) = post_responses(json!({
        "model": "claude-opus-4-7",
        "input": [opaque_compaction_item()]
    }))
    .await;

    assert_compaction_error(
        status,
        &body,
        StatusCode::BAD_REQUEST,
        "unsupported_compaction_item_for_target",
    );
    assert!(
        !body["error"]["message"]
            .as_str()
            .unwrap()
            .contains("unsupported Responses input item"),
        "compaction guard must fail before adapter parsing: {body}"
    );
}

#[tokio::test]
async fn opaque_openai_context_compaction_fails_closed_before_google_adapter() {
    let (status, body) = post_responses(json!({
        "model": "gemini-3.1-flash-lite",
        "input": [{
            "type": "context_compaction",
            "encrypted_content": "openai-native-context-opaque"
        }]
    }))
    .await;

    assert_compaction_error(
        status,
        &body,
        StatusCode::BAD_REQUEST,
        "unsupported_compaction_item_for_target",
    );
    assert!(
        !body["error"]["message"]
            .as_str()
            .unwrap()
            .contains("unsupported Responses input item"),
        "compaction guard must fail before adapter parsing: {body}"
    );
}

#[tokio::test]
async fn too_many_compaction_carriers_fail_before_adapter_conversion() {
    let (status, body) = post_responses(json!({
        "model": "claude-opus-4-7",
        "input": [
            opaque_compaction_item(),
            {
                "type": "context_compaction",
                "encrypted_content": "another-opaque-carrier"
            }
        ]
    }))
    .await;

    assert_compaction_error(
        status,
        &body,
        StatusCode::BAD_REQUEST,
        "too_many_compaction_items",
    );
}

#[tokio::test]
async fn oversized_ump_pack_fails_with_payload_too_large() {
    let oversized = ump_marked_pack_with_size(1024 * 1024 + 1);
    let (status, body) = post_responses(json!({
        "model": "claude-opus-4-7",
        "input": [{
            "type": "compaction",
            "encrypted_content": oversized
        }]
    }))
    .await;

    assert_compaction_error(
        status,
        &body,
        StatusCode::PAYLOAD_TOO_LARGE,
        "compaction_pack_too_large",
    );
}
