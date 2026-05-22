use axum::{
    body::{to_bytes, Bytes},
    http::{header, HeaderMap, HeaderValue, StatusCode},
};
use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
};

use futures::{stream, StreamExt};
use serde_json::{json, Value};

use crate::{
    adapter::{
        anthropic_responses::{
            anthropic_message_to_responses_json_with_context,
            responses_to_anthropic_messages_with_context, AnthropicSseStreamTranslator,
            ToolContext,
        },
        cursor_responses,
        google_responses::{
            google_generate_content_to_responses_with_context, is_google_responses_stream_request,
            responses_to_google_generate_content_with_context, GoogleResponsesSseTranslator,
        },
    },
    compaction::{
        find_compaction_carriers, pack_context_from_headers, prepare_responses_input_for_target,
        validate_compaction_carriers, CompactionHttpError, CompactionLimits,
        RemoteCompactionPolicy,
    },
    cursor_agent::{CursorClientProfile, CursorContinuationKey, CursorRoute, CursorToolCall},
    model_alias::{resolve_model_required, Provider, ResolvedTarget, TargetFormat},
    route::{
        dispatch::{
            plan_for_target, plan_with_resolver, plan_with_state, resolve_planned_model,
            DispatchAction, DispatchEdge, RequestFormat,
        },
        models::validate_codex_catalog_request,
        responses_compaction::{
            is_v2_context_compaction_trigger, proxy_visible_context_compaction_item,
        },
    },
    state::{NewResponseStateRecord, ResponseStateRecord},
    upstream::{
        self,
        cursor::session::{PendingToolContinuationLookup, PendingToolContinuationQuery},
    },
    AppError, AppResult, AppState, UpstreamResponse,
};

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ResponsesRoute {
    CodexResponses,
    BedrockMessages,
    GoogleGenerateContent { upstream_model: String },
    CursorAgent { upstream_model: String },
    WindsurfChat { upstream_model: String },
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq)]
pub struct ExecuteResponsesOptions {
    pub force_stream: bool,
}

pub async fn execute_responses_request(
    state: &AppState,
    headers: HeaderMap,
    mut value: serde_json::Value,
    options: ExecuteResponsesOptions,
) -> AppResult<UpstreamResponse> {
    let plan = plan_with_state(state, RequestFormat::Responses, &value)?;
    if let Some(response) = enforce_compaction_policy(&plan, &headers, &mut value)? {
        return Ok(response);
    }
    match plan.action {
        DispatchAction::CodexResponses => {
            let upstream_model = plan.target.upstream_model.clone();
            let prepared = upstream::codex::prepare_responses_body_with_resolver(value, |model| {
                resolve_planned_model(&plan, model, |model| {
                    state.resolve_model_for_format(model, RequestFormat::Responses.as_str())
                })
            })?;
            validate_codex_catalog_request(state, &prepared, &upstream_model).await?;
            codex_response_stream(
                upstream::codex::responses_prepared_stream(state, prepared).await?,
            )
        }
        DispatchAction::BedrockAnthropicMessages => {
            if options.force_stream {
                value["stream"] = serde_json::Value::Bool(true);
            }
            let (body, tool_context) = responses_to_anthropic_messages_with_context(value)?;
            let upstream_response =
                upstream::bedrock::forward_messages_response(state, body, headers).await?;
            anthropic_messages_response_to_responses(
                upstream_response,
                &plan.requested_model,
                tool_context,
            )
            .await
        }
        DispatchAction::GoogleGenerateContent => {
            if options.force_stream {
                value["stream"] = serde_json::Value::Bool(true);
            }
            let upstream_model = plan.target.upstream_model.clone();
            let stream = is_google_responses_stream_request(&value);
            let (body, tool_context) =
                responses_to_google_generate_content_with_context(value, &upstream_model)?;
            let body = Bytes::from(serde_json::to_vec(&body)?);
            let upstream_response = if stream {
                upstream::google::forward_stream_generate_content_direct_response(
                    state,
                    &upstream_model,
                    headers,
                    body,
                )
                .await?
            } else {
                upstream::google::forward_generate_content_direct_response(
                    state,
                    &upstream_model,
                    headers,
                    body,
                )
                .await?
            };
            google_generate_content_response_to_responses(
                upstream_response,
                &plan.requested_model,
                tool_context,
            )
            .await
        }
        DispatchAction::CursorAgent => {
            execute_cursor_responses(state, &headers, &plan, value).await
        }
        DispatchAction::WindsurfChat => {
            execute_windsurf_responses(state, &headers, &plan, value).await
        }
    }
}

