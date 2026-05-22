use std::{
    collections::BTreeMap,
    env,
    future::Future,
    sync::{Arc, Mutex},
    time::{Duration, SystemTime},
};

use http::HeaderMap;
use serde_json::{json, Value};

use crate::{AppError, AppResult};

pub const CODEX_MODELS_URL: &str = "https://chatgpt.com/backend-api/codex/models";
pub const DEFAULT_CODEX_CLIENT_VERSION: &str = "26.506.31421";
pub const DEFAULT_CODEX_CATALOG_TTL_SECS: u64 = 60 * 60;

const HIDDEN_CODEX_MODEL_IDS: &[&str] = &["codex-auto-review"];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CodexCatalogConfig {
    pub client_version: String,
    pub ttl: Duration,
}

#[derive(Clone, Debug)]
pub struct CodexCatalogCache {
    inner: Arc<Mutex<Option<CachedCodexCatalog>>>,
    refresh_lock: Arc<tokio::sync::Mutex<()>>,
    config: CodexCatalogConfig,
}

#[derive(Clone, Debug)]
struct CachedCodexCatalog {
    catalog: Arc<CodexCatalog>,
    fetched_at: SystemTime,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CodexCatalog {
    pub client_version: String,
    pub models: BTreeMap<String, CodexCatalogModel>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CodexCatalogModel {
    pub slug: String,
    pub visibility: Option<String>,
    pub hidden: bool,
    pub supported_in_api: bool,
    pub reasoning_levels: Vec<String>,
    pub service_tiers: Vec<String>,
    pub verbosity: Vec<String>,
    pub truncation_policy: Vec<String>,
    pub input_modalities: Vec<String>,
    pub output_modalities: Vec<String>,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CodexCatalogRequest<'a> {
    pub model: &'a str,
    pub include_hidden: bool,
    pub reasoning_effort: Option<&'a str>,
    pub service_tier: Option<&'a str>,
    pub verbosity: Option<&'a str>,
    pub truncation: Option<&'a str>,
    pub input_modalities: &'a [&'a str],
    pub output_modalities: &'a [&'a str],
}

impl CodexCatalogConfig {
    pub fn from_env() -> Result<Self, String> {
        Self::from_env_vars(|name| env::var(name))
    }

    pub fn from_env_vars<F>(mut var: F) -> Result<Self, String>
    where
        F: FnMut(&str) -> Result<String, env::VarError>,
    {
        let client_version = var("UMP_V2_CODEX_CLIENT_VERSION")
            .unwrap_or_else(|_| DEFAULT_CODEX_CLIENT_VERSION.to_string());
        let client_version = validate_client_version(&client_version)?;
        let ttl_secs = match var("UMP_V2_CODEX_CATALOG_TTL_SECS") {
            Ok(value) => value
                .parse::<u64>()
                .map_err(|err| format!("invalid UMP_V2_CODEX_CATALOG_TTL_SECS: {err}"))?,
            Err(_) => DEFAULT_CODEX_CATALOG_TTL_SECS,
        };

        Ok(Self {
            client_version,
            ttl: Duration::from_secs(ttl_secs),
        })
    }
}

impl Default for CodexCatalogConfig {
    fn default() -> Self {
        Self {
            client_version: DEFAULT_CODEX_CLIENT_VERSION.to_string(),
            ttl: Duration::from_secs(DEFAULT_CODEX_CATALOG_TTL_SECS),
        }
    }
}

impl CodexCatalogCache {
    pub fn new(config: CodexCatalogConfig) -> Self {
        Self {
            inner: Arc::new(Mutex::new(None)),
            refresh_lock: Arc::new(tokio::sync::Mutex::new(())),
            config,
        }
    }

    pub fn config(&self) -> &CodexCatalogConfig {
        &self.config
    }

    pub fn client_version(&self) -> &str {
        &self.config.client_version
    }

    pub fn ttl(&self) -> Duration {
        self.config.ttl
    }

    pub fn get_if_fresh(&self) -> Option<Arc<CodexCatalog>> {
        self.lock()
            .as_ref()
            .filter(|cached| !cached.is_expired(self.config.ttl))
            .map(|cached| Arc::clone(&cached.catalog))
    }

    pub fn get_latest(&self) -> Option<Arc<CodexCatalog>> {
        self.lock()
            .as_ref()
            .map(|cached| Arc::clone(&cached.catalog))
    }

