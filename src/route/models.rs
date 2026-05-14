use axum::{
    extract::{Query, State},
    http::Uri,
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::{
    model_alias::KNOWN_MODELS, upstream::codex::codex_headers, AppError, AppResult, AppState,
};

const HIDDEN_CODEX_MODEL_IDS: &[&str] = &["codex-auto-review"];
const CODEX_MODELS_URL: &str = "https://chatgpt.com/backend-api/codex/models";
const DEFAULT_CODEX_CLIENT_VERSION: &str = "26.506.31421";

#[derive(Debug, Deserialize)]
pub struct ModelsQuery {
    client_version: Option<String>,
    include_hidden: Option<bool>,
}

pub async fn models(
    State(state): State<AppState>,
    uri: Uri,
    Query(query): Query<ModelsQuery>,
) -> AppResult<Json<Value>> {
    if uri.path().starts_with("/api/provider/openai/") {
        return codex_models(State(state), Query(query)).await;
    }

    let mut data = KNOWN_MODELS
        .iter()
        .filter(|model| !is_hidden_codex_model(model.id))
        .map(|model| {
            json!({
                "id": model.id,
                "object": "model",
                "owned_by": format!("{:?}", model.provider).to_lowercase(),
            })
        })
        .collect::<Vec<_>>();
    for configured in state.routing_config.configured_models()? {
        if data
            .iter()
            .any(|model| model["id"].as_str() == Some(configured.id.as_str()))
        {
            continue;
        }
        data.push(json!({
            "id": configured.id,
            "object": "model",
            "owned_by": format!("{:?}", configured.provider).to_lowercase(),
        }));
    }

    Ok(Json(json!({
        "object": "list",
        "data": data
    })))
}

async fn codex_models(
    State(state): State<AppState>,
    Query(query): Query<ModelsQuery>,
) -> AppResult<Json<Value>> {
    let client_version = query
        .client_version
        .as_deref()
        .unwrap_or(DEFAULT_CODEX_CLIENT_VERSION);
    let url = codex_models_endpoint(CODEX_MODELS_URL, Some(client_version))?;
    let mut request = state.http.get(url);
    for (name, value) in codex_headers(&state)?.iter() {
        request = request.header(name, value);
    }
    let response = request
        .send()
        .await
        .map_err(|error| AppError::Upstream(format!("Codex models request failed: {error}")))?;
    let status = response.status();
    let text = response
        .text()
        .await
        .map_err(|error| AppError::Upstream(format!("Codex models body failed: {error}")))?;
    if !status.is_success() {
        return Err(AppError::Upstream(format!(
            "Codex models returned {status}: {text}"
        )));
    }
    let catalog: Value = serde_json::from_str(&text)?;
    codex_catalog_to_openai_models(
        Some(client_version),
        &catalog,
        query.include_hidden.unwrap_or(false),
    )
    .map(Json)
}

pub fn codex_models_endpoint(base_url: &str, client_version: Option<&str>) -> AppResult<String> {
    let client_version = required_client_version(client_version)?;
    let separator = if base_url.contains('?') { '&' } else { '?' };
    Ok(format!(
        "{base_url}{separator}client_version={client_version}"
    ))
}

pub fn codex_catalog_to_openai_models(
    client_version: Option<&str>,
    catalog: &Value,
    include_hidden: bool,
) -> AppResult<Value> {
    required_client_version(client_version)?;
    let models = catalog
        .get("models")
        .and_then(Value::as_array)
        .ok_or_else(|| AppError::BadRequest("Codex models catalog missing models".into()))?;
    let data = models
        .iter()
        .filter_map(|model| codex_catalog_model_id(model, include_hidden))
        .map(|id| {
            json!({
                "id": id,
                "object": "model",
                "owned_by": "codex",
            })
        })
        .collect::<Vec<_>>();

    Ok(json!({
        "object": "list",
        "data": data
    }))
}

fn required_client_version(client_version: Option<&str>) -> AppResult<&str> {
    client_version
        .map(str::trim)
        .filter(|version| !version.is_empty())
        .ok_or_else(|| AppError::BadRequest("client_version is required for Codex models".into()))
}

fn codex_catalog_model_id(model: &Value, include_hidden: bool) -> Option<&str> {
    let id = model.get("slug").and_then(Value::as_str)?;
    if !include_hidden && is_hidden_codex_catalog_model(model, id) {
        return None;
    }
    if model
        .get("supported_in_api")
        .and_then(Value::as_bool)
        .is_some_and(|supported| !supported)
    {
        return None;
    }
    Some(id)
}

fn is_hidden_codex_catalog_model(model: &Value, id: &str) -> bool {
    is_hidden_codex_model(id)
        || model
            .get("visibility")
            .and_then(Value::as_str)
            .is_some_and(|visibility| visibility != "list")
}

fn is_hidden_codex_model(id: &str) -> bool {
    HIDDEN_CODEX_MODEL_IDS.contains(&id)
}
