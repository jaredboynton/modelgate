use std::{
    collections::HashMap,
    env,
    net::SocketAddr,
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, AtomicU32, Ordering},
        Arc, Mutex,
    },
    time::{Duration, SystemTime},
};

use crate::{
    amp_compat::AmpStore,
    codex_catalog::{CodexCatalogCache, CodexCatalogConfig},
    hot_config::HotRoutingConfig,
    model_alias::{resolve_model_required, ResolvedModel},
    AppResult,
};

const CODEX_RESPONSES_WSS_URL: &str = "wss://chatgpt.com/backend-api/codex/responses";
const CODEX_RESPONSES_HTTP_URL: &str = "https://chatgpt.com/backend-api/codex/responses";
const TEST_CODEX_RESPONSES_WSS_URL: &str = "ws://127.0.0.1:1/backend-api/codex/responses";
const TEST_CODEX_RESPONSES_HTTP_URL: &str = "http://127.0.0.1:1/backend-api/codex/responses";

#[derive(Clone)]
pub struct AppState {
    pub http: reqwest::Client,
    pub specter: specter::Client,
    pub amp_store: AmpStore,
    pub codex_home: PathBuf,
    pub auth_home: PathBuf,
    pub google_api_key: Option<Arc<str>>,
    pub bedrock_region: Arc<str>,
    pub runtime: RuntimeConfig,
    pub routing_config: HotRoutingConfig,
    pub codex_catalog: CodexCatalogCache,
    response_storage: ResponseStorage,
    codex_wss_latched: Arc<AtomicBool>,
    codex_wss_failures: Arc<AtomicU32>,
}

#[derive(Clone, Debug)]
pub struct RuntimeConfig {
    pub listen_addr: SocketAddr,
    pub codex_transport: CodexTransport,
    pub codex_responses_wss_url: String,
    pub codex_responses_http_url: String,
    pub codex_wss_connect_timeout: Duration,
    pub codex_max_concurrent: usize,
    pub codex_handshakes_per_min: u32,
    pub codex_catalog_client_version: String,
    pub codex_catalog_ttl: Duration,
    pub bedrock_discovery_timeout: Duration,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CodexTransport {
    Wss,
    Http,
    WssThenHttp,
}

#[derive(Clone, Debug)]
pub struct ResponseStateRecord {
    pub route: String,
    pub provider: String,
    pub upstream_model: String,
    pub upstream_response_id: String,
    pub adapter_response_id: String,
    pub conversation_id: Option<String>,
    pub raw_response: serde_json::Value,
    pub raw_input_items: serde_json::Value,
    pub upstream_codex_minted: bool,
    pub public_retrievable: bool,
    pub created_at: SystemTime,
    pub updated_at: SystemTime,
}

#[derive(Clone, Debug)]
pub struct NewResponseStateRecord {
    pub route: String,
    pub provider: String,
    pub upstream_model: String,
    pub upstream_response_id: String,
    pub adapter_response_id: String,
    pub conversation_id: Option<String>,
    pub raw_response: serde_json::Value,
    pub raw_input_items: serde_json::Value,
    pub upstream_codex_minted: bool,
}

#[derive(Clone)]
struct ResponseStorage {
    inner: Arc<Mutex<ResponseStorageInner>>,
    ttl: Duration,
}

#[derive(Default)]
struct ResponseStorageInner {
    volatile: HashMap<String, ResponseStateRecord>,
    public_retrievable: HashMap<String, ResponseStateRecord>,
}

impl Default for ResponseStorage {
    fn default() -> Self {
        Self {
            inner: Arc::new(Mutex::new(ResponseStorageInner::default())),
            ttl: Duration::from_secs(60 * 60),
        }
    }
}

impl AppState {
    pub fn from_env() -> Self {
        Self::from_env_with_config(
            RuntimeConfig::from_env().expect("invalid UMP v2 runtime config"),
        )
    }