    pub fn store_validated(&self, raw: &Value) -> AppResult<Arc<CodexCatalog>> {
        let catalog = CodexCatalog::parse(self.client_version(), raw)?;
        Ok(self.replace(catalog))
    }

    pub async fn get_or_refresh_with<F, Fut>(&self, fetch: F) -> AppResult<Arc<CodexCatalog>>
    where
        F: FnOnce(String) -> Fut,
        Fut: Future<Output = AppResult<Value>>,
    {
        if let Some(catalog) = self.get_if_fresh() {
            return Ok(catalog);
        }

        let _refresh = self.refresh_lock.lock().await;
        if let Some(catalog) = self.get_if_fresh() {
            return Ok(catalog);
        }

        let raw = fetch(self.client_version().to_string()).await?;
        self.store_validated(&raw)
    }

    pub async fn refresh_from_endpoint(
        &self,
        http: &reqwest::Client,
        headers: &HeaderMap,
        base_url: &str,
    ) -> AppResult<Arc<CodexCatalog>> {
        self.get_or_refresh_with(|client_version| async move {
            let url = codex_models_endpoint(base_url, &client_version)?;
            let mut request = http.get(url);
            for (name, value) in headers.iter() {
                request = request.header(name, value);
            }
            let response = request.send().await.map_err(|error| {
                AppError::Upstream(format!("Codex models request failed: {error}"))
            })?;
            let status = response.status();
            let text = response.text().await.map_err(|error| {
                AppError::Upstream(format!("Codex models body failed: {error}"))
            })?;
            if !status.is_success() {
                return Err(AppError::Upstream(format!(
                    "Codex models returned {status}: {text}"
                )));
            }
            serde_json::from_str(&text).map_err(AppError::Json)
        })
        .await
    }

    pub fn replace(&self, catalog: CodexCatalog) -> Arc<CodexCatalog> {
        let catalog = Arc::new(catalog);
        *self.lock() = Some(CachedCodexCatalog {
            catalog: Arc::clone(&catalog),
            fetched_at: SystemTime::now(),
        });
        catalog
    }

    pub fn clear(&self) {
        *self.lock() = None;
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Option<CachedCodexCatalog>> {
        self.inner.lock().expect("codex catalog mutex poisoned")
    }
}

impl Default for CodexCatalogCache {
    fn default() -> Self {
        Self::new(CodexCatalogConfig::default())
    }
}

impl CachedCodexCatalog {
    fn is_expired(&self, ttl: Duration) -> bool {
        self.fetched_at
            .elapsed()
            .map(|elapsed| elapsed >= ttl)
            .unwrap_or(true)
    }
}

impl CodexCatalog {
    pub fn parse(client_version: &str, raw: &Value) -> AppResult<Self> {
        let client_version =
            validate_client_version(client_version).map_err(AppError::BadRequest)?;
        let models = raw
            .get("models")
            .and_then(Value::as_array)
            .ok_or_else(|| AppError::BadRequest("Codex models catalog missing models".into()))?;
        let models = models
            .iter()
            .map(parse_model)
            .collect::<AppResult<Vec<_>>>()?
            .into_iter()
            .map(|model| (model.slug.clone(), model))
            .collect::<BTreeMap<_, _>>();

        Ok(Self {
            client_version,
            models,
        })
    }

    pub fn model(&self, slug: &str) -> Option<&CodexCatalogModel> {
        self.models.get(slug)
    }

    pub fn validate_model(
        &self,
        slug: &str,
        include_hidden: bool,
    ) -> AppResult<&CodexCatalogModel> {
        let model = self
            .model(slug)
            .ok_or_else(|| AppError::ModelNotSupported(slug.to_string()))?;
        if !include_hidden && model.hidden {
            return Err(AppError::ModelNotSupported(slug.to_string()));
        }
        if !model.supported_in_api {
            return Err(AppError::ModelNotSupported(slug.to_string()));
        }
        Ok(model)
    }

