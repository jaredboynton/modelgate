use axum::{
    http::{header, HeaderMap, HeaderValue, Response, StatusCode},
    response::IntoResponse,
    Json,
};
use serde_json::{json, Value};

use crate::{
    compaction::{
        encode_ump_pack_from_env, pack_context_from_headers, validate_compaction_carriers,
        CompactVisibleContext, CompactionLimits, CompactionPackContext, RemoteCompactionPolicy,
    },
    error::{openai_error_body, CompactionHttpError},
    route::dispatch::{plan_with_state, DispatchAction, RequestFormat},
    upstream, AppError, AppResult, AppState,
};

const CODE_PROXY_COMPACTION_UNAVAILABLE: &str = "proxy_compaction_unavailable";
const CODE_NATIVE_COMPACTION_UNAVAILABLE: &str = "native_compaction_unavailable";

pub async fn compact_responses(
    state: &AppState,
    headers: HeaderMap,
    value: Value,
) -> AppResult<Response<axum::body::Body>> {
    validate_compaction_input(&value)?;
    let plan = plan_with_state(state, RequestFormat::Responses, &value)?;
    validate_compaction_carriers(
        value.get("input").expect("validated input exists"),
        &plan.target,
        CompactionLimits::default(),
    )?;

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
            CODE_PROXY_COMPACTION_UNAVAILABLE,
            "server_error",
            "remote compaction unavailable for target; use local compaction",
        )
        .into()),
        RemoteCompactionPolicy::ProxyVisibleSummary => {
            let context = pack_context_from_headers(&headers, "POST /v1/responses", &plan.target)?;
            proxy_visible_compaction_response(&value, &context)
        }
        RemoteCompactionPolicy::Native => match plan.action {
            DispatchAction::CodexResponses => compact_codex_responses(state, headers, value).await,
            DispatchAction::BedrockAnthropicMessages
            | DispatchAction::GoogleGenerateContent
            | DispatchAction::CursorAgent
            | DispatchAction::WindsurfChat => Err(CompactionHttpError::new(
                StatusCode::BAD_REQUEST,
                "unsupported_compaction_item_for_target",
                "invalid_request",
                "native compaction is not supported for the resolved target",
            )
            .into()),
        },
    }
}

pub fn is_v2_context_compaction_trigger(value: &Value) -> AppResult<bool> {
    let Some(input) = value.get("input") else {
        return Ok(false);
    };
    let Some(items) = input.as_array() else {
        return Ok(false);
    };
    let mut found = false;
    for item in items {
        if item.get("type").and_then(Value::as_str) != Some("context_compaction") {
            continue;
        }
        if item
            .get("encrypted_content")
            .is_some_and(|value| !value.is_null())
        {
            continue;
        }
        if found {
            return Err(CompactionHttpError::new(
                StatusCode::BAD_REQUEST,
                "too_many_compaction_items",
                "invalid_request",
                "multiple context_compaction items are not supported",
            )
            .into());
        }
        found = true;
    }
    Ok(found)
}

pub fn context_compaction_unavailable_frame(status: StatusCode) -> Value {
    json!({
        "type": "error",
        "status": status.as_u16(),
        "error": {
            "code": CODE_PROXY_COMPACTION_UNAVAILABLE,
            "message": "remote compaction unavailable for target; use local compaction"
        }
    })
}

fn validate_compaction_input(value: &Value) -> AppResult<()> {
    if !value.is_object() {
        return Err(compaction_bad_request(
            "Responses compaction request must be a JSON object",
        ));
    }
    if value.get("model").and_then(Value::as_str).is_none() {
        return Err(compaction_bad_request("missing model"));
    }
    match value.get("input") {
        Some(Value::Array(_)) => Ok(()),
        Some(_) => Err(compaction_bad_request("input must be an array")),
        None => Err(compaction_bad_request("missing input")),
    }
}

fn compaction_bad_request(message: &'static str) -> AppError {
    CompactionHttpError::new(
        StatusCode::BAD_REQUEST,
        "invalid_compaction_input",
        "invalid_request",
        message,
    )
    .into()
}