    pub fn from_env_with_config(runtime: RuntimeConfig) -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let codex_home = env::var_os("UMP_V2_CODEX_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| home.join(".codex"));
        let auth_home = env::var_os("UMP_V2_AUTH_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| home.join(".ump"));
        let google_api_key = env::var("GOOGLE_API_KEY")
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .map(Arc::<str>::from);
        let bedrock_region = Arc::<str>::from(
            env::var("AWS_REGION")
                .or_else(|_| env::var("AWS_DEFAULT_REGION"))
                .unwrap_or_else(|_| "us-east-1".to_string()),
        );
        let routing_config = HotRoutingConfig::from_env(&auth_home);
        let codex_catalog = CodexCatalogCache::new(runtime.codex_catalog_config());

        Self {
            http: reqwest::Client::new(),
            specter: specter::Client::new().expect("failed to construct Specter client"),
            amp_store: AmpStore::from_env(),
            codex_home,
            auth_home,
            google_api_key,
            bedrock_region,
            runtime,
            routing_config,
            codex_catalog,
            response_storage: ResponseStorage::default(),
            codex_wss_latched: Arc::new(AtomicBool::new(false)),
            codex_wss_failures: Arc::new(AtomicU32::new(0)),
        }
    }

    pub fn for_tests(codex_home: PathBuf, auth_home: PathBuf) -> Self {
        assert_test_home(&codex_home);
        assert_test_home(&auth_home);
        let routing_config = HotRoutingConfig::from_path(auth_home.join("config.json"));
        let runtime = RuntimeConfig::for_tests();
        let codex_catalog = CodexCatalogCache::new(runtime.codex_catalog_config());

        Self {
            http: reqwest::Client::new(),
            specter: specter::Client::new().expect("failed to construct Specter client"),
            amp_store: AmpStore::new(auth_home.join("amp-threads")),
            codex_home,
            auth_home,
            google_api_key: None,
            bedrock_region: Arc::<str>::from("us-east-1"),
            runtime,
            routing_config,
            codex_catalog,
            response_storage: ResponseStorage::default(),
            codex_wss_latched: Arc::new(AtomicBool::new(false)),
            codex_wss_failures: Arc::new(AtomicU32::new(0)),
        }
    }

    pub fn resolve_model(&self, input: &str) -> AppResult<ResolvedModel> {
        if let Some(alias) = self.routing_config.resolve_model(input)? {
            return Ok(alias);
        }
        resolve_model_required(input)
    }

    pub fn resolve_model_for_format(
        &self,
        input: &str,
        source_format: &str,
    ) -> AppResult<ResolvedModel> {
        if let Some(alias) = self
            .routing_config
            .resolve_model_for_format(input, Some(source_format))?
        {
            return Ok(alias);
        }
        resolve_model_required(input)
    }

    pub fn codex_wss_latched(&self) -> bool {
        self.codex_wss_latched.load(Ordering::Relaxed)
    }

    pub fn record_codex_wss_failure(&self) -> bool {
        let failures = self.codex_wss_failures.fetch_add(1, Ordering::Relaxed) + 1;
        if failures >= 3 {
            self.codex_wss_latched.store(true, Ordering::Relaxed);
        }
        self.codex_wss_latched()
    }

    pub fn reset_codex_wss_latch(&self) {
        self.codex_wss_latched.store(false, Ordering::Relaxed);
        self.codex_wss_failures.store(0, Ordering::Relaxed);
    }

    pub fn remember_response_for_continuation(
        &self,
        record: NewResponseStateRecord,
    ) -> ResponseStateRecord {
        let record = response_state_record(record, false);
        let mut storage = self.response_storage.lock();
        storage
            .volatile
            .insert(record.adapter_response_id.clone(), record.clone());
        record
    }

    pub fn store_public_response(&self, record: NewResponseStateRecord) -> ResponseStateRecord {
        let record = response_state_record(record, true);
        let mut storage = self.response_storage.lock();
        storage
            .volatile
            .insert(record.adapter_response_id.clone(), record.clone());
        storage
            .public_retrievable
            .insert(record.adapter_response_id.clone(), record.clone());
        record
    }

    pub fn continuation_response(&self, adapter_response_id: &str) -> Option<ResponseStateRecord> {
        self.response_storage
            .lock()
            .volatile
            .get(adapter_response_id)
            .filter(|record| !self.response_storage.is_expired(record))
            .cloned()
    }

    pub fn public_response(&self, adapter_response_id: &str) -> Option<serde_json::Value> {
        self.response_storage
            .lock()
            .public_retrievable
            .get(adapter_response_id)
            .filter(|record| !self.response_storage.is_expired(record))
            .map(|record| record.raw_response.clone())
    }

    pub fn public_input_items(&self, adapter_response_id: &str) -> Option<serde_json::Value> {
        self.response_storage
            .lock()
            .public_retrievable
            .get(adapter_response_id)
            .filter(|record| !self.response_storage.is_expired(record))
            .map(|record| record.raw_input_items.clone())
    }

    pub fn cleanup_expired_responses(&self) {
        let mut storage = self.response_storage.lock();
        storage
            .volatile
            .retain(|_, record| !self.response_storage.is_expired(record));
        storage
            .public_retrievable
            .retain(|_, record| !self.response_storage.is_expired(record));
    }
}

impl ResponseStorage {
    fn lock(&self) -> std::sync::MutexGuard<'_, ResponseStorageInner> {
        self.inner.lock().expect("response storage mutex poisoned")
    }