    pub fn validate_request(
        &self,
        request: CodexCatalogRequest<'_>,
    ) -> AppResult<&CodexCatalogModel> {
        let model = self.validate_model(request.model, request.include_hidden)?;
        validate_optional_capability(
            model,
            "reasoning.effort",
            request.reasoning_effort,
            &model.reasoning_levels,
        )?;
        validate_optional_capability(
            model,
            "service_tier",
            request.service_tier,
            &model.service_tiers,
        )?;
        validate_optional_capability(model, "verbosity", request.verbosity, &model.verbosity)?;
        validate_optional_capability(
            model,
            "truncation",
            request.truncation,
            &model.truncation_policy,
        )?;
        validate_modalities(
            model,
            "input modality",
            request.input_modalities,
            &model.input_modalities,
        )?;
        validate_modalities(
            model,
            "output modality",
            request.output_modalities,
            &model.output_modalities,
        )?;
        Ok(model)
    }

    pub fn to_openai_models(&self, include_hidden: bool) -> Value {
        let data = self
            .models
            .values()
            .filter(|model| model.supported_in_api)
            .filter(|model| include_hidden || !model.hidden)
            .map(|model| {
                json!({
                    "id": model.slug,
                    "object": "model",
                    "owned_by": "codex",
                })
            })
            .collect::<Vec<_>>();

        json!({
            "object": "list",
            "data": data,
        })
    }
}

pub fn codex_models_endpoint(base_url: &str, client_version: &str) -> AppResult<String> {
    let client_version = validate_client_version(client_version).map_err(AppError::BadRequest)?;
    let separator = if base_url.contains('?') { '&' } else { '?' };
    Ok(format!(
        "{base_url}{separator}client_version={client_version}"
    ))
}

fn parse_model(value: &Value) -> AppResult<CodexCatalogModel> {
    let object = value
        .as_object()
        .ok_or_else(|| AppError::BadRequest("Codex catalog model must be an object".into()))?;
    let slug = object
        .get("slug")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|slug| !slug.is_empty())
        .ok_or_else(|| AppError::BadRequest("Codex catalog model missing slug".into()))?
        .to_string();
    let visibility = object
        .get("visibility")
        .and_then(Value::as_str)
        .map(ToOwned::to_owned);
    let hidden = HIDDEN_CODEX_MODEL_IDS.contains(&slug.as_str())
        || visibility
            .as_deref()
            .is_some_and(|visibility| visibility != "list");
    let supported_in_api = object
        .get("supported_in_api")
        .and_then(Value::as_bool)
        .unwrap_or(false);

    Ok(CodexCatalogModel {
        slug,
        visibility,
        hidden,
        supported_in_api,
        reasoning_levels: first_string_list(
            value,
            &[
                "reasoning_levels",
                "reasoning_efforts",
                "supported_reasoning_levels",
                "supported_reasoning_efforts",
            ],
        ),
        service_tiers: first_string_list(value, &["service_tiers", "supported_service_tiers"]),
        verbosity: supported_verbosity(value),
        truncation_policy: first_string_list(
            value,
            &["truncation_policy", "truncation", "supported_truncation"],
        ),
        input_modalities: first_string_list(
            value,
            &["input_modalities", "supported_input_modalities"],
        ),
        output_modalities: first_string_list(
            value,
            &[
                "output_modalities",
                "supported_output_modalities",
                "modalities",
            ],
        ),
    })
}

fn validate_client_version(value: &str) -> Result<String, String> {
    let value = value.trim();
    if value.is_empty() {
        return Err("client_version is required for Codex models".into());
    }
    if !value
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_'))
    {
        return Err(
            "client_version may only contain ASCII letters, numbers, dots, dashes, or underscores"
                .into(),
        );
    }
    Ok(value.to_string())
}

fn first_string_list(value: &Value, keys: &[&str]) -> Vec<String> {
    keys.iter()
        .find_map(|key| string_list(value.get(*key), &["id", "value", "name", "effort", "mode"]))
        .unwrap_or_default()
}

fn supported_verbosity(value: &Value) -> Vec<String> {
    let explicit = first_string_list(value, &["verbosity", "supported_verbosity"]);
    if !explicit.is_empty() {
        return explicit;
    }
    if value
        .get("support_verbosity")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        return ["low", "medium", "high"]
            .into_iter()
            .map(ToOwned::to_owned)
            .collect();
    }
    Vec::new()
}

fn string_list(value: Option<&Value>, object_keys: &[&str]) -> Option<Vec<String>> {
    match value? {
        Value::Array(values) => Some(
            values
                .iter()
                .filter_map(|value| string_scalar(value, object_keys))
                .collect(),
        ),
        Value::Object(_) | Value::String(_) => {
            Some(string_scalar(value?, object_keys).into_iter().collect())
        }
        _ => None,
    }
}

