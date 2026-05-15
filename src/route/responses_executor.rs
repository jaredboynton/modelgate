use axum::{
    body::{to_bytes, Bytes},
    http::{header, HeaderMap, HeaderValue, StatusCode},
};
use std::collections::HashMap;

use futures::StreamExt;
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
    cursor_agent::{CursorContinuationKey, CursorRoute, CursorToolCall},
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
    upstream, AppError, AppResult, AppState, UpstreamResponse,
};

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ResponsesRoute {
    CodexResponses,
    BedrockMessages,
    GoogleGenerateContent { upstream_model: String },
    CursorAgent { upstream_model: String },
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
            codex_response_bytes(upstream::codex::responses_prepared(state, prepared).await?)
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
    crate::upstream::cursor::workspace::attach_to_request(&mut request, headers).await;

    crate::upstream::cursor::ensure_credentials(state).await?;
    validate_cursor_tool_results(
        state,
        request.continuation_key.as_ref(),
        &request.tool_results,
    )?;

    if stream {
        let events = upstream::cursor::run::run(state, request).await;
        let mut ctx = crate::adapter::cursor_events::ResponseContext::new(
            &plan.requested_model,
            format!("resp_{}", uuid::Uuid::new_v4().simple()),
        );
        let stream = events
            .map(move |event| {
                let mut bytes = Vec::new();
                for frame in cursor_responses::emit_event(&event, &mut ctx) {
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
    let response_id = response["id"].as_str().unwrap_or_default().to_string();
    let conversation_id = done
        .as_ref()
        .map(|(_, conversation_id)| conversation_id.clone());
    if let Some(conversation_id) = conversation_id.as_ref() {
        let key = cursor_continuation_key(
            CursorRoute::Responses,
            plan,
            &value,
            &response_id,
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
                stable_field_hash: [0u8; 32],
                response_id: response_id.clone(),
                conversation_id: conversation_id.clone(),
                blob_store: HashMap::new(),
            },
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
    UpstreamResponse::json("cursor", response).map_err(AppError::from)
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
        | ResponsesRoute::CursorAgent { .. } => Err(AppError::ModelNotSupported(
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
        _ => Err(AppError::ModelNotSupported(plan.requested_model)),
    }
}

pub fn required_model(value: &serde_json::Value) -> AppResult<&str> {
    value
        .get("model")
        .and_then(|value| value.as_str())
        .ok_or_else(|| AppError::BadRequest("missing model".into()))
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
    use axum::body::to_bytes;

    use super::*;

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
}
