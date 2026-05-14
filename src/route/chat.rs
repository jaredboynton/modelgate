use axum::{
    body::Bytes,
    extract::State,
    http::{header, HeaderMap, HeaderValue, StatusCode},
};

use crate::{
    adapter::anthropic_responses::{
        chat_completions_to_responses, responses_to_anthropic_messages,
    },
    model_alias::{resolve_model_required, Provider, ResolvedModel},
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
    let model = required_model(&value)?;
    let alias = state.resolve_model_for_format(model, "chat_completions")?;
    match chat_route_for_alias(model, &alias)? {
        ChatRoute::CodexResponses => {
            let mut responses = chat_completions_to_responses(value)?;
            responses["model"] = serde_json::Value::String(alias.upstream_model);
            codex_response_bytes(upstream::codex::responses(&state, responses).await?)
        }
        ChatRoute::BedrockMessages => {
            let messages = responses_to_anthropic_messages(chat_completions_to_responses(value)?)?;
            upstream::bedrock::forward_messages_response(&state, messages, headers).await
        }
    }
}

pub fn route_for_chat_model(value: &serde_json::Value) -> AppResult<ChatRoute> {
    route_for_chat_model_with_resolver(value, resolve_model_required)
}

pub fn route_for_chat_model_with_resolver<F>(
    value: &serde_json::Value,
    mut resolve: F,
) -> AppResult<ChatRoute>
where
    F: FnMut(&str) -> AppResult<ResolvedModel>,
{
    let model = required_model(value)?;
    let alias = resolve(model)?;
    chat_route_for_alias(model, &alias)
}

fn chat_route_for_alias(model: &str, alias: &ResolvedModel) -> AppResult<ChatRoute> {
    match alias.provider {
        Provider::Codex if alias.upstream_model.starts_with("gpt-") => {
            Ok(ChatRoute::CodexResponses)
        }
        Provider::Bedrock
            if model.starts_with("anthropic/") || alias.upstream_model.contains("claude") =>
        {
            Ok(ChatRoute::BedrockMessages)
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