fn string_scalar(value: &Value, object_keys: &[&str]) -> Option<String> {
    match value {
        Value::String(value) => {
            let value = value.trim();
            if value.is_empty() {
                None
            } else {
                Some(value.to_string())
            }
        }
        Value::Object(object) => object_keys.iter().find_map(|key| {
            object
                .get(*key)
                .and_then(|value| string_scalar(value, object_keys))
        }),
        _ => None,
    }
}

fn validate_optional_capability(
    model: &CodexCatalogModel,
    field: &str,
    requested: Option<&str>,
    allowed: &[String],
) -> AppResult<()> {
    let Some(requested) = requested else {
        return Ok(());
    };
    if !allowed.iter().any(|value| value == requested) {
        return Err(AppError::ModelNotSupported(format!(
            "{} does not support {field}={requested}",
            model.slug
        )));
    }
    Ok(())
}

fn validate_modalities(
    model: &CodexCatalogModel,
    field: &str,
    requested: &[&str],
    allowed: &[String],
) -> AppResult<()> {
    if requested.is_empty() {
        return Ok(());
    }
    if let Some(unsupported) = requested
        .iter()
        .find(|requested| !allowed.iter().any(|value| value == **requested))
    {
        return Err(AppError::ModelNotSupported(format!(
            "{} does not support {field}={unsupported}",
            model.slug
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    fn catalog_fixture() -> Value {
        json!({
            "models": [
                {
                    "slug": "gpt-5.5",
                    "visibility": "list",
                    "supported_in_api": true,
                    "reasoning_levels": ["low", "medium", "high", "xhigh"],
                    "service_tiers": ["auto", "priority"],
                    "verbosity": ["low", "medium", "high"],
                    "truncation_policy": ["auto", "disabled"],
                    "input_modalities": ["text", "image"],
                    "output_modalities": ["text"]
                },
                {
                    "slug": "codex-auto-review",
                    "visibility": "list",
                    "supported_in_api": true
                },
                {
                    "slug": "gpt-hidden",
                    "visibility": "hidden",
                    "supported_in_api": true
                },
                {
                    "slug": "gpt-disabled",
                    "visibility": "list",
                    "supported_in_api": false
                }
            ]
        })
    }

    #[test]
    fn config_uses_defaults_and_env_overrides() {
        let defaults =
            CodexCatalogConfig::from_env_vars(|_| Err(env::VarError::NotPresent)).unwrap();
        assert_eq!(defaults.client_version, DEFAULT_CODEX_CLIENT_VERSION);
        assert_eq!(
            defaults.ttl,
            Duration::from_secs(DEFAULT_CODEX_CATALOG_TTL_SECS)
        );

        let custom = CodexCatalogConfig::from_env_vars(|name| match name {
            "UMP_V2_CODEX_CLIENT_VERSION" => Ok(" 99.1.2 ".to_string()),
            "UMP_V2_CODEX_CATALOG_TTL_SECS" => Ok("42".to_string()),
            _ => Err(env::VarError::NotPresent),
        })
        .unwrap();
        assert_eq!(custom.client_version, "99.1.2");
        assert_eq!(custom.ttl, Duration::from_secs(42));
    }

    #[test]
    fn parse_validates_visibility_supported_flag_and_capabilities() {
        let catalog =
            CodexCatalog::parse(DEFAULT_CODEX_CLIENT_VERSION, &catalog_fixture()).unwrap();

        let model = catalog
            .validate_request(CodexCatalogRequest {
                model: "gpt-5.5",
                reasoning_effort: Some("high"),
                service_tier: Some("priority"),
                verbosity: Some("medium"),
                truncation: Some("auto"),
                input_modalities: &["text", "image"],
                output_modalities: &["text"],
                ..CodexCatalogRequest::default()
            })
            .unwrap();
        assert_eq!(model.slug, "gpt-5.5");

        assert!(catalog.validate_model("codex-auto-review", false).is_err());
        assert!(catalog.validate_model("gpt-hidden", false).is_err());
        assert!(catalog.validate_model("gpt-disabled", true).is_err());
        assert!(catalog.validate_model("missing", false).is_err());
        assert!(catalog.validate_model("codex-auto-review", true).is_ok());
    }

    #[test]
    fn openai_projection_omits_hidden_and_unsupported_models_by_default() {
        let catalog =
            CodexCatalog::parse(DEFAULT_CODEX_CLIENT_VERSION, &catalog_fixture()).unwrap();
        let public = catalog.to_openai_models(false);
        let ids = public["data"]
            .as_array()
            .unwrap()
            .iter()
            .map(|model| model["id"].as_str().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(ids, vec!["gpt-5.5"]);

        let internal = catalog.to_openai_models(true);
        let ids = internal["data"]
            .as_array()
            .unwrap()
            .iter()
            .map(|model| model["id"].as_str().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(ids, vec!["codex-auto-review", "gpt-5.5", "gpt-hidden"]);
    }

    #[test]
    fn request_validation_rejects_unsupported_catalog_dimensions() {
        let catalog =
            CodexCatalog::parse(DEFAULT_CODEX_CLIENT_VERSION, &catalog_fixture()).unwrap();

        for request in [
            CodexCatalogRequest {
                model: "gpt-5.5",
                reasoning_effort: Some("max"),
                ..CodexCatalogRequest::default()
            },
            CodexCatalogRequest {
                model: "gpt-5.5",
                service_tier: Some("scale"),
                ..CodexCatalogRequest::default()
            },
            CodexCatalogRequest {
                model: "gpt-5.5",
                verbosity: Some("silent"),
                ..CodexCatalogRequest::default()
            },
            CodexCatalogRequest {
                model: "gpt-5.5",
                truncation: Some("none"),
                ..CodexCatalogRequest::default()
            },
            CodexCatalogRequest {
                model: "gpt-5.5",
                input_modalities: &["audio"],
                ..CodexCatalogRequest::default()
            },
            CodexCatalogRequest {
                model: "gpt-5.5",
                output_modalities: &["audio"],
                ..CodexCatalogRequest::default()
            },
        ] {
            assert!(catalog.validate_request(request).is_err());
        }
    }

    #[test]
    fn endpoint_requires_client_version_and_preserves_existing_query() {
        assert_eq!(
            codex_models_endpoint(CODEX_MODELS_URL, DEFAULT_CODEX_CLIENT_VERSION).unwrap(),
            "https://chatgpt.com/backend-api/codex/models?client_version=26.506.31421"
        );
        assert_eq!(
            codex_models_endpoint("https://example.test/models?foo=bar", "1.2.3").unwrap(),
            "https://example.test/models?foo=bar&client_version=1.2.3"
        );
        assert!(codex_models_endpoint(CODEX_MODELS_URL, "  ").is_err());
    }

    #[tokio::test]
    async fn cache_reuses_fresh_validated_catalog_until_ttl_expires() {
        let cache = CodexCatalogCache::new(CodexCatalogConfig {
            client_version: DEFAULT_CODEX_CLIENT_VERSION.to_string(),
            ttl: Duration::from_secs(60),
        });
        let calls = AtomicUsize::new(0);

        let first = cache
            .get_or_refresh_with(|_| async {
                calls.fetch_add(1, Ordering::SeqCst);
                Ok(catalog_fixture())
            })
            .await
            .unwrap();
        let second = cache
            .get_or_refresh_with(|_| async {
                calls.fetch_add(1, Ordering::SeqCst);
                Err(AppError::Upstream("should not refresh fresh cache".into()))
            })
            .await
            .unwrap();

        assert_eq!(first, second);
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn expired_cache_refreshes_or_fails_closed() {
        let cache = CodexCatalogCache::new(CodexCatalogConfig {
            client_version: DEFAULT_CODEX_CLIENT_VERSION.to_string(),
            ttl: Duration::ZERO,
        });
        cache.store_validated(&catalog_fixture()).unwrap();

        let failed = cache
            .get_or_refresh_with(|_| async {
                Err(AppError::Upstream("catalog unavailable".into()))
            })
            .await
            .unwrap_err();
        assert!(failed.to_string().contains("catalog unavailable"));

        let refreshed = cache
            .get_or_refresh_with(|_| async {
                Ok(json!({
                    "models": [{
                        "slug": "gpt-5.6",
                        "visibility": "list",
                        "supported_in_api": true
                    }]
                }))
            })
            .await
            .unwrap();
        assert!(refreshed.model("gpt-5.6").is_some());
        assert!(refreshed.model("gpt-5.5").is_none());
    }
}