async fn execute_cursor_responses(
    state: &AppState,
    headers: &HeaderMap,
    plan: &crate::route::dispatch::DispatchPlan,
    value: serde_json::Value,
) -> AppResult<UpstreamResponse> {
    let stream = value
        .get("stream")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);
    let store_public = value
        .get("store")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(true);
    let raw_input_items = value
        .get("input")
        .cloned()
        .unwrap_or(serde_json::Value::Null);

    let mut request = cursor_responses::build_request(&value)?;
    request.upstream_model = plan.target.upstream_model.clone();
    let detection = crate::upstream::cursor::client_profile::detect_client_profile(headers);
    request.client_profile = detection.profile.into();
    request.continuation_key = cursor_continuation_key_for_request(state, plan, &value)?;
    if request.continuation_key.is_none() {
        request.continuation_key = infer_cursor_tool_result_continuation(
            state,
            plan,
            &value,
            request.client_profile,
            &request.tool_results,
        )?;
    }
    // Check credentials first (fast path on cache hit, fails closed before expensive workspace work).
    crate::upstream::cursor::ensure_credentials(state).await?;
    crate::upstream::cursor::workspace::attach_to_request(&mut request, headers).await;
    validate_cursor_tool_results(
        state,
        request.continuation_key.as_ref(),
        &request.tool_results,
    )?;
    let client_profile = request.client_profile;

    if stream {
        let events = upstream::cursor::run::run(state, request).await;
        let state_for_stream = state.clone();
        let plan_for_stream = plan.clone();
        let value_for_stream = value.clone();
        let raw_input_items_for_stream = raw_input_items.clone();
        let client_profile_for_stream = client_profile;
        let stream_response_id = format!("resp_{}", uuid::Uuid::new_v4().simple());
        let mut ctx = crate::adapter::cursor_events::ResponseContext::new(
            &plan.requested_model,
            stream_response_id.clone(),
        );
        let mut finalized = false;
        let stream = events
            .map(move |mut event| {
                if let crate::cursor_agent::CursorAgentEvent::Done { response_id, .. } = &mut event
                {
                    *response_id = stream_response_id.clone();
                }
                let mut bytes = Vec::new();
                for frame in cursor_responses::emit_event(&event, &mut ctx) {
                    bytes.extend_from_slice(frame.to_wire().as_bytes());
                }
                if !finalized {
                    if let Some(response) = cursor_response_from_context_if_done(&ctx) {
                        store_cursor_response_state(
                            &state_for_stream,
                            &plan_for_stream,
                            &value_for_stream,
                            response,
                            raw_input_items_for_stream.clone(),
                            store_public,
                            client_profile_for_stream,
                        );
                        finalized = true;
                    }
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
    let done = cursor_done_ids(&events);

    let mut response = cursor_responses::collect_non_stream(events)?;
    response["model"] = serde_json::Value::String(plan.requested_model.clone());
    let conversation_id = done
        .as_ref()
        .map(|(_, conversation_id)| conversation_id.clone());
    if let Some(conversation_id) = conversation_id.as_ref() {
        response["conversation_id"] = Value::String(conversation_id.clone());
    }
    store_cursor_response_state(
        state,
        plan,
        &value,
        response.clone(),
        raw_input_items,
        store_public,
        client_profile,
    );
    UpstreamResponse::json("cursor", response).map_err(AppError::from)
}

async fn execute_windsurf_responses(
    state: &AppState,
    headers: &HeaderMap,
    plan: &crate::route::dispatch::DispatchPlan,
    value: Value,
) -> AppResult<UpstreamResponse> {
    upstream::windsurf::ensure_credentials(state).await?;
    let store_public = value.get("store").and_then(Value::as_bool).unwrap_or(true);
    let prior = windsurf_prior_response_for_request(state, plan, &value)?;
    validate_windsurf_tool_results(prior.as_ref(), &value)?;
    let adapter_prior =
        prior.as_ref().map(
            |record| crate::adapter::windsurf_responses::PriorWindsurfResponse {
                raw_response: record.raw_response.clone(),
                raw_input_items: record.raw_input_items.clone(),
            },
        );
    let (chat_request, raw_input_items) = crate::adapter::windsurf_responses::build_chat_request(
        &value,
        &plan.target.upstream_model,
        adapter_prior.as_ref(),
    )?;

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

    if crate::adapter::windsurf_chat::has_tool_context(&chat_request) {
        let planning = crate::adapter::windsurf_chat::tool_planning_request(
            &chat_request,
            &plan.target.upstream_model,
            windsurf_profile,
        )?;
        let content =
            upstream::windsurf::collect_chat_text(state, &planning, &plan.target.upstream_model)
                .await?;
        let tool_plan = crate::adapter::windsurf_chat::parse_tool_plan(&content)
            .unwrap_or(crate::adapter::windsurf_chat::ToolPlan::Final(content));
        let response = match tool_plan {
            crate::adapter::windsurf_chat::ToolPlan::Final(content) => {
                crate::adapter::windsurf_responses::response_from_text(
                    &plan.requested_model,
                    content,
                )
            }
            crate::adapter::windsurf_chat::ToolPlan::ToolCalls(mut calls) => {
                for call in &mut calls {
                    crate::adapter::windsurf_chat::map_windsurf_tool_call_to_client(
                        call,
                        windsurf_profile,
                    );
                }
                crate::adapter::windsurf_responses::response_from_tool_calls(
                    &plan.requested_model,
                    &calls,
                )
            }
        };
        store_windsurf_response_state(state, plan, response.clone(), raw_input_items, store_public);
        if chat_request
            .get("stream")
            .and_then(Value::as_bool)
            .unwrap_or(false)
        {
            return Ok(windsurf_static_responses_sse(response));
        }
        return UpstreamResponse::json("windsurf", response).map_err(AppError::from);
    }

    if chat_request
        .get("stream")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        let response_id = crate::adapter::windsurf_responses::response_id();
        let chunks =
            upstream::windsurf::stream_chat_text(state, &chat_request, &plan.target.upstream_model)
                .await?;
        let buffer = Arc::new(Mutex::new(String::new()));
        let buffer_for_chunks = Arc::clone(&buffer);
        let content_stream = chunks.map(move |chunk| {
            Ok::<Bytes, AppError>(match chunk {
                Ok(delta) => {
                    buffer_for_chunks
                        .lock()
                        .expect("Windsurf stream buffer mutex poisoned")
                        .push_str(&delta);
                    crate::adapter::windsurf_responses::text_delta_frame(&delta)
                }
                Err(error) => crate::adapter::windsurf_responses::error_stream_frame(&error),
            })
        });
        let start = crate::adapter::windsurf_responses::text_stream_start(
            &response_id,
            &plan.requested_model,
        );
        let state_for_finish = state.clone();
        let plan_for_finish = plan.clone();
        let requested_model = plan.requested_model.clone();
        let raw_input_items_for_finish = raw_input_items.clone();
        let finish = stream::once(async move {
            let content = buffer
                .lock()
                .expect("Windsurf stream buffer mutex poisoned")
                .clone();
            let response = crate::adapter::windsurf_responses::response_from_text_with_id(
                &response_id,
                &requested_model,
                content,
            );
            store_windsurf_response_state(
                &state_for_finish,
                &plan_for_finish,
                response.clone(),
                raw_input_items_for_finish,
                store_public,
            );
            Ok::<Bytes, AppError>(crate::adapter::windsurf_responses::text_stream_finish(
                &response,
            ))
        });
        let stream = stream::once(async move { Ok::<Bytes, AppError>(start) })
            .chain(content_stream)
            .chain(finish);
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
        upstream::windsurf::collect_chat_text(state, &chat_request, &plan.target.upstream_model)
            .await?;
    let response =
        crate::adapter::windsurf_responses::response_from_text(&plan.requested_model, content);
    store_windsurf_response_state(state, plan, response.clone(), raw_input_items, store_public);
    UpstreamResponse::json("windsurf", response).map_err(AppError::from)
}

fn windsurf_static_responses_sse(response: Value) -> UpstreamResponse {
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/event-stream"),
    );
    UpstreamResponse::stream(
        "windsurf",
        StatusCode::OK,
        headers,
        stream::once(async move {
            Ok::<Bytes, AppError>(crate::adapter::windsurf_responses::static_response_sse(
                &response,
            ))
        }),
    )
}

fn windsurf_prior_response_for_request(
    state: &AppState,
    plan: &crate::route::dispatch::DispatchPlan,
    value: &Value,
) -> AppResult<Option<ResponseStateRecord>> {
    let Some(previous) = value.get("previous_response_id") else {
        return Ok(None);
    };
    if previous.is_null() {
        return Ok(None);
    }
    let previous_response_id = previous.as_str().ok_or_else(|| AppError::BadRequestCode {
        code: "previous_response_field_mismatch",
        message: "previous_response_id must be a string".into(),
    })?;
    let prior = state
        .continuation_response(previous_response_id)
        .ok_or_else(|| AppError::BadRequestCode {
            code: "unknown_previous_response_id",
            message: "unknown previous_response_id".into(),
        })?;
    validate_windsurf_prior_response(plan, &prior)?;
    Ok(Some(prior))
}

fn validate_windsurf_prior_response(
    plan: &crate::route::dispatch::DispatchPlan,
    prior: &ResponseStateRecord,
) -> AppResult<()> {
    if prior.provider != "windsurf" {
        return Err(AppError::BadRequestCode {
            code: "previous_response_target_format_mismatch",
            message: format!(
                "previous_response_id belongs to {}, not windsurf",
                prior.provider
            ),
        });
    }
    if prior.route != "responses" {
        return Err(AppError::BadRequestCode {
            code: "previous_response_route_mismatch",
            message: format!(
                "previous_response_id belongs to {}, not responses",
                prior.route
            ),
        });
    }
    if prior.upstream_model != plan.target.upstream_model {
        return Err(AppError::BadRequestCode {
            code: "previous_response_model_mismatch",
            message: format!(
                "previous_response_id belongs to {}, not {}",
                prior.upstream_model, plan.target.upstream_model
            ),
        });
    }
    Ok(())
}

fn validate_windsurf_tool_results(
    prior: Option<&ResponseStateRecord>,
    value: &Value,
) -> AppResult<()> {
    let tool_results = crate::adapter::windsurf_responses::tool_result_call_ids(value)?;
    if tool_results.is_empty() {
        return Ok(());
    }
    let Some(prior) = prior else {
        return Err(AppError::BadRequestCode {
            code: "unknown_previous_response_id",
            message: "tool result requires previous_response_id".into(),
        });
    };
    let pending =
        crate::adapter::windsurf_responses::response_function_call_ids(&prior.raw_response);
    for call_id in tool_results {
        if !pending.contains(&call_id) {
            return Err(AppError::BadRequestCode {
                code: "previous_response_field_mismatch",
                message: format!("tool result references unknown call_id {call_id}"),
            });
        }
    }
    Ok(())
}

fn store_windsurf_response_state(
    state: &AppState,
    plan: &crate::route::dispatch::DispatchPlan,
    response: Value,
    raw_input_items: Value,
    store_public: bool,
) {
    let response_id = response["id"].as_str().unwrap_or_default().to_string();
    let record = NewResponseStateRecord {
        route: "responses".into(),
        provider: "windsurf".into(),
        upstream_model: plan.target.upstream_model.clone(),
        upstream_response_id: response_id.clone(),
        adapter_response_id: response_id,
        conversation_id: None,
        raw_response: response,
        raw_input_items,
        upstream_codex_minted: false,
    };
    if store_public {
        state.store_public_response(record);
    } else {
        state.remember_response_for_continuation(record);
    }
}

fn cursor_response_from_context_if_done(
    ctx: &crate::adapter::cursor_events::ResponseContext,
) -> Option<Value> {
    if !ctx.completed {
        return None;
    }
    let status = ctx.response_status();
    let mut response = json!({
        "id": ctx.response_id.clone(),
        "object": "response",
        "created_at": 0,
        "model": ctx.model.clone(),
        "status": status,
        "output": ctx.completed_items().to_vec(),
        "usage": ctx.usage_envelope(),
    });
    if let Some(conversation_id) = ctx.conversation_id.as_ref() {
        response["conversation_id"] = Value::String(conversation_id.clone());
    }
    if status == "incomplete" {
        response["incomplete_details"] = json!({ "reason": "max_output_tokens" });
    }
    Some(response)
}

fn store_cursor_response_state(
    state: &AppState,
    plan: &crate::route::dispatch::DispatchPlan,
    value: &Value,
    response: Value,
    raw_input_items: Value,
    store_public: bool,
    client_profile: CursorClientProfile,
) {
    let response_id = response["id"].as_str().unwrap_or_default().to_string();
    let conversation_id = response
        .get("conversation_id")
        .and_then(Value::as_str)
        .map(ToOwned::to_owned);
    if let Some(conversation_id) = conversation_id.as_ref() {
        store_cursor_continuation(
            state,
            plan,
            value,
            &response,
            &response_id,
            conversation_id,
            client_profile,
        );
    }
    let record = NewResponseStateRecord {
        route: "responses".into(),
        provider: "cursor".into(),
        upstream_model: plan.target.upstream_model.clone(),
        upstream_response_id: response_id.clone(),
        adapter_response_id: response_id,
        conversation_id,
        raw_response: response.clone(),
        raw_input_items,
        upstream_codex_minted: false,
    };
    if store_public {
        state.store_public_response(record);
    } else {
        state.remember_response_for_continuation(record);
    }
}

fn store_cursor_continuation(
    state: &AppState,
    plan: &crate::route::dispatch::DispatchPlan,
    value: &Value,
    response: &Value,
    response_id: &str,
    conversation_id: &str,
    client_profile: CursorClientProfile,
) {
    let key = cursor_continuation_key(
        CursorRoute::Responses,
        plan,
        value,
        response_id,
        conversation_id,
    );
    state.cursor_sessions.store_continuation(
        &key,
        crate::upstream::cursor::session::ConversationState {
            checkpoint: None,
            pending_tool_calls: cursor_pending_tool_calls(&response["output"]),
            last_access: std::time::Instant::now(),
            route: key.route,
            provider: key.provider,
            upstream_model: key.upstream_model.clone(),
            target_format: key.target_format,
            client_profile,
            stable_field_hash: [0u8; 32],
            response_id: response_id.to_string(),
            conversation_id: conversation_id.to_string(),
            blob_store: HashMap::new(),
        },
    );
}

fn cursor_continuation_key_for_request(
    state: &AppState,
    plan: &crate::route::dispatch::DispatchPlan,
    value: &Value,
) -> AppResult<Option<CursorContinuationKey>> {
    let Some(previous) = value.get("previous_response_id") else {
        return Ok(None);
    };
    if previous.is_null() {
        return Ok(None);
    }
    let previous_response_id = previous.as_str().ok_or_else(|| AppError::BadRequestCode {
        code: "previous_response_field_mismatch",
        message: "previous_response_id must be a string".into(),
    })?;
    let prior = state
        .continuation_response(previous_response_id)
        .ok_or_else(|| AppError::BadRequestCode {
            code: "unknown_previous_response_id",
            message: "unknown previous_response_id".into(),
        })?;
    validate_cursor_prior_response(plan, &prior)?;
    let conversation_id =
        prior
            .conversation_id
            .as_deref()
            .ok_or_else(|| AppError::BadRequestCode {
                code: "unknown_previous_response_id",
                message: "previous_response_id has no Cursor conversation state".into(),
            })?;
    let key = cursor_continuation_key(
        CursorRoute::Responses,
        plan,
        value,
        previous_response_id,
        conversation_id,
    );
    if state.cursor_sessions.lookup_continuation(&key).is_none() {
        return Err(AppError::BadRequestCode {
            code: "unknown_previous_response_id",
            message: "unknown Cursor continuation state".into(),
        });
    }
    Ok(Some(key))
}

fn validate_cursor_prior_response(
    plan: &crate::route::dispatch::DispatchPlan,
    prior: &ResponseStateRecord,
) -> AppResult<()> {
    if prior.provider != "cursor" {
        return Err(AppError::BadRequestCode {
            code: "previous_response_target_format_mismatch",
            message: format!(
                "previous_response_id belongs to {}, not cursor",
                prior.provider
            ),
        });
    }
    if prior.route != "responses" {
        return Err(AppError::BadRequestCode {
            code: "previous_response_route_mismatch",
            message: format!(
                "previous_response_id belongs to {}, not responses",
                prior.route
            ),
        });
    }
    if prior.upstream_model != plan.target.upstream_model {
        return Err(AppError::BadRequestCode {
            code: "previous_response_model_mismatch",
            message: format!(
                "previous_response_id belongs to {}, not {}",
                prior.upstream_model, plan.target.upstream_model
            ),
        });
    }
    Ok(())
}

fn infer_cursor_tool_result_continuation(
    state: &AppState,
    plan: &crate::route::dispatch::DispatchPlan,
    value: &Value,
    client_profile: CursorClientProfile,
    tool_results: &[crate::cursor_agent::CursorToolResult],
) -> AppResult<Option<CursorContinuationKey>> {
    if tool_results.is_empty() {
        return Ok(None);
    }
    if value.get("previous_response_id").is_some() {
        return Err(tool_result_requires_previous_response_id());
    }
    if client_profile != CursorClientProfile::Droid {
        return Err(tool_result_requires_previous_response_id());
    }

    let mut seen = HashSet::new();
    let mut call_ids = Vec::with_capacity(tool_results.len());
    for result in tool_results {
        if !seen.insert(result.call_id.as_str()) {
            return Err(AppError::BadRequestCode {
                code: "previous_response_field_mismatch",
                message: format!("duplicate tool result call_id {}", result.call_id),
            });
        }
        call_ids.push(result.call_id.clone());
    }

    let stable_request_fields = cursor_stable_fields(value);
    match state
        .cursor_sessions
        .find_pending_tool_continuation(PendingToolContinuationQuery {
            route: CursorRoute::Responses,
            provider: Provider::Cursor,
            upstream_model: &plan.target.upstream_model,
            target_format: TargetFormat::CursorAgent,
            client_profile,
            stable_request_fields: &stable_request_fields,
            call_ids: &call_ids,
        }) {
        PendingToolContinuationLookup::Found(key) => Ok(Some(key)),
        PendingToolContinuationLookup::NotFound => Err(AppError::BadRequestCode {
            code: "unknown_previous_response_id",
            message: "tool result requires previous_response_id or an active Droid Cursor tool continuation".into(),
        }),
        PendingToolContinuationLookup::Ambiguous => Err(AppError::BadRequestCode {
            code: "unknown_previous_response_id",
            message: "ambiguous Droid Cursor tool continuation".into(),
        }),
    }
}

fn tool_result_requires_previous_response_id() -> AppError {
    AppError::BadRequestCode {
        code: "unknown_previous_response_id",
        message: "tool result requires previous_response_id".into(),
    }
}

fn validate_cursor_tool_results(
    state: &AppState,
    key: Option<&CursorContinuationKey>,
    tool_results: &[crate::cursor_agent::CursorToolResult],
) -> AppResult<()> {
    if tool_results.is_empty() {
        return Ok(());
    }
    let Some(key) = key else {
        return Err(AppError::BadRequestCode {
            code: "unknown_previous_response_id",
            message: "tool result requires previous_response_id".into(),
        });
    };
    for result in tool_results {
        if state
            .cursor_sessions
            .consume_pending_tool_call(key, &result.call_id)
            .is_none()
        {
            return Err(AppError::BadRequestCode {
                code: "previous_response_field_mismatch",
                message: format!("tool result references unknown call_id {}", result.call_id),
            });
        }
    }
    Ok(())
}

fn cursor_continuation_key(
    route: CursorRoute,
    plan: &crate::route::dispatch::DispatchPlan,
    value: &Value,
    response_id: &str,
    conversation_id: &str,
) -> CursorContinuationKey {
    CursorContinuationKey {
        route,
        provider: Provider::Cursor,
        upstream_model: plan.target.upstream_model.clone(),
        target_format: TargetFormat::CursorAgent,
        stable_request_fields: cursor_stable_fields(value),
        response_id: response_id.to_string(),
        conversation_id: conversation_id.to_string(),
    }
}

fn cursor_stable_fields(value: &Value) -> Value {
    let mut object = value.as_object().cloned().unwrap_or_default();
    for key in [
        "input",
        "stream",
        "previous_response_id",
        "store",
        "metadata",
        "user",
        "prompt_cache_key",
        "prompt_cache_retention",
        "tools",
        "tool_choice",
        "max_tool_calls",
        "parallel_tool_calls",
    ] {
        object.remove(key);
    }
    let mut entries: Vec<_> = object.into_iter().collect();
    entries.sort_by(|a, b| a.0.cmp(&b.0));
    let mut sorted = serde_json::Map::new();
    for (key, value) in entries {
        sorted.insert(key, value);
    }
    Value::Object(sorted)
}

fn cursor_done_ids(events: &[crate::cursor_agent::CursorAgentEvent]) -> Option<(String, String)> {
    events.iter().rev().find_map(|event| match event {
        crate::cursor_agent::CursorAgentEvent::Done {
            response_id,
            conversation_id,
            ..
        } => Some((response_id.clone(), conversation_id.clone())),
        _ => None,
    })
}

fn cursor_pending_tool_calls(output: &Value) -> Vec<CursorToolCall> {
    output
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|item| {
            let item_type = item.get("type")?.as_str()?;
            if item_type != "function_call" && item_type != "custom_tool_call" {
                return None;
            }
            let call_id = item.get("call_id")?.as_str()?.to_string();
            let name = item.get("name")?.as_str()?.to_string();
            let raw_args = item
                .get("arguments")
                .or_else(|| item.get("input"))
                .and_then(Value::as_str)
                .unwrap_or("{}");
            let arguments = serde_json::from_str(raw_args).unwrap_or_else(|_| json!(raw_args));
            Some(CursorToolCall {
                id: call_id,
                name,
                arguments,
            })
        })
        .collect()
}

fn enforce_compaction_policy(
    plan: &crate::route::dispatch::DispatchPlan,
    headers: &HeaderMap,
    value: &mut serde_json::Value,
) -> AppResult<Option<UpstreamResponse>> {
    let carriers = value
        .get("input")
        .map(find_compaction_carriers)
        .unwrap_or_default();
    CompactionLimits::default().check_carrier_count(carriers.len())?;
    let is_v2_trigger = is_v2_context_compaction_trigger(value)?;
    if !is_v2_trigger {
        if let Some(input) = value.get("input") {
            validate_compaction_carriers(input, &plan.target, CompactionLimits::default())?;
        }
        let pack_context = if carriers.iter().any(|carrier| carrier.is_ump_pack) {
            Some(pack_context_from_headers(
                headers,
                "POST /v1/responses",
                &plan.target,
            )?)
        } else {
            None
        };
        prepare_responses_input_for_target(
            value,
            &plan.target,
            CompactionLimits::default(),
            pack_context.as_ref(),
        )?;
        return Ok(None);
    }
    if plan.action == DispatchAction::CodexResponses {
        return Ok(None);
    }

    match plan.remote_compaction_policy {
        RemoteCompactionPolicy::Off => Err(CompactionHttpError::new(
            StatusCode::CONFLICT,
            "compaction_disabled_for_target",
            "invalid_request",
            "remote compaction is disabled for target",
        )
        .into()),
        RemoteCompactionPolicy::Local => Err(CompactionHttpError::new(
            StatusCode::SERVICE_UNAVAILABLE,
            "proxy_compaction_unavailable",
            "server_error",
            "remote compaction unavailable for target; use local compaction",
        )
        .into()),
        RemoteCompactionPolicy::ProxyVisibleSummary => {
            let context = pack_context_from_headers(headers, "POST /v1/responses", &plan.target)?;
            let item = proxy_visible_context_compaction_item(value, &context)?;
            UpstreamResponse::json(
                "ump",
                serde_json::json!({
                    "id": format!("resp_compact_{}", uuid::Uuid::new_v4().simple()),
                    "object": "response",
                    "status": "completed",
                    "model": plan.requested_model,
                    "output": [item],
                    "usage": {
                        "input_tokens": 0,
                        "input_tokens_details": null,
                        "output_tokens": 0,
                        "output_tokens_details": null,
                        "total_tokens": 0
                    }
                }),
            )
            .map(Some)
            .map_err(AppError::from)
        }
        RemoteCompactionPolicy::Native => {
            Err(CompactionHttpError::unsupported_item_for_target(&plan.target).into())
        }
    }
}

pub fn route_for_responses_model(value: &serde_json::Value) -> AppResult<ResponsesRoute> {
    route_for_responses_model_with_resolver(value, resolve_model_required)
}

pub fn route_for_responses_model_with_resolver<F>(
    value: &serde_json::Value,
    resolve: F,
) -> AppResult<ResponsesRoute>
where
    F: FnMut(&str) -> AppResult<crate::model_alias::ResolvedModel>,
{
    route_from_plan(plan_with_resolver(
        value,
        RequestFormat::Responses,
        resolve,
    )?)
}

pub fn responses_route_for_alias(
    model: &str,
    alias: &crate::model_alias::ResolvedModel,
) -> AppResult<ResponsesRoute> {
    route_from_plan(plan_for_target(
        RequestFormat::Responses,
        model,
        ResolvedTarget::from_resolved_model(alias.clone(), model)?,
    )?)
}

pub fn ensure_codex_model(value: &serde_json::Value) -> AppResult<()> {
    match route_for_responses_model(value)? {
        ResponsesRoute::CodexResponses => Ok(()),
        ResponsesRoute::BedrockMessages
        | ResponsesRoute::GoogleGenerateContent { .. }
        | ResponsesRoute::CursorAgent { .. }
        | ResponsesRoute::WindsurfChat { .. } => Err(AppError::ModelNotSupported(
            required_model(value)?.to_string(),
        )),
    }
}

fn route_from_plan(plan: crate::route::dispatch::DispatchPlan) -> AppResult<ResponsesRoute> {
    match plan.edge {
        DispatchEdge::ResponsesToResponsesCodex => Ok(ResponsesRoute::CodexResponses),
        DispatchEdge::ResponsesToAnthropicMessagesBedrock => Ok(ResponsesRoute::BedrockMessages),
        DispatchEdge::ResponsesToGoogleGenerateContentGoogle => {
            Ok(ResponsesRoute::GoogleGenerateContent {
                upstream_model: plan.target.upstream_model,
            })
        }
        DispatchEdge::ResponsesToCursorAgentCursor => Ok(ResponsesRoute::CursorAgent {
            upstream_model: plan.target.upstream_model,
        }),
        DispatchEdge::ResponsesToWindsurfChatWindsurf => Ok(ResponsesRoute::WindsurfChat {
            upstream_model: plan.target.upstream_model,
        }),
        _ => Err(AppError::ModelNotSupported(plan.requested_model)),
    }
}

pub fn required_model(value: &serde_json::Value) -> AppResult<&str> {
    value
        .get("model")
        .and_then(|value| value.as_str())
        .ok_or_else(|| AppError::BadRequest("missing model".into()))
}

fn codex_response_stream(
    stream: upstream::codex::CodexResponseStream,
) -> AppResult<UpstreamResponse> {
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/event-stream"),
    );
    Ok(UpstreamResponse::stream(
        "codex",
        StatusCode::OK,
        headers,
        stream,
    ))
}

