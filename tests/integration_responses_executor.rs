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
