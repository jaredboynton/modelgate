use std::{
    env, fs,
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
    time::SystemTime,
};

use serde::Deserialize;
use serde_json::{json, Value};

use crate::{
    compaction::{load_pack_keys_from_env, RemoteCompactionPolicy},
    model_alias::{
        default_remote_compaction_policy, Provider, ResolvedModel, ResolvedTarget, SourceFormat,
        TargetFormat,
    },
    AppError, AppResult,
};

#[derive(Clone, Debug)]
pub struct HotRoutingConfig {
    path: Option<Arc<PathBuf>>,
    cache: Arc<RwLock<Option<CachedRoutingConfig>>>,
}

#[derive(Clone, Debug)]
struct CachedRoutingConfig {
    metadata: ConfigMetadata,
    config: Option<Arc<RoutingConfigFile>>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ConfigMetadata {
    exists: bool,
    modified: Option<SystemTime>,
    len: u64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RoutingConfigFile {
    #[serde(default)]
    pub(crate) routes: Vec<ConfiguredRoute>,
    #[serde(default)]
    pub(crate) compaction: Option<CompactionConfigFile>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ConfiguredRoute {
    pub(crate) source: ConfiguredSource,
    pub(crate) target: ConfiguredTarget,
    #[serde(default = "default_enabled")]
    pub(crate) enabled: bool,
    #[serde(default)]
    pub(crate) remote_compaction_policy: Option<RemoteCompactionPolicy>,
    #[serde(default)]
    pub(crate) compaction: Option<RouteCompactionConfigFile>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CompactionConfigFile {
    #[serde(default)]
    pub(crate) default_policy: Option<RemoteCompactionPolicy>,
    #[serde(default)]
    pub(crate) allow_lossy_compaction_drop: Option<bool>,
    #[serde(default)]
    pub(crate) keys_env: Option<String>,
    #[serde(default)]
    pub(crate) summarizer_model_env: Option<String>,
    #[serde(default)]
    pub(crate) privacy_policy: Option<CompactionPrivacyPolicy>,
    #[serde(default)]
    pub(crate) allow_route_privacy_relaxation: Option<bool>,
    #[serde(default)]
    pub(crate) cross_provider_allowlist: Vec<Value>,
    #[serde(default)]
    pub(crate) max_encrypted_content_bytes: Option<usize>,
    #[serde(default)]
    pub(crate) max_decrypted_pack_bytes: Option<usize>,
    #[serde(default)]
    pub(crate) max_source_items: Option<usize>,
    #[serde(default)]
    pub(crate) max_rendered_tokens: Option<usize>,
    #[serde(default)]
    pub(crate) max_compactor_input_bytes: Option<usize>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RouteCompactionConfigFile {
    #[serde(default)]
    pub(crate) privacy_policy: Option<CompactionPrivacyPolicy>,
    #[serde(default)]
    pub(crate) cross_provider_allowlist: Vec<Value>,
    #[serde(default)]
    pub(crate) max_encrypted_content_bytes: Option<usize>,
    #[serde(default)]
    pub(crate) max_decrypted_pack_bytes: Option<usize>,
    #[serde(default)]
    pub(crate) max_source_items: Option<usize>,
    #[serde(default)]
    pub(crate) max_rendered_tokens: Option<usize>,
    #[serde(default)]
    pub(crate) max_compactor_input_bytes: Option<usize>,
}

#[derive(Debug, Clone, Copy, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum CompactionPrivacyPolicy {
    SameProviderOnly,
    ExplicitCrossProvider,
    Off,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ConfiguredSource {
    pub(crate) model: String,
    #[serde(default, rename = "format")]
    pub(crate) format: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ConfiguredTarget {
    pub(crate) provider: Provider,
    pub(crate) model: String,
    #[serde(default, rename = "format")]
    pub(crate) format: Option<String>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ConfiguredModel {
    pub id: String,
    pub provider: Provider,
    pub remote_compaction_policy: RemoteCompactionPolicy,
}

impl HotRoutingConfig {
    pub fn from_env(auth_home: &Path) -> Self {
        let path = env::var_os("UMP_V2_CONFIG")
            .map(PathBuf::from)
            .filter(|path| !path.as_os_str().is_empty())
            .unwrap_or_else(|| auth_home.join("config.json"));
        Self::from_path(path)
    }

    pub fn from_path(path: PathBuf) -> Self {
        Self {
            path: Some(Arc::new(path)),
            cache: Arc::new(RwLock::new(None)),
        }
    }

    pub fn disabled() -> Self {
        Self {
            path: None,
            cache: Arc::new(RwLock::new(None)),
        }
    }

    pub fn resolve_model(&self, model: &str) -> AppResult<Option<ResolvedModel>> {
        self.resolve_model_for_format(model, None)
    }

    pub fn resolve_model_for_format(
        &self,
        model: &str,
        source_format: Option<&str>,
    ) -> AppResult<Option<ResolvedModel>> {
        Ok(self
            .resolve_target_for_format(model, source_format)?
            .map(ResolvedModel::from))
    }

    pub fn resolve_target_for_format(
        &self,
        model: &str,
        source_format: Option<&str>,
    ) -> AppResult<Option<ResolvedTarget>> {
        let Some(config) = self.load()? else {
            return Ok(None);
        };
        let Some(route) = config.routes.iter().find(|route| {
            route.enabled
                && route.source.model == model
                && source_format_matches(route.source.format.as_deref(), source_format)
        }) else {
            return Ok(None);
        };
        let target_format = match route.target.format.as_deref().and_then(parse_target_format) {
            Some(target_format) => target_format,
            None => route
                .target
                .provider
                .default_target_format()
                .ok_or_else(|| AppError::ModelNotSupported(model.to_string()))?,
        };
        Ok(Some(ResolvedTarget {
            provider: route.target.provider,
            upstream_model: route.target.model.clone(),
            target_format,
        }))
    }

    pub fn remote_compaction_policy_for_format(
        &self,
        model: &str,
        source_format: Option<&str>,
        target: &ResolvedTarget,
    ) -> AppResult<RemoteCompactionPolicy> {
        let Some(config) = self.load()? else {
            return Ok(target.default_remote_compaction_policy());
        };
        if let Some(route) = config.routes.iter().find(|route| {
            route.enabled
                && route.source.model == model
                && source_format_matches(route.source.format.as_deref(), source_format)
        }) {
            if let Some(policy) = route.remote_compaction_policy {
                return Ok(policy);
            }
        }
        Ok(config
            .compaction
            .as_ref()
            .and_then(|compaction| compaction.default_policy)
            .unwrap_or_else(|| {
                default_remote_compaction_policy(target.provider, target.target_format)
            }))
    }

    /// Resolve any hot-config target override and the effective remote compaction
    /// policy for `(model, source_format)` using a single snapshot load.
    ///
    /// Returns `(Some(target), policy)` when a matching enabled route exists.
    /// When no route matches, returns `(None, Some(global_policy))` if a global
    /// default is configured, otherwise `(None, None)` (caller should use the
    /// final static target's default policy).
    pub fn resolve_target_and_compaction_policy(
        &self,
        model: &str,
        source_format: Option<&str>,
    ) -> AppResult<(Option<ResolvedTarget>, Option<RemoteCompactionPolicy>)> {
        let Some(config) = self.load()? else {
            return Ok((None, None));
        };

        let matching = config.routes.iter().find(|route| {
            route.enabled
                && route.source.model == model
                && source_format_matches(route.source.format.as_deref(), source_format)
        });

        let target = matching.map(|route| {
            let target_format = route
                .target
                .format
                .as_deref()
                .and_then(parse_target_format)
                .or_else(|| route.target.provider.default_target_format())
                .ok_or_else(|| AppError::ModelNotSupported(model.to_string()))
                .unwrap_or(TargetFormat::Responses); // safe default; callers validate
            ResolvedTarget {
                provider: route.target.provider,
                upstream_model: route.target.model.clone(),
                target_format,
            }
        });

        let policy = if let Some(route) = matching {
            route
                .remote_compaction_policy
                .or_else(|| config.compaction.as_ref().and_then(|c| c.default_policy))
                .or_else(|| {
                    target
                        .as_ref()
                        .map(|t| default_remote_compaction_policy(t.provider, t.target_format))
                })
        } else {
            config.compaction.as_ref().and_then(|c| c.default_policy)
        };

        Ok((target, policy))
    }

    pub fn configured_models(&self) -> AppResult<Vec<ConfiguredModel>> {
        let Some(config) = self.load()? else {
            return Ok(Vec::new());
        };
        let default_policy = config
            .compaction
            .as_ref()
            .and_then(|compaction| compaction.default_policy);
        Ok(config
            .routes
            .iter()
            .filter(|route| route.enabled)
            .map(|route| {
                let target_format = route
                    .target
                    .format
                    .as_deref()
                    .and_then(parse_target_format)
                    .or_else(|| route.target.provider.default_target_format());
                let remote_compaction_policy = route
                    .remote_compaction_policy
                    .or(default_policy)
                    .or_else(|| {
                        target_format.map(|target_format| {
                            default_remote_compaction_policy(route.target.provider, target_format)
                        })
                    })
                    .unwrap_or(RemoteCompactionPolicy::Local);
                ConfiguredModel {
                    id: route.source.model.clone(),
                    provider: route.target.provider,
                    remote_compaction_policy,
                }
            })
            .collect())
    }

    pub fn read_json(&self) -> AppResult<Value> {
        let Some(path) = &self.path else {
            return Ok(json!({ "routes": [] }));
        };
        let raw = match fs::read_to_string(path.as_ref()) {
            Ok(raw) => raw,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(json!({ "routes": [] }));
            }
            Err(error) => return Err(AppError::Io(error)),
        };
        if raw.trim().is_empty() {
            return Ok(json!({ "routes": [] }));
        }
        let value: Value = serde_json::from_str(&raw)
            .map_err(|error| AppError::BadRequest(format!("invalid routing config: {error}")))?;
        validate_config_value(&value)?;
        Ok(value)
    }

    pub fn write_json(&self, value: &Value) -> AppResult<()> {
        validate_config_value(value)?;
        let Some(path) = &self.path else {
            return Err(AppError::BadRequest(
                "routing config file is not configured".into(),
            ));
        };
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let formatted = serde_json::to_string_pretty(value)?;
        let temp_path = path.with_extension("json.tmp");
        fs::write(&temp_path, format!("{formatted}\n"))?;
        fs::rename(temp_path, path.as_ref())?;
        Ok(())
    }

    fn load(&self) -> AppResult<Option<Arc<RoutingConfigFile>>> {
        let Some(path) = &self.path else {
            return Ok(None);
        };
        let metadata = config_metadata(path.as_ref())?;
        if let Some(cached) = self
            .cache
            .read()
            .expect("routing config cache poisoned")
            .as_ref()
            .filter(|cached| cached.metadata == metadata)
        {
            return Ok(cached.config.clone());
        }
        if !metadata.exists {
            *self.cache.write().expect("routing config cache poisoned") =
                Some(CachedRoutingConfig {
                    metadata,
                    config: None,
                });
            return Ok(None);
        }
        let raw = match fs::read_to_string(path.as_ref()) {
            Ok(raw) => raw,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(AppError::Io(error)),
        };
        if raw.trim().is_empty() {
            *self.cache.write().expect("routing config cache poisoned") =
                Some(CachedRoutingConfig {
                    metadata,
                    config: None,
                });
            return Ok(None);
        }
        let value: Value = serde_json::from_str(&raw)
            .map_err(|error| AppError::BadRequest(format!("invalid routing config: {error}")))?;
        let config = Arc::new(parse_config_value(value)?);
        *self.cache.write().expect("routing config cache poisoned") = Some(CachedRoutingConfig {
            metadata,
            config: Some(Arc::clone(&config)),
        });
        Ok(Some(config))
    }
}

fn config_metadata(path: &Path) -> AppResult<ConfigMetadata> {
    match fs::metadata(path) {
        Ok(metadata) => Ok(ConfigMetadata {
            exists: true,
            modified: metadata.modified().ok(),
            len: metadata.len(),
        }),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(ConfigMetadata {
            exists: false,
            modified: None,
            len: 0,
        }),
        Err(error) => Err(AppError::Io(error)),
    }
}

fn default_enabled() -> bool {
    true
}

pub(crate) fn parse_config_value(value: Value) -> AppResult<RoutingConfigFile> {
    validate_no_secret_keys(&value)?;
    let config = serde_json::from_value::<RoutingConfigFile>(value)
        .map_err(|error| AppError::BadRequest(format!("invalid routing config: {error}")))?;
    validate_config_file(&config)?;
    Ok(config)
}

fn validate_config_value(value: &Value) -> AppResult<()> {
    parse_config_value(value.clone()).map(|_| ())
}

fn validate_config_file(config: &RoutingConfigFile) -> AppResult<()> {
    if let Some(compaction) = &config.compaction {
        touch_compaction_config(compaction);
    }
    validate_proxy_visible_prerequisites(config)?;
    for route in config.routes.iter().filter(|route| route.enabled) {
        validate_source_format(route.source.format.as_deref())?;
        validate_target_format(route.target.format.as_deref())?;
        if let Some(compaction) = &route.compaction {
            touch_route_compaction_config(compaction);
        }
        if route.target.provider == Provider::Unsupported {
            return Err(AppError::BadRequest(format!(
                "invalid routing config: unsupported target provider for model {}",
                route.target.model
            )));
        }
    }
    Ok(())
}

fn validate_proxy_visible_prerequisites(config: &RoutingConfigFile) -> AppResult<()> {
    let default_policy = config
        .compaction
        .as_ref()
        .and_then(|compaction| compaction.default_policy);
    if default_policy == Some(RemoteCompactionPolicy::ProxyVisibleSummary) {
        validate_proxy_visible_runtime("compaction.default_policy")?;
        validate_proxy_visible_privacy(
            config
                .compaction
                .as_ref()
                .and_then(|compaction| compaction.privacy_policy),
            "compaction.privacy_policy",
        )?;
    }
    for route in config.routes.iter().filter(|route| route.enabled) {
        let effective_policy = route.remote_compaction_policy.or(default_policy);
        if effective_policy != Some(RemoteCompactionPolicy::ProxyVisibleSummary) {
            continue;
        }
        validate_proxy_visible_runtime(&format!(
            "route {} remote_compaction_policy",
            route.source.model
        ))?;
        let effective_privacy = route
            .compaction
            .as_ref()
            .and_then(|compaction| compaction.privacy_policy)
            .or_else(|| {
                config
                    .compaction
                    .as_ref()
                    .and_then(|compaction| compaction.privacy_policy)
            });
        validate_proxy_visible_privacy(
            effective_privacy,
            &format!("route {} compaction.privacy_policy", route.source.model),
        )?;
    }
    Ok(())
}

fn validate_proxy_visible_runtime(field: &str) -> AppResult<()> {
    load_pack_keys_from_env().map_err(|error| {
        AppError::BadRequest(format!(
            "invalid routing config: {field} requires UMP_COMPACTION_KEYS_JSON: {error}"
        ))
    })?;
    let has_instance_id = env::var_os("UMP_COMPACTION_INSTANCE_ID")
        .and_then(|value| value.into_string().ok())
        .is_some_and(|value| !value.trim().is_empty());
    if !has_instance_id {
        return Err(AppError::BadRequest(format!(
            "invalid routing config: {field} requires UMP_COMPACTION_INSTANCE_ID"
        )));
    }
    Ok(())
}

fn validate_proxy_visible_privacy(
    privacy_policy: Option<CompactionPrivacyPolicy>,
    field: &str,
) -> AppResult<()> {
    if privacy_policy == Some(CompactionPrivacyPolicy::Off) {
        return Err(AppError::BadRequest(format!(
            "invalid routing config: {field} cannot be off when proxy_visible_summary is enabled"
        )));
    }
    Ok(())
}

fn touch_compaction_config(config: &CompactionConfigFile) {
    let _ = (
        config.allow_lossy_compaction_drop,
        config.keys_env.as_deref(),
        config.summarizer_model_env.as_deref(),
        config.privacy_policy,
        config.allow_route_privacy_relaxation,
        config.cross_provider_allowlist.len(),
        config.max_encrypted_content_bytes,
        config.max_decrypted_pack_bytes,
        config.max_source_items,
        config.max_rendered_tokens,
        config.max_compactor_input_bytes,
    );
}

fn touch_route_compaction_config(config: &RouteCompactionConfigFile) {
    let _ = (
        config.privacy_policy,
        config.cross_provider_allowlist.len(),
        config.max_encrypted_content_bytes,
        config.max_decrypted_pack_bytes,
        config.max_source_items,
        config.max_rendered_tokens,
        config.max_compactor_input_bytes,
    );
}

pub(crate) fn source_format_matches(
    route_format: Option<&str>,
    request_format: Option<&str>,
) -> bool {
    match route_format {
        Some(route_format) => Some(route_format) == request_format,
        None => true,
    }
}

fn validate_source_format(format: Option<&str>) -> AppResult<()> {
    let Some(format) = format else {
        return Ok(());
    };
    if parse_source_format(format).is_some() {
        Ok(())
    } else {
        Err(AppError::BadRequest(format!(
            "invalid routing config: unsupported source format {format}"
        )))
    }
}

fn validate_target_format(format: Option<&str>) -> AppResult<()> {
    let Some(format) = format else {
        return Ok(());
    };
    if parse_target_format(format).is_some() {
        Ok(())
    } else {
        Err(AppError::BadRequest(format!(
            "invalid routing config: unsupported target format {format}"
        )))
    }
}

pub(crate) fn parse_source_format(format: &str) -> Option<SourceFormat> {
    match format {
        "responses" => Some(SourceFormat::Responses),
        "anthropic_messages" => Some(SourceFormat::AnthropicMessages),
        "chat_completions" => Some(SourceFormat::ChatCompletions),
        "google_generate_content" => Some(SourceFormat::GoogleGenerateContent),
        "openai_images" => Some(SourceFormat::OpenaiImages),
        _ => None,
    }
}

pub(crate) fn parse_target_format(format: &str) -> Option<TargetFormat> {
    match format {
        "responses" => Some(TargetFormat::Responses),
        "anthropic_messages" => Some(TargetFormat::AnthropicMessages),
        "google_generate_content" => Some(TargetFormat::GoogleGenerateContent),
        "openai_images" => Some(TargetFormat::OpenaiImages),
        "cursor_agent" => Some(TargetFormat::CursorAgent),
        _ => None,
    }
}

pub(crate) fn validate_no_secret_keys(value: &Value) -> AppResult<()> {
    validate_no_secret_keys_at(value, "$")
}

fn validate_no_secret_keys_at(value: &Value, path: &str) -> AppResult<()> {
    match value {
        Value::Object(object) => {
            for (key, child) in object {
                let key_path = format!("{path}.{key}");
                if is_forbidden_key(key) {
                    return Err(AppError::BadRequest(format!(
                        "invalid routing config: forbidden secret key at {key_path}"
                    )));
                }
                validate_no_secret_keys_at(child, &key_path)?;
            }
        }
        Value::Array(items) => {
            for (index, child) in items.iter().enumerate() {
                validate_no_secret_keys_at(child, &format!("{path}[{index}]"))?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn is_forbidden_key(key: &str) -> bool {
    let normalized: String = key
        .chars()
        .filter(|ch| !matches!(ch, '_' | '-' | '.' | ':') && !ch.is_ascii_whitespace())
        .flat_map(|ch| ch.to_lowercase())
        .collect();
    matches!(
        normalized.as_str(),
        "token"
            | "secret"
            | "apikey"
            | "authorization"
            | "password"
            | "credential"
            | "accesstoken"
            | "refreshtoken"
    )
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;

    use super::*;

    struct EnvRestore {
        key: &'static str,
        previous: Option<OsString>,
    }

    impl EnvRestore {
        fn clear(key: &'static str) -> Self {
            let previous = env::var_os(key);
            env::remove_var(key);
            Self { key, previous }
        }

        fn set(key: &'static str, value: &str) -> Self {
            let previous = env::var_os(key);
            env::set_var(key, value);
            Self { key, previous }
        }
    }

    impl Drop for EnvRestore {
        fn drop(&mut self) {
            match &self.previous {
                Some(value) => env::set_var(self.key, value),
                None => env::remove_var(self.key),
            }
        }
    }

    #[test]
    fn hot_config_reads_current_file_contents() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("config.json");
        let config = HotRoutingConfig::from_path(path.clone());

        fs::write(
            &path,
            serde_json::json!({
                "routes": [{
                    "source": { "model": "gemini-3.1-flash-lite", "format": "responses" },
                    "target": { "provider": "codex", "model": "gpt-5.5", "format": "responses" }
                }]
            })
            .to_string(),
        )
        .unwrap();

        let resolved = config
            .resolve_model_for_format("gemini-3.1-flash-lite", Some("responses"))
            .unwrap()
            .unwrap();
        assert_eq!(resolved.provider, Provider::Codex);
        assert_eq!(resolved.upstream_model, "gpt-5.5");

        fs::write(
            &path,
            serde_json::json!({
                "routes": [{
                    "source": { "model": "gemini-3.1-flash-lite" },
                    "target": { "provider": "google", "model": "gemini-3.1-flash-lite" }
                }]
            })
            .to_string(),
        )
        .unwrap();

        let resolved = config
            .resolve_model("gemini-3.1-flash-lite")
            .unwrap()
            .unwrap();
        assert_eq!(resolved.provider, Provider::Google);
        assert_eq!(resolved.upstream_model, "gemini-3.1-flash-lite");
    }

    #[test]
    fn hot_config_honors_source_format_when_present() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("config.json");
        fs::write(
            &path,
            serde_json::json!({
                "routes": [
                    {
                        "source": { "model": "same-model", "format": "responses" },
                        "target": { "provider": "google", "model": "gemini-3.1-flash-lite", "format": "google_generate_content" }
                    },
                    {
                        "source": { "model": "same-model", "format": "chat_completions" },
                        "target": { "provider": "codex", "model": "gpt-5.5", "format": "responses" }
                    }
                ]
            })
            .to_string(),
        )
        .unwrap();
        let config = HotRoutingConfig::from_path(path);

        let responses = config
            .resolve_model_for_format("same-model", Some("responses"))
            .unwrap()
            .unwrap();
        assert_eq!(responses.provider, Provider::Google);
        assert_eq!(responses.upstream_model, "gemini-3.1-flash-lite");

        let chat = config
            .resolve_model_for_format("same-model", Some("chat_completions"))
            .unwrap()
            .unwrap();
        assert_eq!(chat.provider, Provider::Codex);
        assert_eq!(chat.upstream_model, "gpt-5.5");

        assert!(config.resolve_model("same-model").unwrap().is_none());
    }

    #[test]
    fn hot_config_rejects_unknown_formats_on_read() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("config.json");
        fs::write(
            &path,
            serde_json::json!({
                "routes": [{
                    "source": { "model": "same-model", "format": "made_up" },
                    "target": { "provider": "codex", "model": "gpt-5.5", "format": "responses" }
                }]
            })
            .to_string(),
        )
        .unwrap();
        let config = HotRoutingConfig::from_path(path);

        assert!(config
            .resolve_model_for_format("same-model", Some("made_up"))
            .unwrap_err()
            .to_string()
            .contains("unsupported source format made_up"));
    }

    #[test]
    fn hot_config_exposes_explicit_target_format() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("config.json");
        fs::write(
            &path,
            serde_json::json!({
                "routes": [{
                    "source": { "model": "same-model", "format": "responses" },
                    "target": { "provider": "google", "model": "gemini-3.1-flash-lite", "format": "google_generate_content" }
                }]
            })
            .to_string(),
        )
        .unwrap();
        let config = HotRoutingConfig::from_path(path);

        let resolved = config
            .resolve_target_for_format("same-model", Some("responses"))
            .unwrap()
            .unwrap();
        assert_eq!(resolved.provider, Provider::Google);
        assert_eq!(resolved.upstream_model, "gemini-3.1-flash-lite");
        assert_eq!(resolved.target_format, TargetFormat::GoogleGenerateContent);
    }

    #[test]
    fn hot_config_defaults_target_format_by_provider() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("config.json");
        fs::write(
            &path,
            serde_json::json!({
                "routes": [
                    {
                        "source": { "model": "bedrock-model", "format": "responses" },
                        "target": { "provider": "bedrock", "model": "claude" }
                    },
                    {
                        "source": { "model": "codex-model", "format": "responses" },
                        "target": { "provider": "codex", "model": "gpt-5.5" }
                    },
                    {
                        "source": { "model": "google-model", "format": "responses" },
                        "target": { "provider": "google", "model": "gemini-3.1-flash-lite" }
                    }
                ]
            })
            .to_string(),
        )
        .unwrap();
        let config = HotRoutingConfig::from_path(path);

        let bedrock = config
            .resolve_target_for_format("bedrock-model", Some("responses"))
            .unwrap()
            .unwrap();
        assert_eq!(bedrock.target_format, TargetFormat::AnthropicMessages);

        let codex = config
            .resolve_target_for_format("codex-model", Some("responses"))
            .unwrap()
            .unwrap();
        assert_eq!(codex.target_format, TargetFormat::Responses);

        let google = config
            .resolve_target_for_format("google-model", Some("responses"))
            .unwrap()
            .unwrap();
        assert_eq!(google.target_format, TargetFormat::GoogleGenerateContent);
    }

    #[test]
    fn hot_config_rejects_chat_completions_as_target_format() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("config.json");
        fs::write(
            &path,
            serde_json::json!({
                "routes": [{
                    "source": { "model": "same-model", "format": "chat_completions" },
                    "target": { "provider": "codex", "model": "gpt-5.5", "format": "chat_completions" }
                }]
            })
            .to_string(),
        )
        .unwrap();
        let config = HotRoutingConfig::from_path(path);

        assert!(config
            .resolve_model_for_format("same-model", Some("chat_completions"))
            .unwrap_err()
            .to_string()
            .contains("unsupported target format chat_completions"));
    }

    #[test]
    fn hot_config_uses_first_enabled_duplicate_exact_source() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("config.json");
        fs::write(
            &path,
            serde_json::json!({
                "routes": [
                    {
                        "source": { "model": "same-model", "format": "responses" },
                        "target": { "provider": "codex", "model": "gpt-5.5" }
                    },
                    {
                        "source": { "model": "same-model", "format": "responses" },
                        "target": { "provider": "google", "model": "gemini-3.1-flash-lite" }
                    }
                ]
            })
            .to_string(),
        )
        .unwrap();
        let config = HotRoutingConfig::from_path(path);

        let resolved = config
            .resolve_model_for_format("same-model", Some("responses"))
            .unwrap()
            .unwrap();
        assert_eq!(resolved.provider, Provider::Codex);
        assert_eq!(resolved.upstream_model, "gpt-5.5");
    }

    #[test]
    fn hot_config_uses_first_enabled_wildcard_specific_overlap() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("config.json");
        fs::write(
            &path,
            serde_json::json!({
                "routes": [
                    {
                        "source": { "model": "same-model" },
                        "target": { "provider": "codex", "model": "gpt-5.5" }
                    },
                    {
                        "source": { "model": "same-model", "format": "responses" },
                        "target": { "provider": "google", "model": "gemini-3.1-flash-lite" }
                    }
                ]
            })
            .to_string(),
        )
        .unwrap();
        let config = HotRoutingConfig::from_path(path);

        let resolved = config
            .resolve_model_for_format("same-model", Some("responses"))
            .unwrap()
            .unwrap();
        assert_eq!(resolved.provider, Provider::Codex);
        assert_eq!(resolved.upstream_model, "gpt-5.5");
    }

    #[test]
    fn hot_config_ignores_disabled_duplicate_sources() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("config.json");
        fs::write(
            &path,
            serde_json::json!({
                "routes": [
                    {
                        "source": { "model": "same-model", "format": "responses" },
                        "target": { "provider": "codex", "model": "gpt-5.5" }
                    },
                    {
                        "enabled": false,
                        "source": { "model": "same-model", "format": "responses" },
                        "target": { "provider": "google", "model": "gemini-3.1-flash-lite" }
                    }
                ]
            })
            .to_string(),
        )
        .unwrap();
        let config = HotRoutingConfig::from_path(path);

        let resolved = config
            .resolve_model_for_format("same-model", Some("responses"))
            .unwrap()
            .unwrap();
        assert_eq!(resolved.provider, Provider::Codex);
        assert_eq!(resolved.upstream_model, "gpt-5.5");
    }

    #[test]
    fn hot_config_rejects_proxy_visible_without_runtime_keys() {
        let _guard = crate::TEST_ENV_LOCK.lock().unwrap();
        let _keys = EnvRestore::clear("UMP_COMPACTION_KEYS_JSON");
        let _instance = EnvRestore::clear("UMP_COMPACTION_INSTANCE_ID");
        let error = parse_config_value(serde_json::json!({
            "routes": [{
                "source": { "model": "compact", "format": "responses" },
                "target": {
                    "provider": "bedrock",
                    "model": "anthropic.claude-opus-4-7",
                    "format": "anthropic_messages"
                },
                "remote_compaction_policy": "proxy_visible_summary"
            }]
        }))
        .unwrap_err();

        assert!(error.to_string().contains("UMP_COMPACTION_KEYS_JSON"));
    }

    #[test]
    fn hot_config_rejects_proxy_visible_with_privacy_off() {
        let _guard = crate::TEST_ENV_LOCK.lock().unwrap();
        let _keys = EnvRestore::set(
            "UMP_COMPACTION_KEYS_JSON",
            r#"{"current":"fixture","keys":{"fixture":"AAECAwQFBgcICQoLDA0ODxAREhMUFRYXGBkaGxwdHh8"}}"#,
        );
        let _instance = EnvRestore::set("UMP_COMPACTION_INSTANCE_ID", "hot-config-test");
        let error = parse_config_value(serde_json::json!({
            "compaction": {
                "privacy_policy": "off"
            },
            "routes": [{
                "source": { "model": "compact", "format": "responses" },
                "target": {
                    "provider": "bedrock",
                    "model": "anthropic.claude-opus-4-7",
                    "format": "anthropic_messages"
                },
                "remote_compaction_policy": "proxy_visible_summary"
            }]
        }))
        .unwrap_err();

        assert!(error.to_string().contains("cannot be off"));
    }

    #[test]
    fn hot_config_disabled_proxy_visible_route_does_not_force_env_validation() {
        let _guard = crate::TEST_ENV_LOCK.lock().unwrap();
        let _keys = EnvRestore::clear("UMP_COMPACTION_KEYS_JSON");
        let _instance = EnvRestore::clear("UMP_COMPACTION_INSTANCE_ID");

        // A disabled route requesting proxy_visible_summary must not trip
        // env validation, because it cannot serve traffic.
        parse_config_value(serde_json::json!({
            "routes": [{
                "enabled": false,
                "source": { "model": "compact", "format": "responses" },
                "target": {
                    "provider": "bedrock",
                    "model": "anthropic.claude-opus-4-7",
                    "format": "anthropic_messages"
                },
                "remote_compaction_policy": "proxy_visible_summary"
            }]
        }))
        .unwrap();
    }
}
