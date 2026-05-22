use axum::{
    body::Bytes,
    extract::State,
    http::{header, HeaderMap, HeaderValue, StatusCode},
};
use futures::StreamExt;

use crate::{
    adapter::{anthropic_responses::anthropic_messages_to_responses, cursor_messages},
    model_alias::{resolve_model_required, ResolvedTarget},
    route::dispatch::{
        plan_for_target, plan_with_resolver, plan_with_state, DispatchAction, DispatchEdge,
        RequestFormat,
    },
    upstream,
    upstream_response::sse_headers,
    AppError, AppResult, AppState, UpstreamResponse,
};

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum MessagesRoute {
    BedrockMessages,
    CodexResponses,
    CursorAgent { upstream_model: String },
}

pub async fn messages(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> AppResult<UpstreamResponse> {
    let value = serde_json::from_slice(&body)?;
    let plan = plan_with_state(&state, RequestFormat::AnthropicMessages, &value)?;
    match plan.action {
        DispatchAction::BedrockAnthropicMessages => {
            upstream::bedrock::forward_messages_response(&state, value, headers).await
        }
        DispatchAction::CodexResponses => {
            let mut responses = anthropic_messages_to_responses(value)?;
            responses["model"] = serde_json::Value::String(plan.target.upstream_model);
            codex_response_bytes(upstream::codex::responses(&state, responses).await?)
        }
        DispatchAction::GoogleGenerateContent => {
            Err(AppError::ModelNotSupported(plan.requested_model))
        }
        DispatchAction::WindsurfChat => Err(AppError::ModelNotSupported(plan.requested_model)),
        DispatchAction::CursorAgent => {
            execute_cursor_messages(&state, &headers, &plan, value).await
        }
    }
}

async fn execute_cursor_messages(
    state: &AppState,
    headers: &HeaderMap,
    plan: &crate::route::dispatch::DispatchPlan,
    value: serde_json::Value,
) -> AppResult<UpstreamResponse> {
    let stream = value
        .get("stream")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);
    let mut request = cursor_messages::build_request(&value)?;
    request.upstream_model = plan.target.upstream_model.clone();
    let detection = crate::upstream::cursor::client_profile::detect_client_profile(headers);
    request.client_profile = detection.profile.into();
    // Check credentials first (fast path on cache hit, fails closed before expensive workspace work).
    upstream::cursor::ensure_credentials(state).await?;
    upstream::cursor::workspace::attach_to_request(&mut request, headers).await;

    if stream {
        let events = upstream::cursor::run::run(state, request).await;
        let mut ctx = cursor_messages::MessagesContext::new(&plan.requested_model);
        let stream = events
            .map(move |event| {
                let mut bytes = Vec::new();
                for frame in cursor_messages::emit_event(&event, &mut ctx) {
                    bytes.extend_from_slice(frame.to_wire().as_bytes());
                }
                Ok::<Bytes, AppError>(Bytes::from(bytes))
            })
            .filter_map(|chunk| async move {
                match chunk {
                    Ok(bytes) if bytes.is_empty() => None,
                    other => Some(other),
                }
            });
        return Ok(UpstreamResponse::stream(
            "cursor",
            StatusCode::OK,
            sse_headers(),
            stream,
        ));
    }

    let events: Vec<_> = upstream::cursor::run::run(state, request)
        .await
        .collect()
        .await;
    let response = cursor_messages::collect_non_stream(&plan.requested_model, events)?;
    UpstreamResponse::json("cursor", response).map_err(AppError::from)
}

pub async fn count_tokens(State(state): State<AppState>, body: Bytes) -> AppResult<Bytes> {
    let value: serde_json::Value = serde_json::from_slice(&body)?;
    plan_with_state(&state, RequestFormat::AnthropicMessages, &value)?;
    let rough = value.to_string().len().div_ceil(4);
    Ok(Bytes::from(
        serde_json::json!({ "input_tokens": rough }).to_string(),
    ))
}

pub fn route_for_messages_model(value: &serde_json::Value) -> AppResult<MessagesRoute> {
    route_for_messages_model_with_resolver(value, resolve_model_required)
}

pub fn route_for_messages_model_with_resolver<F>(
    value: &serde_json::Value,
    resolve: F,
) -> AppResult<MessagesRoute>
where
    F: FnMut(&str) -> AppResult<crate::model_alias::ResolvedModel>,
{
    route_from_plan(plan_with_resolver(
        value,
        RequestFormat::AnthropicMessages,
        resolve,
    )?)
}

fn route_from_plan(plan: crate::route::dispatch::DispatchPlan) -> AppResult<MessagesRoute> {
    match plan.edge {
        DispatchEdge::AnthropicMessagesToAnthropicMessagesBedrock => {
            Ok(MessagesRoute::BedrockMessages)
        }
        DispatchEdge::AnthropicMessagesToResponsesCodex => Ok(MessagesRoute::CodexResponses),
        DispatchEdge::AnthropicMessagesToCursorAgentCursor => Ok(MessagesRoute::CursorAgent {
            upstream_model: plan.target.upstream_model,
        }),
        _ => Err(AppError::ModelNotSupported(plan.requested_model)),
    }
}

pub fn messages_route_for_alias(
    model: &str,
    alias: &crate::model_alias::ResolvedModel,
) -> AppResult<MessagesRoute> {
    route_from_plan(plan_for_target(
        RequestFormat::AnthropicMessages,
        model,
        ResolvedTarget::from_resolved_model(alias.clone(), model)?,
    )?)
}

fn codex_response_bytes(body: Bytes) -> AppResult<UpstreamResponse> {
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/json"),
    );
    Ok(UpstreamResponse::bytes(
        "codex",
        StatusCode::OK,
        headers,
        body,
    ))
}