    fn is_expired(&self, record: &ResponseStateRecord) -> bool {
        record
            .updated_at
            .elapsed()
            .map(|elapsed| elapsed > self.ttl)
            .unwrap_or(false)
    }
}

fn response_state_record(
    record: NewResponseStateRecord,
    public_retrievable: bool,
) -> ResponseStateRecord {
    let now = SystemTime::now();
    ResponseStateRecord {
        route: record.route,
        provider: record.provider,
        upstream_model: record.upstream_model,
        upstream_response_id: record.upstream_response_id,
        adapter_response_id: record.adapter_response_id,
        conversation_id: record.conversation_id,
        raw_response: record.raw_response,
        raw_input_items: record.raw_input_items,
        upstream_codex_minted: record.upstream_codex_minted,
        public_retrievable,
        created_at: now,
        updated_at: now,
    }
}

impl RuntimeConfig {
    pub fn from_env() -> Result<Self, String> {
        Self::from_env_vars(|name| env::var(name))
    }

    fn from_env_vars<F>(mut var: F) -> Result<Self, String>
    where
        F: FnMut(&str) -> Result<String, env::VarError>,
    {
        let codex_catalog_config = CodexCatalogConfig::from_env_vars(&mut var)?;
        Ok(Self {
            listen_addr: var("UMP_V2_LISTEN_ADDR")
                .unwrap_or_else(|_| "127.0.0.1:18743".to_string())
                .parse()
                .map_err(|err| format!("invalid UMP_V2_LISTEN_ADDR: {err}"))?,
            codex_transport: CodexTransport::from_value(
                var("UMP_V2_CODEX_TRANSPORT")
                    .unwrap_or_else(|_| "wss-then-http".to_string())
                    .as_str(),
            )?,
            codex_responses_wss_url: var("UMP_V2_CODEX_RESPONSES_WSS_URL")
                .unwrap_or_else(|_| CODEX_RESPONSES_WSS_URL.to_string()),
            codex_responses_http_url: var("UMP_V2_CODEX_RESPONSES_HTTP_URL")
                .unwrap_or_else(|_| CODEX_RESPONSES_HTTP_URL.to_string()),
            codex_wss_connect_timeout: duration_ms_env(
                &mut var,
                "UMP_V2_CODEX_WSS_CONNECT_TIMEOUT_MS",
                5000,
            )?,
            codex_max_concurrent: usize_env(&mut var, "UMP_V2_CODEX_MAX_CONCURRENT", 20)?,
            codex_handshakes_per_min: u32_env(&mut var, "UMP_V2_CODEX_HANDSHAKES_PER_MIN", 55)?,
            codex_catalog_client_version: codex_catalog_config.client_version,
            codex_catalog_ttl: codex_catalog_config.ttl,
            bedrock_discovery_timeout: duration_ms_env(
                &mut var,
                "UMP_V2_BEDROCK_DISCOVERY_TIMEOUT_MS",
                5000,
            )?,
        })
    }

