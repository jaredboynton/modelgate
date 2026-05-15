use axum::{
    body::Bytes,
    extract::State,
    http::{header, HeaderMap, HeaderValue, StatusCode},
};
use futures::StreamExt;

use crate::{
    adapter::{
        anthropic_responses::{chat_completions_to_responses, responses_to_anthropic_messages},
        cursor_chat,
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

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ChatRoute {
    CodexResponses,
    BedrockMessages,
    CursorAgent { upstream_model: String },
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
        DispatchAction::CursorAgent => execute_cursor_chat(&state, &headers, &plan, value).await,
    }
}

async fn execute_cursor_chat(
    state: &AppState,
    headers: &HeaderMap,
    plan: &crate::route::dispatch::DispatchPlan,
    value: serde_json::Value,
) -> AppResult<UpstreamResponse> {
    let stream = value
        .get("stream")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);
    let mut request = cursor_chat::build_request(&value)?;
    request.upstream_model = plan.target.upstream_model.clone();
    let detection = crate::upstream::cursor::client_profile::detect_client_profile(headers);
    request.client_profile = detection.profile.into();
    crate::upstream::cursor::workspace::attach_to_request(&mut request, headers).await;

    upstream::cursor::ensure_credentials(state).await?;

    if stream {
        let events = upstream::cursor::run::run(state, request).await;
        let mut ctx = cursor_chat::ChatContext::new(&plan.requested_model);
        let stream = events
            .map(move |event| {
                let done = matches!(event, crate::cursor_agent::CursorAgentEvent::Done { .. });
                let mut bytes = Vec::new();
                for chunk in cursor_chat::emit_event(&event, &mut ctx) {
                    bytes.extend_from_slice(format!("data: {chunk}\n\n").as_bytes());
                }
                if done {
                    bytes.extend_from_slice(b"data: [DONE]\n\n");
                }
                Ok::<Bytes, AppError>(Bytes::from(bytes))
            })
            .filter_map(|chunk| async move {
                match chunk {
                    Ok(bytes) if bytes.is_empty() => None,
                    other => Some(other),
                }
            });
        let mut headers = HeaderMap::new();
        headers.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("text/event-stream"),
        );
        return Ok(UpstreamResponse::stream(
            "cursor",
            StatusCode::OK,
            headers,
            stream,
        ));
    }

    let events: Vec<_> = upstream::cursor::run::run(state, request)
        .await
        .collect()
        .await;
    let response = cursor_chat::collect_non_stream(&plan.requested_model, events)?;
    UpstreamResponse::json("cursor", response).map_err(AppError::from)
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
        DispatchEdge::ChatCompletionsToCursorAgentCursor => Ok(ChatRoute::CursorAgent {
            upstream_model: plan.target.upstream_model,
        }),
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
