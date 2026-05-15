use axum::{
    extract::{Query, State},
    http::Uri,
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::{
    codex_catalog::{
        codex_models_endpoint as shared_codex_models_endpoint, CodexCatalog, CodexCatalogRequest,
        CODEX_MODELS_URL, DEFAULT_CODEX_CLIENT_VERSION,
    },
    compaction::RemoteCompactionPolicy,
    model_alias::KNOWN_MODELS,
    upstream::codex::codex_headers,
    AppError, AppResult, AppState,
};

const HIDDEN_CODEX_MODEL_IDS: &[&str] = &["codex-auto-review"];

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
                "remote_compaction_policy": remote_compaction_policy_name(
                    model.default_remote_compaction_policy(),
                ),
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
            "remote_compaction_policy": remote_compaction_policy_name(
                configured.remote_compaction_policy,
            ),
        }));
    }
    if let Ok(descriptors) = crate::upstream::cursor::fetch_usable_models_for_state(&state).await {
        for descriptor in descriptors {
            let discovery = match descriptor.discovery {
                crate::upstream::cursor::models::DiscoverySource::Live => "live",
                crate::upstream::cursor::models::DiscoverySource::Fallback => "fallback",
            };
            if let Some(existing) = data
                .iter_mut()
                .find(|model| model["id"].as_str() == Some(descriptor.id.as_str()))
            {
                existing["cursor_discovery"] = json!(discovery);
                existing["context_window"] = json!(descriptor.context_window);
                existing["max_output_tokens"] = json!(descriptor.max_output_tokens);
                existing["supports_reasoning"] = json!(descriptor.supports_reasoning);
                continue;
            }
            data.push(json!({
                "id": descriptor.id,
                "object": "model",
                "owned_by": "cursor",
                "remote_compaction_policy": remote_compaction_policy_name(
                    RemoteCompactionPolicy::Local,
                ),
                "cursor_discovery": discovery,
                "context_window": descriptor.context_window,
                "max_output_tokens": descriptor.max_output_tokens,
                "supports_reasoning": descriptor.supports_reasoning,
            }));
        }
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
    codex_openai_models_projection(
        &state,
        query
            .client_version
            .as_deref()
            .unwrap_or(DEFAULT_CODEX_CLIENT_VERSION),
        query.include_hidden.unwrap_or(false),
    )
    .await
    .map(Json)
}

pub async fn codex_openai_models_projection(
    state: &AppState,
    client_version: &str,
    include_hidden: bool,
) -> AppResult<Value> {
    let client_version = required_client_version(Some(client_version))?;
    let catalog = codex_catalog_for_client_version(state, client_version).await?;
    Ok(add_codex_remote_compaction_policy(
        catalog.to_openai_models(include_hidden),
    ))
}

pub async fn validate_codex_catalog_request(
    state: &AppState,
    request: &Value,
    upstream_model: &str,
) -> AppResult<()> {
    let catalog = codex_catalog(state).await?;
    validate_codex_catalog_request_with_catalog(&catalog, request, upstream_model)
}

pub async fn validate_codex_catalog_websocket_request(
    state: &AppState,
    request: &Value,
    upstream_model: &str,
) -> AppResult<()> {
    if let Some(catalog) = state.codex_catalog.get_if_fresh() {
        return validate_codex_catalog_request_with_catalog(&catalog, request, upstream_model);
    }
    validate_codex_catalog_request(state, request, upstream_model).await
}

async fn codex_catalog(state: &AppState) -> AppResult<CodexCatalog> {
    if let Some(catalog) = state.codex_catalog.get_if_fresh() {
        return Ok(catalog);
    }
    let headers = codex_headers(state)?;
    state
        .codex_catalog
        .refresh_from_endpoint(&state.http, &headers, CODEX_MODELS_URL)
        .await
}