async fn anthropic_messages_response_to_responses(
    response: UpstreamResponse,
    requested_model: &str,
    tool_context: ToolContext,
) -> AppResult<UpstreamResponse> {
    if !response.status.is_success() {
        return Ok(response);
    }

    let is_stream = response
        .headers
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.starts_with("text/event-stream"));
    let provider = response.provider;
    if is_stream {
        let mut translator =
            AnthropicSseStreamTranslator::with_model_and_context(requested_model, tool_context);
        let stream = response
            .body
            .into_data_stream()
            .map(move |chunk| match chunk {
                Ok(bytes) => translator.push_bytes(bytes),
                Err(error) => Err(AppError::Upstream(format!(
                    "Anthropic SSE stream failed: {error}"
                ))),
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
            provider,
            response.status,
            headers,
            stream,
        ));
    }

    let body = to_bytes(response.body, usize::MAX)
        .await
        .map_err(|error| AppError::Upstream(error.to_string()))?;

    let mut message: serde_json::Value = serde_json::from_slice(&body)?;
    if let Some(object) = message.as_object_mut() {
        object.insert(
            "model".into(),
            serde_json::Value::String(requested_model.to_string()),
        );
    }
    let responses = anthropic_message_to_responses_json_with_context(message, &tool_context)?;
    UpstreamResponse::json(provider, responses).map_err(AppError::Json)
}