    pub fn for_tests() -> Self {
        Self {
            codex_responses_wss_url: TEST_CODEX_RESPONSES_WSS_URL.to_string(),
            codex_responses_http_url: TEST_CODEX_RESPONSES_HTTP_URL.to_string(),
            ..Self::default()
        }
    }

    pub fn codex_catalog_config(&self) -> CodexCatalogConfig {
        CodexCatalogConfig {
            client_version: self.codex_catalog_client_version.clone(),
            ttl: self.codex_catalog_ttl,
        }
    }
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            listen_addr: "127.0.0.1:18743".parse().unwrap(),
            codex_transport: CodexTransport::WssThenHttp,
            codex_responses_wss_url: CODEX_RESPONSES_WSS_URL.to_string(),
            codex_responses_http_url: CODEX_RESPONSES_HTTP_URL.to_string(),
            codex_wss_connect_timeout: Duration::from_millis(5000),
            codex_max_concurrent: 20,
            codex_handshakes_per_min: 55,
            codex_catalog_client_version: CodexCatalogConfig::default().client_version,
            codex_catalog_ttl: CodexCatalogConfig::default().ttl,
            bedrock_discovery_timeout: Duration::from_millis(5000),
        }
    }
}

impl CodexTransport {
    fn from_value(value: &str) -> Result<Self, String> {
        match value {
            "wss" => Ok(Self::Wss),
            "http" => Ok(Self::Http),
            "wss-then-http" => Ok(Self::WssThenHttp),
            other => Err(format!("invalid UMP_V2_CODEX_TRANSPORT: {other}")),
        }
    }
}

fn duration_ms_env<F>(var: &mut F, name: &str, default_ms: u64) -> Result<Duration, String>
where
    F: FnMut(&str) -> Result<String, env::VarError>,
{
    let value = match var(name) {
        Ok(value) => value,
        Err(_) => return Ok(Duration::from_millis(default_ms)),
    };
    let millis = value
        .parse::<u64>()
        .map_err(|err| format!("invalid {name}: {err}"))?;
    Ok(Duration::from_millis(millis))
}

fn usize_env<F>(var: &mut F, name: &str, default_value: usize) -> Result<usize, String>
where
    F: FnMut(&str) -> Result<String, env::VarError>,
{
    match var(name) {
        Ok(value) => value
            .parse()
            .map_err(|err| format!("invalid {name}: {err}")),
        Err(_) => Ok(default_value),
    }
}

fn u32_env<F>(var: &mut F, name: &str, default_value: u32) -> Result<u32, String>
where
    F: FnMut(&str) -> Result<String, env::VarError>,
{
    match var(name) {
        Ok(value) => value
            .parse()
            .map_err(|err| format!("invalid {name}: {err}")),
        Err(_) => Ok(default_value),
    }
}

