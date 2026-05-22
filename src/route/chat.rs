use axum::{
    body::Bytes,
    extract::State,
    http::{header, HeaderMap, HeaderValue, StatusCode},
};
use futures::{stream, StreamExt};

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
    WindsurfChat { upstream_model: String },
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
        DispatchAction::WindsurfChat => execute_windsurf_chat(&state, &headers, &plan, value).await,
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
    // Check credentials first (fast path on cache hit, fails closed before expensive workspace work).
    upstream::cursor::ensure_credentials(state).await?;
    crate::upstream::cursor::workspace::attach_to_request(&mut request, headers).await;

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

async fn execute_windsurf_chat(
    state: &AppState,
    headers: &HeaderMap,
    plan: &crate::route::dispatch::DispatchPlan,
    value: serde_json::Value,
) -> AppResult<UpstreamResponse> {
    crate::adapter::windsurf_chat::validate_request(&value)?;
    upstream::windsurf::ensure_credentials(state).await?;

    let detection = crate::upstream::cursor::client_profile::detect_client_profile(headers);
    let profile = detection.profile;
    let windsurf_profile = match profile {
        crate::upstream::cursor::client_profile::ClientProfile::CodexCli => {
            crate::adapter::windsurf_chat::WindsurfClientProfile::CodexCli
        }
        crate::upstream::cursor::client_profile::ClientProfile::ClaudeCode => {
            crate::adapter::windsurf_chat::WindsurfClientProfile::ClaudeCode
        }
        crate::upstream::cursor::client_profile::ClientProfile::Droid => {
            crate::adapter::windsurf_chat::WindsurfClientProfile::Droid
        }
        crate::upstream::cursor::client_profile::ClientProfile::Devin => {
            crate::adapter::windsurf_chat::WindsurfClientProfile::Devin
        }
        _ => crate::adapter::windsurf_chat::WindsurfClientProfile::Other,
    };

    let stream_response = crate::adapter::windsurf_chat::is_stream_request(&value);
    if crate::adapter::windsurf_chat::has_tool_context(&value) {
        let planning = crate::adapter::windsurf_chat::tool_planning_request(
            &value,
            &plan.target.upstream_model,
            windsurf_profile,
        )?;
        let content =
            upstream::windsurf::collect_chat_text(state, &planning, &plan.target.upstream_model)
                .await?;
        let requested_model = plan_request_model(&value)?;
        let tool_plan = crate::adapter::windsurf_chat::parse_tool_plan(&content)
            .unwrap_or(crate::adapter::windsurf_chat::ToolPlan::Final(content));
        return match tool_plan {
            crate::adapter::windsurf_chat::ToolPlan::Final(content) => {
                if stream_response {
                    Ok(windsurf_chat_text_stream(&requested_model, content))
                } else {
                    UpstreamResponse::json(
                        "windsurf",
                        crate::adapter::windsurf_chat::non_stream_text_response(
                            &requested_model,
                            content,
                        ),
                    )
                    .map_err(AppError::from)
                }
            }
            crate::adapter::windsurf_chat::ToolPlan::ToolCalls(mut calls) => {
                for call in &mut calls {
                    crate::adapter::windsurf_chat::map_windsurf_tool_call_to_client(
                        call,
                        windsurf_profile,
                    );
                }
                if stream_response {
                    Ok(windsurf_chat_tool_stream(&requested_model, calls))
                } else {
                    UpstreamResponse::json(
                        "windsurf",
                        crate::adapter::windsurf_chat::non_stream_tool_response(
                            &requested_model,
                            &calls,
                        ),
                    )
                    .map_err(AppError::from)
                }
            }
        };
    }

    if stream_response {
        let chunks =
            upstream::windsurf::stream_chat_text(state, &value, &plan.target.upstream_model)
                .await?;
        let id = crate::adapter::windsurf_chat::chat_completion_id();
        let created = crate::adapter::windsurf_chat::created_timestamp();
        let model = plan.requested_model.clone();
        let initial = crate::adapter::windsurf_chat::initial_stream_frame(&id, created, &model);
        let finish =
            crate::adapter::windsurf_chat::finish_stream_frame(&id, created, &model, "stop");
        let id_for_chunks = id.clone();
        let model_for_chunks = model.clone();
        let content_stream = chunks.map(move |chunk| {
            Ok::<Bytes, AppError>(match chunk {
                Ok(delta) => crate::adapter::windsurf_chat::content_stream_frame(
                    &id_for_chunks,
                    created,
                    &model_for_chunks,
                    &delta,
                ),
                Err(error) => crate::adapter::windsurf_chat::error_stream_frame(
                    &id_for_chunks,
                    created,
                    &model_for_chunks,
                    &error,
                ),
            })
        });
        let stream = stream::once(async move { Ok::<Bytes, AppError>(initial) })
            .chain(content_stream)
            .chain(stream::once(async move { Ok::<Bytes, AppError>(finish) }));
        let mut headers = HeaderMap::new();
        headers.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("text/event-stream"),
        );
        return Ok(UpstreamResponse::stream(
            "windsurf",
            StatusCode::OK,
            headers,
            stream,
        ));
    }

    let content =
        upstream::windsurf::collect_chat_text(state, &value, &plan.target.upstream_model).await?;
    UpstreamResponse::json(
        "windsurf",
        crate::adapter::windsurf_chat::non_stream_text_response(&plan.requested_model, content),
    )
    .map_err(AppError::from)
}

fn windsurf_chat_text_stream(model: &str, content: String) -> UpstreamResponse {
    let id = crate::adapter::windsurf_chat::chat_completion_id();
    let created = crate::adapter::windsurf_chat::created_timestamp();
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&crate::adapter::windsurf_chat::initial_stream_frame(
        &id, created, model,
    ));
    bytes.extend_from_slice(&crate::adapter::windsurf_chat::content_stream_frame(
        &id, created, model, &content,
    ));
    bytes.extend_from_slice(&crate::adapter::windsurf_chat::finish_stream_frame(
        &id, created, model, "stop",
    ));
    windsurf_static_sse(bytes)
}

fn windsurf_chat_tool_stream(
    model: &str,
    calls: Vec<crate::adapter::windsurf_chat::ToolCallPlan>,
) -> UpstreamResponse {
    let id = crate::adapter::windsurf_chat::chat_completion_id();
    let created = crate::adapter::windsurf_chat::created_timestamp();
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&crate::adapter::windsurf_chat::initial_stream_frame(
        &id, created, model,
    ));
    bytes.extend_from_slice(&crate::adapter::windsurf_chat::tool_calls_stream_frame(
        &id, created, model, &calls,
    ));
    bytes.extend_from_slice(&crate::adapter::windsurf_chat::finish_stream_frame(
        &id,
        created,
        model,
        "tool_calls",
    ));
    windsurf_static_sse(bytes)
}

fn windsurf_static_sse(bytes: Vec<u8>) -> UpstreamResponse {
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/event-stream"),
    );
    UpstreamResponse::stream(
        "windsurf",
        StatusCode::OK,
        headers,
        stream::once(async move { Ok::<Bytes, AppError>(Bytes::from(bytes)) }),
    )
}

fn plan_request_model(value: &serde_json::Value) -> AppResult<String> {
    value
        .get("model")
        .and_then(serde_json::Value::as_str)
        .map(ToOwned::to_owned)
        .ok_or_else(|| AppError::BadRequest("missing model".into()))
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
        DispatchEdge::ChatCompletionsToWindsurfChatWindsurf => Ok(ChatRoute::WindsurfChat {
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
