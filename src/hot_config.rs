use std::{
    env, fs,
    path::{Path, PathBuf},
    sync::Arc,
};

use serde::Deserialize;
use serde_json::{json, Value};

use crate::{
    model_alias::{Provider, ResolvedModel},
    AppError, AppResult,
};

#[derive(Clone, Debug)]
pub struct HotRoutingConfig {
    path: Option<Arc<PathBuf>>,
}

#[derive(Debug, Deserialize)]
struct RoutingConfigFile {
    #[serde(default)]
    routes: Vec<ConfiguredRoute>,
}

#[derive(Debug, Deserialize)]
struct ConfiguredRoute {
    source: ConfiguredSource,
    target: ConfiguredTarget,
    #[serde(default = "default_enabled")]
    enabled: bool,
}

#[derive(Debug, Deserialize)]
struct ConfiguredSource {
    model: String,
    #[serde(default, rename = "format")]
    format: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ConfiguredTarget {
    provider: Provider,
    model: String,
    #[serde(default, rename = "format")]
    format: Option<String>,
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
        let Some(config) = self.load()? else {
            return Ok(None);
        };
        Ok(config
            .routes
            .into_iter()
            .find(|route| {
                route.enabled
                    && route.source.model == model
                    && source_format_matches(route.source.format.as_deref(), source_format)
            })
            .map(|route| ResolvedModel {
                provider: route.target.provider,
                upstream_model: route.target.model,
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
        let config = serde_json::from_str(&raw)
            .map_err(|error| AppError::BadRequest(format!("invalid routing config: {error}")))?;
        validate_config_file(&config)?;
        Ok(Some(config))
    }
}

fn default_enabled() -> bool {
    true
}

fn validate_config_value(value: &Value) -> AppResult<()> {
    let config = serde_json::from_value::<RoutingConfigFile>(value.clone())
        .map_err(|error| AppError::BadRequest(format!("invalid routing config: {error}")))?;
    validate_config_file(&config)
}

fn validate_config_file(config: &RoutingConfigFile) -> AppResult<()> {
    for route in &config.routes {
        validate_format(route.source.format.as_deref())?;
        validate_format(route.target.format.as_deref())?;
    }
    Ok(())
}

fn source_format_matches(route_format: Option<&str>, request_format: Option<&str>) -> bool {
    match route_format {
        Some(route_format) => Some(route_format) == request_format,
        None => true,
    }
}

fn validate_format(format: Option<&str>) -> AppResult<()> {
    let Some(format) = format else {
        return Ok(());
    };
    match format {
        "responses"
        | "anthropic_messages"
        | "chat_completions"
        | "google_generate_content"
        | "openai_images" => Ok(()),
        other => Err(AppError::BadRequest(format!(
            "invalid routing config: unsupported format {other}"
        ))),
    }
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
            .contains("unsupported format made_up"));
    }
}