async fn codex_catalog_for_client_version(
    state: &AppState,
    client_version: &str,
) -> AppResult<CodexCatalog> {
    if client_version == state.codex_catalog.client_version() {
        return codex_catalog(state).await;
    }

    let headers = codex_headers(state)?;
    let url = shared_codex_models_endpoint(CODEX_MODELS_URL, client_version)?;
    let mut request = state.http.get(url);
    for (name, value) in headers.iter() {
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
    let raw = serde_json::from_str(&text)?;
    CodexCatalog::parse(client_version, &raw)
}

pub fn codex_models_endpoint(base_url: &str, client_version: Option<&str>) -> AppResult<String> {
    let client_version = required_client_version(client_version)?;
    shared_codex_models_endpoint(base_url, client_version)
}

pub fn codex_catalog_to_openai_models(
    client_version: Option<&str>,
    catalog: &Value,
    include_hidden: bool,
) -> AppResult<Value> {
    let client_version = required_client_version(client_version)?;
    Ok(add_codex_remote_compaction_policy(
        CodexCatalog::parse(client_version, catalog)?.to_openai_models(include_hidden),
    ))
}

fn add_codex_remote_compaction_policy(mut value: Value) -> Value {
    if let Some(models) = value.get_mut("data").and_then(Value::as_array_mut) {
        for model in models {
            if let Some(object) = model.as_object_mut() {
                object.insert("remote_compaction_policy".into(), json!("native"));
            }
        }
    }
    value
}

fn remote_compaction_policy_name(policy: RemoteCompactionPolicy) -> &'static str {
    match policy {
        RemoteCompactionPolicy::Native => "native",
        RemoteCompactionPolicy::ProxyVisibleSummary => "proxy_visible_summary",
        RemoteCompactionPolicy::Local => "local",
        RemoteCompactionPolicy::Off => "off",
    }
}

fn required_client_version(client_version: Option<&str>) -> AppResult<&str> {
    client_version
        .map(str::trim)
        .filter(|version| !version.is_empty())
        .ok_or_else(|| AppError::BadRequest("client_version is required for Codex models".into()))
}

fn is_hidden_codex_model(id: &str) -> bool {
    HIDDEN_CODEX_MODEL_IDS.contains(&id)
}

fn validate_codex_catalog_request_with_catalog(
    catalog: &CodexCatalog,
    request: &Value,
    upstream_model: &str,
) -> AppResult<()> {
    let reasoning_effort = request
        .get("reasoning")
        .and_then(Value::as_object)
        .and_then(|reasoning| reasoning.get("effort"))
        .and_then(Value::as_str);
    let service_tier = request.get("service_tier").and_then(Value::as_str);
    let verbosity = request
        .get("text")
        .and_then(Value::as_object)
        .and_then(|text| text.get("verbosity"))
        .or_else(|| request.get("verbosity"))
        .and_then(Value::as_str);
    let truncation = request.get("truncation").and_then(Value::as_str);
    let input_modalities = input_modalities(request);
    let output_modalities = string_array(request.get("output_modalities"))
        .or_else(|| string_array(request.get("modalities")))
        .unwrap_or_default();
    let input_modalities = input_modalities
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();
    let output_modalities = output_modalities
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();

    catalog.validate_request(CodexCatalogRequest {
        model: upstream_model,
        include_hidden: false,
        reasoning_effort,
        service_tier,
        verbosity,
        truncation,
        input_modalities: &input_modalities,
        output_modalities: &output_modalities,
    })?;
    Ok(())
}

fn input_modalities(request: &Value) -> Vec<String> {
    let mut modalities = Vec::new();
    collect_input_modalities(request.get("input"), &mut modalities);
    modalities.sort();
    modalities.dedup();
    modalities
}

fn collect_input_modalities(value: Option<&Value>, modalities: &mut Vec<String>) {
    match value {
        Some(Value::String(_)) => push_modality(modalities, "text"),
        Some(Value::Array(values)) => {
            for value in values {
                collect_input_modalities(Some(value), modalities);
            }
        }
        Some(Value::Object(object)) => {
            match object.get("type").and_then(Value::as_str) {
                Some("input_text" | "text") => push_modality(modalities, "text"),
                Some("input_image" | "image") => push_modality(modalities, "image"),
                Some("message") => collect_input_modalities(object.get("content"), modalities),
                _ => {}
            }
            collect_input_modalities(object.get("content"), modalities);
        }
        _ => {}
    }
}

fn push_modality(modalities: &mut Vec<String>, modality: &str) {
    if !modalities.iter().any(|value| value == modality) {
        modalities.push(modality.to_string());
    }
}

fn string_array(value: Option<&Value>) -> Option<Vec<String>> {
    let values = value?.as_array()?;
    Some(
        values
            .iter()
            .filter_map(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned)
            .collect(),
    )
}
