mod common;

use axum::{body::Bytes, http::HeaderMap};
use serde_json::json;
use unified_model_proxy_v2::route::{
    responses::responses,
    responses_executor::{execute_responses_request, ExecuteResponsesOptions},
};

#[tokio::test]
async fn http_responses_handler_stays_a_thin_executor_wrapper() {
    let homes = common::TestHomes::new();
    let body = Bytes::from(
        json!({
            "model": "claude-sonnet-4-6"
        })
        .to_string(),
    );
    let error = match responses(axum::extract::State(homes.state), HeaderMap::new(), body).await {
        Ok(_) => panic!("expected adapter validation error"),
        Err(error) => error,
    };

    assert!(error.to_string().contains("missing input"));
}

#[tokio::test]
async fn shared_executor_is_callable_for_http_backed_facade_mode() {
    let homes = common::TestHomes::new();
    let error = match execute_responses_request(
        &homes.state,
        HeaderMap::new(),
        json!({
            "model": "claude-sonnet-4-6"
        }),
        ExecuteResponsesOptions { force_stream: true },
    )
    .await
    {
        Ok(_) => panic!("expected adapter validation error"),
        Err(error) => error,
    };

    assert!(error.to_string().contains("missing input"));
}

#[tokio::test]
async fn shared_executor_rejects_opaque_compaction_before_provider_adapters() {
    let homes = common::TestHomes::new();
    let error = match execute_responses_request(
        &homes.state,
        HeaderMap::new(),
        json!({
            "model": "claude-opus-4-7",
            "input": [{
                "type": "compaction",
                "encrypted_content": "openai-native-opaque"
            }]
        }),
        ExecuteResponsesOptions {
            force_stream: false,
        },
    )
    .await
    {
        Ok(_) => panic!("opaque OpenAI compaction must fail for Bedrock targets"),
        Err(error) => error,
    };

    assert_eq!(error.status(), axum::http::StatusCode::BAD_REQUEST);
    assert_eq!(error.code(), Some("unsupported_compaction_item_for_target"));
    assert!(
        !error
            .to_string()
            .contains("unsupported Responses input item"),
        "shared executor must fail before Anthropic adapter conversion: {error}"
    );
}

// ---------------------------------------------------------------------------
// Cursor route arm coverage (Phase 1)
//
// Phase 1 lands the route enum variant + dispatch but stops short of the
// real Cursor adapter (Lane G). The executor returns
// Cursor composer models dispatch into the Cursor adapter. In isolated test
// homes without real credentials the route fails at the upstream-owned
// credential preflight, not at model resolution.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn shared_executor_returns_missing_cursor_credential_for_composer_without_auth() {
    let homes = common::TestHomes::new();
    let error = match execute_responses_request(
        &homes.state,
        HeaderMap::new(),
        json!({
            "model": "composer-2-fast",
            "input": "hello"
        }),
        ExecuteResponsesOptions {
            force_stream: false,
        },
    )
    .await
    {
        Ok(_) => panic!("Cursor route should require credentials in isolated test homes"),
        Err(error) => error,
    };

    assert_eq!(error.status(), axum::http::StatusCode::UNAUTHORIZED);
    assert_eq!(
        error.error_type(),
        "missing_credential",
        "Cursor composer models should reach the adapter path before auth preflight fails: {error}"
    );
}

#[tokio::test]
async fn http_responses_handler_returns_missing_cursor_credential_for_composer_without_auth() {
    let homes = common::TestHomes::new();
    let body = Bytes::from(
        json!({
            "model": "composer-2-fast",
            "input": "hello"
        })
        .to_string(),
    );
    let error = match responses(axum::extract::State(homes.state), HeaderMap::new(), body).await {
        Ok(_) => panic!("Cursor route should require credentials in isolated test homes"),
        Err(error) => error,
    };

    assert_eq!(error.status(), axum::http::StatusCode::UNAUTHORIZED);
    assert_eq!(error.error_type(), "missing_credential");
}