pub fn proxy_visible_compaction_response(
    value: &Value,
    context: &CompactionPackContext,
) -> AppResult<Response<axum::body::Body>> {
    let pack = encode_ump_pack_from_env(
        compact_visible_context_from_input(value.get("input"))?,
        context,
    )?;
    Ok((
        StatusCode::OK,
        Json(json!({
            "object": "response.compaction",
            "output": [{
                "type": "compaction",
                "encrypted_content": pack
            }]
        })),
    )
        .into_response())
}

pub fn proxy_visible_context_compaction_item(
    value: &Value,
    context: &CompactionPackContext,
) -> AppResult<Value> {
    Ok(json!({
        "id": format!("ctxc_{}", uuid::Uuid::new_v4().simple()),
        "type": "context_compaction",
        "encrypted_content": encode_ump_pack_from_env(
            compact_visible_context_from_input(value.get("input"))?,
            context
        )?,
    }))
}

fn compact_visible_context_from_input(input: Option<&Value>) -> AppResult<CompactVisibleContext> {
    let summary = input
        .map(summarize_visible_input)
        .filter(|summary| !summary.trim().is_empty());
    Ok(CompactVisibleContext {
        task_objective: summary.clone(),
        durable_constraints: Vec::new(),
        summary,
        context_degraded: true,
    })
}

fn summarize_visible_input(value: &Value) -> String {
    match value {
        Value::String(text) => text.chars().take(512).collect(),
        Value::Array(items) => items
            .iter()
            .filter_map(item_visible_text)
            .collect::<Vec<_>>()
            .join("\n"),
        other => item_visible_text(other).unwrap_or_default(),
    }
}

fn item_visible_text(value: &Value) -> Option<String> {
    let object = value.as_object()?;
    if object.get("type").and_then(Value::as_str) == Some("context_compaction") {
        return None;
    }
    if let Some(text) = object.get("text").and_then(Value::as_str) {
        return Some(text.chars().take(512).collect());
    }
    if let Some(content) = object.get("content") {
        return Some(summarize_visible_input(content));
    }
    None
}

async fn compact_codex_responses(
    state: &AppState,
    headers: HeaderMap,
    mut value: Value,
) -> AppResult<Response<axum::body::Body>> {
    prepare_codex_compaction_request(state, &mut value)?;
    let compact_url = codex_compact_url(&state.runtime.codex_responses_http_url);
    let mut upstream_headers = upstream::codex::codex_headers(state)?;
    copy_compaction_binding_header(&headers, &mut upstream_headers, "session-id");
    copy_compaction_binding_header(&headers, &mut upstream_headers, "thread-id");
    let response = state
        .specter
        .post(compact_url)
        .headers(specter::Headers::from(upstream_headers))
        .json(&value)
        .send()
        .await
        .map_err(|error| AppError::Upstream(format!("Codex compact request failed: {error}")))?;
    let status = response.status();
    let body = response.into_body();

    if matches!(
        status,
        StatusCode::NOT_FOUND | StatusCode::METHOD_NOT_ALLOWED | StatusCode::NOT_IMPLEMENTED
    ) {
        return Ok(compaction_error_response(
            StatusCode::SERVICE_UNAVAILABLE,
            CODE_NATIVE_COMPACTION_UNAVAILABLE,
            "upstream_error",
            "native compaction unavailable for target",
        ));
    }
    if !status.is_success() {
        return Ok(compaction_error_response(
            status,
            CODE_NATIVE_COMPACTION_UNAVAILABLE,
            "upstream_error",
            String::from_utf8_lossy(&body),
        ));
    }

    let body_json: Value = serde_json::from_slice(&body)?;
    validate_codex_compaction_output(&body_json)?;
    Ok((status, Json(body_json)).into_response())
}

fn prepare_codex_compaction_request(state: &AppState, value: &mut Value) -> AppResult<()> {
    let model = value
        .get("model")
        .and_then(Value::as_str)
        .ok_or_else(|| AppError::BadRequest("missing model".into()))?;
    let alias = state.resolve_model_for_format(model, RequestFormat::Responses.as_str())?;
    value["model"] = Value::String(alias.upstream_model);
    Ok(())
}