async fn google_generate_content_response_to_responses(
    response: UpstreamResponse,
    requested_model: &str,
    tool_context: crate::adapter::google_responses::GoogleToolContext,
) -> AppResult<UpstreamResponse> {
    if !response.status.is_success() {
        return Ok(response);
    }

    let is_stream = response
        .headers
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.starts_with("text/event-stream"));
    let provider = response.provider;
    if is_stream {
        let mut translator =
            GoogleResponsesSseTranslator::with_tool_context(requested_model, tool_context);
        let stream = response
            .body
            .into_data_stream()
            .map(move |chunk| match chunk {
                Ok(bytes) => translator.push_bytes(bytes),
                Err(error) => Err(AppError::Upstream(format!(
                    "Google Responses SSE stream failed: {error}"
                ))),
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
            provider,
            response.status,
            headers,
            stream,
        ));
    }

    let body = to_bytes(response.body, usize::MAX)
        .await
        .map_err(|error| AppError::Upstream(error.to_string()))?;
    let google: serde_json::Value = serde_json::from_slice(&body)?;
    let responses =
        google_generate_content_to_responses_with_context(google, requested_model, &tool_context)?;
    UpstreamResponse::json(provider, responses).map_err(AppError::Json)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        compaction::RemoteCompactionPolicy,
        cursor_agent::{CursorAgentEvent, CursorFinishReason, CursorToolResult},
        route::dispatch::{DispatchAction, DispatchEdge, RequestFormat},
    };
    use axum::body::to_bytes;

    #[tokio::test]
    async fn anthropic_non_success_response_bypasses_success_conversion() {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );
        let body = Bytes::from_static(br#"{"error":{"type":"upstream","message":"bad"}}"#);
        let response = UpstreamResponse::bytes("bedrock", StatusCode::BAD_REQUEST, headers, body);

        let response = anthropic_messages_response_to_responses(
            response,
            "anthropic/claude-opus-4-7",
            ToolContext::default(),
        )
        .await
        .unwrap();

        assert_eq!(response.status, StatusCode::BAD_REQUEST);
        let body = to_bytes(response.body, usize::MAX).await.unwrap();
        assert_eq!(
            body,
            Bytes::from_static(br#"{"error":{"type":"upstream","message":"bad"}}"#)
        );
    }

    #[test]
    fn cursor_stream_done_context_stores_resumable_store_false_state() {
        let (_temp, state) = test_state();
        let plan = cursor_plan();
        let request = json!({
            "model": "composer-2-fast",
            "stream": true,
            "store": false,
            "input": "use lookup",
            "tools": [{
                "type": "function",
                "name": "lookup",
                "parameters": { "type": "object", "properties": {} }
            }]
        });
        let mut ctx = crate::adapter::cursor_events::ResponseContext::new(
            "composer-2-fast",
            "resp_stream_test",
        );
        for event in [
            CursorAgentEvent::ToolCallStarted {
                call_id: "call_lookup".into(),
                name: "lookup".into(),
                kind: crate::cursor_agent::CursorToolKind::Function,
                argument_index: 0,
            },
            CursorAgentEvent::ToolCallArgumentsDelta {
                call_id: "call_lookup".into(),
                delta: "{}".into(),
            },
            CursorAgentEvent::ToolCallDone {
                call_id: "call_lookup".into(),
                arguments: json!({}),
            },
            CursorAgentEvent::Done {
                finish_reason: CursorFinishReason::ToolCalls,
                response_id: "resp_stream_test".into(),
                conversation_id: "conv_stream_test".into(),
            },
        ] {
            let _ = cursor_responses::emit_event(&event, &mut ctx);
        }
        let response = cursor_response_from_context_if_done(&ctx).expect("Done finalizes response");

        store_cursor_response_state(
            &state,
            &plan,
            &request,
            response,
            request["input"].clone(),
            false,
            CursorClientProfile::GenericOpenAi,
        );

        assert!(
            state.public_response("resp_stream_test").is_none(),
            "store=false stays private"
        );
        assert!(
            state.continuation_response("resp_stream_test").is_some(),
            "store=false remains resumable"
        );
        let follow_up = json!({
            "model": "composer-2-fast",
            "previous_response_id": "resp_stream_test",
            "input": [{
                "type": "function_call_output",
                "call_id": "call_lookup",
                "output": "ok"
            }]
        });
        let key = cursor_continuation_key_for_request(&state, &plan, &follow_up)
            .expect("valid previous_response_id")
            .expect("continuation key");

        let result = CursorToolResult {
            call_id: "call_lookup".into(),
            output: json!("ok"),
            error: None,
        };
        validate_cursor_tool_results(&state, Some(&key), std::slice::from_ref(&result))
            .expect("first tool result consumes pending call");
        let duplicate =
            validate_cursor_tool_results(&state, Some(&key), std::slice::from_ref(&result))
                .expect_err("duplicate tool result rejected");
        assert!(
            format!("{duplicate:?}").contains("unknown call_id"),
            "duplicate error names call id mismatch: {duplicate:?}"
        );
    }

    #[test]
    fn cursor_stream_context_without_done_does_not_finalize() {
        let mut ctx = crate::adapter::cursor_events::ResponseContext::new(
            "composer-2-fast",
            "resp_incomplete",
        );
        let _ = cursor_responses::emit_event(
            &CursorAgentEvent::TextDelta {
                delta: "partial".into(),
                content_index: 0,
            },
            &mut ctx,
        );

        assert!(cursor_response_from_context_if_done(&ctx).is_none());
    }

    fn test_state() -> (tempfile::TempDir, AppState) {
        let temp = tempfile::tempdir().expect("tempdir");
        let root = temp.path().to_path_buf();
        let state = AppState::for_tests(root.join("codex"), root.join("ump"));
        (temp, state)
    }

    fn cursor_plan() -> crate::route::dispatch::DispatchPlan {
        crate::route::dispatch::DispatchPlan {
            source_format: RequestFormat::Responses,
            requested_model: "composer-2-fast".into(),
            target: ResolvedTarget {
                provider: Provider::Cursor,
                upstream_model: "composer-2-fast".into(),
                target_format: TargetFormat::CursorAgent,
            },
            remote_compaction_policy: RemoteCompactionPolicy::Local,
            edge: DispatchEdge::ResponsesToCursorAgentCursor,
            action: DispatchAction::CursorAgent,
        }
    }
}
