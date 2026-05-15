use axum::{
    body::{to_bytes, Bytes},
    http::{header, HeaderMap, HeaderValue, StatusCode},
};
use futures::StreamExt;

use crate::{
    adapter::{
        anthropic_responses::{
            anthropic_message_to_responses_json_with_context,
            responses_to_anthropic_messages_with_context, AnthropicSseStreamTranslator,
            ToolContext,
        },
        google_responses::{
            google_generate_content_to_responses_with_context, is_google_responses_stream_request,
            responses_to_google_generate_content_with_context, GoogleResponsesSseTranslator,
        },
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
pub enum ResponsesRoute {
    CodexResponses,
    BedrockMessages,
    GoogleGenerateContent { upstream_model: String },
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
        ResponsesRoute::BedrockMessages | ResponsesRoute::GoogleGenerateContent { .. } => Err(
            AppError::ModelNotSupported(required_model(value)?.to_string()),
        ),
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