fn copy_compaction_binding_header(
    inbound: &HeaderMap,
    outbound: &mut HeaderMap,
    name: &'static str,
) {
    if let Some(value) = inbound.get(name) {
        outbound.insert(name, value.clone());
    }
}

fn validate_codex_compaction_output(value: &Value) -> AppResult<()> {
    let Some(output) = value.get("output").and_then(Value::as_array) else {
        return Err(AppError::Upstream(
            "Codex compact response missing output array".into(),
        ));
    };
    let has_compaction = output.iter().any(|item| {
        matches!(
            item.get("type").and_then(Value::as_str),
            Some("compaction" | "context_compaction")
        ) && item
            .get("encrypted_content")
            .and_then(Value::as_str)
            .is_some_and(|content| !content.is_empty())
    });
    if !has_compaction {
        return Err(AppError::Upstream(
            "Codex compact response missing encrypted compaction item".into(),
        ));
    }
    Ok(())
}

fn codex_compact_url(responses_url: &str) -> String {
    format!("{}/compact", responses_url.trim_end_matches('/'))
}

fn compaction_error_response(
    status: StatusCode,
    code: &'static str,
    error_type: &'static str,
    message: impl ToString,
) -> Response<axum::body::Body> {
    let mut response = (
        status,
        Json(openai_error_body(
            message.to_string(),
            error_type,
            None,
            Some(code),
        )),
    )
        .into_response();
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/json"),
    );
    response
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compact_input_requires_model_and_array_input() {
        let missing_model = validate_compaction_input(&json!({ "input": [] })).unwrap_err();
        assert_eq!(missing_model.status(), StatusCode::BAD_REQUEST);
        assert_eq!(missing_model.code(), Some("invalid_compaction_input"));

        let missing_input = validate_compaction_input(&json!({ "model": "gpt-5.5" })).unwrap_err();
        assert_eq!(missing_input.status(), StatusCode::BAD_REQUEST);
        assert_eq!(missing_input.code(), Some("invalid_compaction_input"));

        let non_array = validate_compaction_input(&json!({ "model": "gpt-5.5", "input": "hello" }))
            .unwrap_err();
        assert_eq!(non_array.status(), StatusCode::BAD_REQUEST);
        assert_eq!(non_array.code(), Some("invalid_compaction_input"));

        validate_compaction_input(&json!({ "model": "gpt-5.5", "input": [] })).unwrap();
    }

    #[test]
    fn v2_context_compaction_trigger_rejects_malformed_carriers() {
        assert!(is_v2_context_compaction_trigger(&json!({
            "input": [{ "type": "context_compaction" }]
        }))
        .unwrap());

        let duplicate = is_v2_context_compaction_trigger(&json!({
            "input": [
                { "type": "context_compaction" },
                { "type": "context_compaction" }
            ]
        }))
        .unwrap_err();
        assert_eq!(duplicate.code(), Some("too_many_compaction_items"));

        let encrypted = is_v2_context_compaction_trigger(&json!({
            "input": [{ "type": "context_compaction", "encrypted_content": "opaque" }]
        }))
        .unwrap();
        assert!(!encrypted);
    }

    #[test]
    fn codex_compact_url_is_responses_sibling() {
        assert_eq!(
            codex_compact_url("https://chatgpt.com/backend-api/codex/responses/"),
            "https://chatgpt.com/backend-api/codex/responses/compact"
        );
    }

    #[test]
    fn codex_compact_output_requires_encrypted_carrier() {
        validate_codex_compaction_output(&json!({
            "output": [{ "type": "compaction", "encrypted_content": "opaque" }]
        }))
        .unwrap();

        assert!(validate_codex_compaction_output(&json!({ "output": [] })).is_err());
        assert!(validate_codex_compaction_output(&json!({
            "output": [{ "type": "message" }]
        }))
        .is_err());
    }
}
