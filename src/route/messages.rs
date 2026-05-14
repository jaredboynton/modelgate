use axum::{
    body::Bytes,
    extract::State,
    http::{header, HeaderMap, HeaderValue, StatusCode},
};

use crate::{
    adapter::anthropic_responses::anthropic_messages_to_responses,
    model_alias::{resolve_model_required, Provider, ResolvedModel},
    upstream, AppError, AppResult, AppState, UpstreamResponse,
};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum MessagesRoute {
    BedrockMessages,
    CodexResponses,
}

pub async fn messages(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> AppResult<UpstreamResponse> {
    let value = serde_json::from_slice(&body)?;
    let model = required_model(&value)?;
    let alias = state.resolve_model_for_format(model, "anthropic_messages")?;
    match messages_route_for_alias(model, &alias)? {
        MessagesRoute::BedrockMessages => {
            upstream::bedrock::forward_messages_response(&state, value, headers).await
        }
        MessagesRoute::CodexResponses => {
            let mut responses = anthropic_messages_to_responses(value)?;
            responses["model"] = serde_json::Value::String(alias.upstream_model);
            codex_response_bytes(upstream::codex::responses(&state, responses).await?)
        }
    }
}

pub async fn count_tokens(State(state): State<AppState>, body: Bytes) -> AppResult<Bytes> {
    let value: serde_json::Value = serde_json::from_slice(&body)?;
    let model = required_model(&value)?;
    let alias = state.resolve_model_for_format(model, "anthropic_messages")?;
    messages_route_for_alias(model, &alias)?;
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
    mut resolve: F,
) -> AppResult<MessagesRoute>
where
    F: FnMut(&str) -> AppResult<ResolvedModel>,
{
    let model = required_model(value)?;
    let alias = resolve(model)?;
    messages_route_for_alias(model, &alias)
}

fn messages_route_for_alias(model: &str, alias: &ResolvedModel) -> AppResult<MessagesRoute> {
    match alias.provider {
        Provider::Bedrock => Ok(MessagesRoute::BedrockMessages),
        Provider::Codex if alias.upstream_model.starts_with("gpt-") => {
            Ok(MessagesRoute::CodexResponses)
        }
        _ => Err(AppError::ModelNotSupported(model.into())),
    }
}

fn required_model(value: &serde_json::Value) -> AppResult<&str> {
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
