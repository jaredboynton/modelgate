use axum::{
    body::Bytes,
    extract::State,
    http::{header, HeaderMap, HeaderValue, StatusCode},
};

use crate::{
    adapter::anthropic_responses::{
        chat_completions_to_responses, responses_to_anthropic_messages,
    },
    model_alias::{resolve_model_required, ResolvedTarget},
    route::{
        dispatch::{
            plan_for_target, plan_with_resolver, plan_with_state, resolve_planned_model,
            DispatchAction, DispatchEdge, RequestFormat,
        },
        models::validate_codex_catalog_request,
    },
    upstream, AppError, AppResult, AppState, UpstreamResponse,
};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ChatRoute {
    CodexResponses,
    BedrockMessages,
}

pub async fn chat_completions(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> AppResult<UpstreamResponse> {
    let value = serde_json::from_slice(&body)?;
    let plan = plan_with_state(&state, RequestFormat::ChatCompletions, &value)?;
    match plan.action {
        DispatchAction::CodexResponses => {
            let upstream_model = plan.target.upstream_model.clone();
            let responses = chat_completions_to_responses(value)?;
            let prepared =
                upstream::codex::prepare_responses_body_with_resolver(responses, |model| {
                    resolve_planned_model(&plan, model, |model| {
                        state.resolve_model_for_format(
                            model,
                            RequestFormat::ChatCompletions.as_str(),
                        )
                    })
                })?;
            validate_codex_catalog_request(&state, &prepared, &upstream_model).await?;
            codex_response_bytes(upstream::codex::responses_prepared(&state, prepared).await?)
        }
        DispatchAction::BedrockAnthropicMessages => {
            let messages = responses_to_anthropic_messages(chat_completions_to_responses(value)?)?;
            upstream::bedrock::forward_messages_response(&state, messages, headers).await
        }
        DispatchAction::GoogleGenerateContent => {
            Err(AppError::ModelNotSupported(plan.requested_model))
        }
    }
}

pub fn route_for_chat_model(value: &serde_json::Value) -> AppResult<ChatRoute> {
    route_for_chat_model_with_resolver(value, resolve_model_required)
}

pub fn route_for_chat_model_with_resolver<F>(
    value: &serde_json::Value,
    resolve: F,
) -> AppResult<ChatRoute>
where
    F: FnMut(&str) -> AppResult<crate::model_alias::ResolvedModel>,
{
    route_from_plan(plan_with_resolver(
        value,
        RequestFormat::ChatCompletions,
        resolve,
    )?)
}

fn route_from_plan(plan: crate::route::dispatch::DispatchPlan) -> AppResult<ChatRoute> {
    match plan.edge {
        DispatchEdge::ChatCompletionsToResponsesCodex => Ok(ChatRoute::CodexResponses),
        DispatchEdge::ChatCompletionsToAnthropicMessagesBedrock => Ok(ChatRoute::BedrockMessages),
        _ => Err(AppError::ModelNotSupported(plan.requested_model)),
    }
}

pub fn chat_route_for_alias(
    model: &str,
    alias: &crate::model_alias::ResolvedModel,
) -> AppResult<ChatRoute> {
    route_from_plan(plan_for_target(
        RequestFormat::ChatCompletions,
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
