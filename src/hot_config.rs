use std::{
    env, fs,
    path::{Path, PathBuf},
    sync::Arc,
};

use serde::Deserialize;
use serde_json::{json, Value};

use crate::{
    model_alias::{Provider, ResolvedModel, ResolvedTarget, SourceFormat, TargetFormat},
    AppError, AppResult,
};

#[derive(Clone, Debug)]
pub struct HotRoutingConfig {
    path: Option<Arc<PathBuf>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RoutingConfigFile {
    #[serde(default)]
    pub(crate) routes: Vec<ConfiguredRoute>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ConfiguredRoute {
    pub(crate) source: ConfiguredSource,
    pub(crate) target: ConfiguredTarget,
    #[serde(default = "default_enabled")]
    pub(crate) enabled: bool,
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
        }
    }

    pub fn disabled() -> Self {
        Self { path: None }
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
        let Some(route) = config.routes.into_iter().find(|route| {
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
            upstream_model: route.target.model,
            target_format,
        }))
    }

    pub fn configured_models(&self) -> AppResult<Vec<ConfiguredModel>> {
        let Some(config) = self.load()? else {
            return Ok(Vec::new());
        };
        Ok(config
            .routes
            .into_iter()
            .filter(|route| route.enabled)
            .map(|route| ConfiguredModel {
                id: route.source.model,
                provider: route.target.provider,
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

    fn load(&self) -> AppResult<Option<RoutingConfigFile>> {
        let Some(path) = &self.path else {
            return Ok(None);
        };
        let raw = match fs::read_to_string(path.as_ref()) {
            Ok(raw) => raw,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(AppError::Io(error)),
        };
        if raw.trim().is_empty() {
            return Ok(None);
        }
        let value: Value = serde_json::from_str(&raw)
            .map_err(|error| AppError::BadRequest(format!("invalid routing config: {error}")))?;
        let config = parse_config_value(value)?;
        Ok(Some(config))
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
    for route in &config.routes {
        validate_source_format(route.source.format.as_deref())?;
        validate_target_format(route.target.format.as_deref())?;
        if route.target.provider == Provider::Unsupported {
            return Err(AppError::BadRequest(format!(
                "invalid routing config: unsupported target provider for model {}",
                route.target.model
            )));
        }
    }
    Ok(())
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
    use super::*;

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
}