fn assert_test_home(path: &std::path::Path) {
    let temp_root = env::temp_dir();
    assert!(
        path.starts_with(&temp_root),
        "test home must live under temp dir: {}",
        path.display()
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn config_from(values: &[(&str, &str)]) -> Result<RuntimeConfig, String> {
        let values = values
            .iter()
            .map(|(key, value)| ((*key).to_string(), (*value).to_string()))
            .collect::<HashMap<_, _>>();
        RuntimeConfig::from_env_vars(|name| {
            values.get(name).cloned().ok_or(env::VarError::NotPresent)
        })
    }

    #[test]
    fn runtime_config_uses_documented_defaults() {
        let config = config_from(&[]).unwrap();

        assert_eq!(config.listen_addr, "127.0.0.1:18743".parse().unwrap());
        assert_eq!(config.codex_transport, CodexTransport::WssThenHttp);
        assert_eq!(
            config.codex_responses_wss_url,
            "wss://chatgpt.com/backend-api/codex/responses"
        );
        assert_eq!(
            config.codex_responses_http_url,
            "https://chatgpt.com/backend-api/codex/responses"
        );
        assert_eq!(
            config.codex_wss_connect_timeout,
            Duration::from_millis(5000)
        );
        assert_eq!(config.codex_max_concurrent, 20);
        assert_eq!(config.codex_handshakes_per_min, 55);
        assert_eq!(
            config.bedrock_discovery_timeout,
            Duration::from_millis(5000)
        );
    }

    #[test]
    fn runtime_config_parses_overrides() {
        let config = config_from(&[
            ("UMP_V2_LISTEN_ADDR", "127.0.0.1:19000"),
            ("UMP_V2_CODEX_TRANSPORT", "http"),
            ("UMP_V2_CODEX_RESPONSES_WSS_URL", "ws://127.0.0.1:1/ws"),
            ("UMP_V2_CODEX_RESPONSES_HTTP_URL", "http://127.0.0.1:1/http"),
            ("UMP_V2_CODEX_WSS_CONNECT_TIMEOUT_MS", "250"),
            ("UMP_V2_CODEX_MAX_CONCURRENT", "7"),
            ("UMP_V2_CODEX_HANDSHAKES_PER_MIN", "9"),
            ("UMP_V2_BEDROCK_DISCOVERY_TIMEOUT_MS", "125"),
        ])
        .unwrap();

        assert_eq!(config.listen_addr, "127.0.0.1:19000".parse().unwrap());
        assert_eq!(config.codex_transport, CodexTransport::Http);
        assert_eq!(config.codex_responses_wss_url, "ws://127.0.0.1:1/ws");
        assert_eq!(config.codex_responses_http_url, "http://127.0.0.1:1/http");
        assert_eq!(config.codex_wss_connect_timeout, Duration::from_millis(250));
        assert_eq!(config.codex_max_concurrent, 7);
        assert_eq!(config.codex_handshakes_per_min, 9);
        assert_eq!(config.bedrock_discovery_timeout, Duration::from_millis(125));
    }

    #[test]
    fn runtime_config_rejects_invalid_values() {
        assert!(config_from(&[("UMP_V2_LISTEN_ADDR", "not an address")]).is_err());
        assert!(config_from(&[("UMP_V2_CODEX_TRANSPORT", "ftp")]).is_err());
        assert!(config_from(&[("UMP_V2_CODEX_MAX_CONCURRENT", "nope")]).is_err());
    }

    #[test]
    fn for_tests_uses_loopback_codex_endpoint_defaults() {
        let temp = tempfile::tempdir().unwrap();
        let state = AppState::for_tests(temp.path().join("codex"), temp.path().join("ump"));

        assert_eq!(
            state.runtime.codex_responses_wss_url,
            "ws://127.0.0.1:1/backend-api/codex/responses"
        );
        assert_eq!(
            state.runtime.codex_responses_http_url,
            "http://127.0.0.1:1/backend-api/codex/responses"
        );
    }

    #[test]
    fn codex_latch_records_failures_and_resets() {
        let temp = tempfile::tempdir().unwrap();
        let state = AppState::for_tests(temp.path().join("codex"), temp.path().join("ump"));

        assert!(!state.codex_wss_latched());
        assert!(!state.record_codex_wss_failure());
        assert!(!state.record_codex_wss_failure());
        assert!(state.record_codex_wss_failure());

        state.reset_codex_wss_latch();
        assert!(!state.codex_wss_latched());
    }
}
