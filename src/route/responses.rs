use axum::{
    body::{Body, Bytes},
    extract::{Path, State},
    http::{HeaderMap, Response},
    Json,
};

use crate::{
    route::responses_compaction,
    route::responses_executor::{execute_responses_request, ExecuteResponsesOptions},
    AppError, AppResult, AppState, UpstreamResponse,
};

pub use crate::route::responses_executor::{
    ensure_codex_model, responses_route_for_alias, route_for_responses_model,
    route_for_responses_model_with_resolver, ResponsesRoute,
};

pub async fn responses(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> AppResult<UpstreamResponse> {
    let value = serde_json::from_slice(&body)?;
    execute_responses_request(&state, headers, value, ExecuteResponsesOptions::default()).await
}

pub async fn compact_responses(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> AppResult<Response<Body>> {
    let value = serde_json::from_slice(&body)?;
    responses_compaction::compact_responses(&state, headers, value).await
}

pub async fn retrieve_response(
    State(state): State<AppState>,
    Path(response_id): Path<String>,
) -> AppResult<Json<serde_json::Value>> {
    state
        .public_response(&response_id)
        .map(Json)
        .ok_or_else(|| AppError::NotFound(format!("response not found: {response_id}")))
}

pub async fn response_input_items(
    State(state): State<AppState>,
    Path(response_id): Path<String>,
) -> AppResult<Json<serde_json::Value>> {
    state
        .public_input_items(&response_id)
        .map(Json)
        .ok_or_else(|| AppError::NotFound(format!("response input items not found: {response_id}")))
}
