use std::{
    collections::{HashMap, VecDeque},
    env,
    net::SocketAddr,
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, AtomicU32, Ordering},
        Arc, Mutex,
    },
    time::{Duration, SystemTime},
};
use tokio::sync::OwnedSemaphorePermit;

use crate::{
    amp_compat::AmpStore,
    auth::{
        bedrock::BedrockAuth,
        codex::CodexAuth,
        cursor::CursorCredentials,
        file_cache::{AuthFileCache, ResolvedAuthCache},
    },
    codex_catalog::{CodexCatalogCache, CodexCatalogConfig, CODEX_MODELS_URL},
    hot_config::HotRoutingConfig,
    model_alias::{resolve_target_required, ResolvedModel, ResolvedTarget},
    upstream::cursor::session::CursorSessionStore,
    AppError, AppResult,
};

const CODEX_RESPONSES_WSS_URL: &str = "wss://chatgpt.com/backend-api/codex/responses";
const CODEX_RESPONSES_HTTP_URL: &str = "https://chatgpt.com/backend-api/codex/responses";
const GOOGLE_GENERATE_BASE_URL: &str = "https://generativelanguage.googleapis.com";
const WINDSURF_CLOUD_BASE_URL: &str = "https://server.codeium.com";
const TEST_CODEX_RESPONSES_WSS_URL: &str = "ws://127.0.0.1:1/backend-api/codex/responses";
const TEST_CODEX_RESPONSES_HTTP_URL: &str = "http://127.0.0.1:1/backend-api/codex/responses";
const DEFAULT_BEDROCK_REGION: &str = "us-west-2";
const DEFAULT_CODEX_WSS_POOL_SIZE: usize = 4;
const RESPONSE_STORAGE_MAX_ENTRIES: usize = 4096;
const DEFAULT_SPECTER_DNS_CACHE_TTL_MS: u64 = 300_000;
const DEFAULT_SPECTER_MAX_PENDING_PER_ORIGIN: usize = 20;
const DEFAULT_SPECTER_STREAM_BODY_BUFFER_SLOTS: usize = 32;
const DEFAULT_SPECTER_H3_TUNNEL_BYTE_BUDGET: usize = 262_144;
pub type CodexHandshakePermit = Option<OwnedSemaphorePermit>;

#[derive(Clone)]
pub struct AppState {
    pub specter: specter::Client,
    pub amp_store: AmpStore,
    pub codex_home: Arc<PathBuf>,
    pub auth_home: Arc<PathBuf>,
    pub google_api_key: Option<Arc<str>>,
    pub bedrock_region: Arc<str>,
    pub runtime: Arc<RuntimeConfig>,
    pub routing_config: HotRoutingConfig,
    pub codex_catalog: CodexCatalogCache,
    pub cursor_sessions: Arc<CursorSessionStore>,
    pub auth_files: AuthFileCache,
    pub(crate) codex_auth: ResolvedAuthCache<CodexAuth>,
    pub cursor_auth: Arc<tokio::sync::Mutex<Option<CursorCredentials>>>,
    pub(crate) google_auth: ResolvedAuthCache<String>,
    pub(crate) windsurf_auth: ResolvedAuthCache<String>,
    pub(crate) bedrock_auth: ResolvedAuthCache<BedrockAuth>,
    codex_handshake_sem: Option<Arc<tokio::sync::Semaphore>>,
    codex_handshake_times: Arc<std::sync::Mutex<std::collections::VecDeque<std::time::Instant>>>,
    codex_ws_pool: Arc<tokio::sync::Mutex<VecDeque<CodexWsPoolEntry>>>,
    response_storage: ResponseStorage,
    codex_wss_latched: Arc<AtomicBool>,
    codex_wss_failures: Arc<AtomicU32>,
}

struct CodexWsPoolEntry {
    key: String,
    websocket: specter::WebSocket,
}

#[derive(Clone, Debug)]
pub struct RuntimeConfig {
    pub listen_addr: SocketAddr,
    pub codex_transport: CodexTransport,
    pub codex_responses_wss_url: String,
    pub codex_responses_http_url: String,
    pub codex_models_url: String,
    pub codex_wss_connect_timeout: Duration,
    pub codex_max_concurrent: usize,
    pub codex_handshakes_per_min: u32,
    pub codex_wss_pool_size: usize,
    pub codex_catalog_client_version: String,
    pub codex_catalog_ttl: Duration,
    pub specter_h3_upgrade: bool,
    pub specter_http_tls_early_data: bool,
    pub specter_dns_cache: bool,
    pub specter_dns_cache_ttl: Duration,
    pub specter_max_pending_per_origin: usize,
    pub specter_stream_body_buffer_slots: usize,
    pub specter_h3_tunnel_byte_budget: usize,
    pub google_generate_base_url: String,
    pub windsurf_cloud_base_url: String,
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
    max_entries: usize,
}

#[derive(Default)]
struct ResponseStorageInner {
    volatile: HashMap<String, Arc<ResponseStateRecord>>,
    public_retrievable: HashMap<String, Arc<ResponseStateRecord>>,
}

impl Default for ResponseStorage {
    fn default() -> Self {
        Self {
            inner: Arc::new(Mutex::new(ResponseStorageInner::default())),
            ttl: Duration::from_secs(60 * 60),
            max_entries: RESPONSE_STORAGE_MAX_ENTRIES,
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
                .unwrap_or_else(|_| DEFAULT_BEDROCK_REGION.to_string()),
        );
        let routing_config = HotRoutingConfig::from_env(&auth_home);
        let codex_catalog = CodexCatalogCache::new(runtime.codex_catalog_config());
        let codex_max_concurrent = runtime.codex_max_concurrent;

        Self {
            specter: build_specter_client(&runtime),
            amp_store: AmpStore::from_env(),
            codex_home: Arc::new(codex_home),
            auth_home: Arc::new(auth_home),
            google_api_key,
            bedrock_region,
            runtime: Arc::new(runtime),
            routing_config,
            codex_catalog,
            cursor_sessions: Arc::new(CursorSessionStore::new()),
            auth_files: AuthFileCache::new(),
            codex_auth: ResolvedAuthCache::new(),
            cursor_auth: Arc::new(tokio::sync::Mutex::new(None)),
            google_auth: ResolvedAuthCache::new(),
            windsurf_auth: ResolvedAuthCache::new(),
            bedrock_auth: ResolvedAuthCache::new(),
            codex_handshake_sem: if codex_max_concurrent > 0 {
                Some(Arc::new(tokio::sync::Semaphore::new(codex_max_concurrent)))
            } else {
                None
            },
            codex_handshake_times: Arc::new(std::sync::Mutex::new(
                std::collections::VecDeque::new(),
            )),
            codex_ws_pool: Arc::new(tokio::sync::Mutex::new(VecDeque::new())),
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
        let codex_max_concurrent = runtime.codex_max_concurrent;

        Self {
            specter: build_specter_client(&runtime),
            amp_store: AmpStore::new(auth_home.join("amp-threads")),
            codex_home: Arc::new(codex_home),
            auth_home: Arc::new(auth_home),
            google_api_key: None,
            bedrock_region: Arc::<str>::from(DEFAULT_BEDROCK_REGION),
            runtime: Arc::new(runtime),
            routing_config,
            codex_catalog,
            cursor_sessions: Arc::new(CursorSessionStore::new()),
            auth_files: AuthFileCache::new(),
            codex_auth: ResolvedAuthCache::new(),
            cursor_auth: Arc::new(tokio::sync::Mutex::new(None)),
            google_auth: ResolvedAuthCache::new(),
            windsurf_auth: ResolvedAuthCache::new(),
            bedrock_auth: ResolvedAuthCache::new(),
            codex_handshake_sem: if codex_max_concurrent > 0 {
                Some(Arc::new(tokio::sync::Semaphore::new(codex_max_concurrent)))
            } else {
                None
            },
            codex_handshake_times: Arc::new(std::sync::Mutex::new(
                std::collections::VecDeque::new(),
            )),
            codex_ws_pool: Arc::new(tokio::sync::Mutex::new(VecDeque::new())),
            response_storage: ResponseStorage::default(),
            codex_wss_latched: Arc::new(AtomicBool::new(false)),
            codex_wss_failures: Arc::new(AtomicU32::new(0)),
        }
    }

    pub fn resolve_model(&self, input: &str) -> AppResult<ResolvedModel> {
        self.resolve_target(input).map(ResolvedModel::from)
    }

    pub fn resolve_model_for_format(
        &self,
        input: &str,
        source_format: &str,
    ) -> AppResult<ResolvedModel> {
        self.resolve_target_for_format(input, source_format)
            .map(ResolvedModel::from)
    }

    pub fn resolve_target(&self, input: &str) -> AppResult<ResolvedTarget> {
        if let Some(target) = self.routing_config.resolve_target_for_format(input, None)? {
            return Ok(target);
        }
        resolve_target_required(input)
    }

    pub fn resolve_target_for_format(
        &self,
        input: &str,
        source_format: &str,
    ) -> AppResult<ResolvedTarget> {
        if let Some(target) = self
            .routing_config
            .resolve_target_for_format(input, Some(source_format))?
        {
            return Ok(target);
        }
        resolve_target_required(input)
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

    pub(crate) async fn take_codex_ws(&self, key: &str) -> Option<specter::WebSocket> {
        if self.runtime.codex_wss_pool_size == 0 {
            return None;
        }
        let mut pool = self.codex_ws_pool.lock().await;
        let index = pool.iter().position(|entry| entry.key == key)?;
        pool.remove(index).map(|entry| entry.websocket)
    }

    pub(crate) async fn store_codex_ws(&self, key: String, websocket: specter::WebSocket) {
        let max_size = self.runtime.codex_wss_pool_size;
        if max_size == 0 {
            return;
        }
        let mut pool = self.codex_ws_pool.lock().await;
        pool.retain(|entry| entry.key != key);
        pool.push_back(CodexWsPoolEntry { key, websocket });
        while pool.len() > max_size {
            pool.pop_front();
        }
    }

    /// Acquire a handshake permit for Codex (WSS connect or HTTP call).
    /// Enforces the configured `codex_max_concurrent` (across handshake / HTTP
    /// request start) and
    /// `codex_handshakes_per_min` rolling window. Returns a rate-limit error
    /// with a short retry-after when either limit is hit.
    pub async fn codex_acquire_handshake(&self) -> AppResult<CodexHandshakePermit> {
        // Concurrency limit on simultaneous Codex WSS handshakes / HTTP call starts.
        // The owned permit is returned to the caller and is dropped only after the
        // network handshake/request future resolves. Acquire before recording the
        // rolling rate-limit timestamp so queued callers do not burn handshake budget.
        let permit = if let Some(sem) = &self.codex_handshake_sem {
            Some(
                sem.clone()
                    .acquire_owned()
                    .await
                    .map_err(|_| AppError::Upstream("codex concurrency semaphore closed".into()))?,
            )
        } else {
            None
        };

        // Rate limit (handshakes per minute window)
        if self.runtime.codex_handshakes_per_min > 0 {
            let mut times = self
                .codex_handshake_times
                .lock()
                .expect("codex handshake rate poisoned");
            let now = std::time::Instant::now();
            let window = std::time::Duration::from_secs(60);
            while let Some(front) = times.front() {
                if now.duration_since(*front) > window {
                    times.pop_front();
                } else {
                    break;
                }
            }
            if times.len() >= self.runtime.codex_handshakes_per_min as usize {
                return Err(AppError::TooManyRequests {
                    message: "codex handshake rate limit exceeded".into(),
                    retry_after_secs: Some(5),
                });
            }
            times.push_back(now);
        }

        Ok(permit)
    }

    pub fn remember_response_for_continuation(
        &self,
        record: NewResponseStateRecord,
    ) -> ResponseStateRecord {
        let record = Arc::new(response_state_record(record, false));
        let mut storage = self.response_storage.lock();
        storage
            .volatile
            .insert(record.adapter_response_id.clone(), Arc::clone(&record));
        self.response_storage.prune_locked(&mut storage);
        (*record).clone()
    }

    pub fn store_public_response(&self, record: NewResponseStateRecord) -> ResponseStateRecord {
        let record = Arc::new(response_state_record(record, true));
        let mut storage = self.response_storage.lock();
        storage
            .volatile
            .insert(record.adapter_response_id.clone(), Arc::clone(&record));
        storage
            .public_retrievable
            .insert(record.adapter_response_id.clone(), Arc::clone(&record));
        self.response_storage.prune_locked(&mut storage);
        (*record).clone()
    }

    pub fn continuation_response(&self, adapter_response_id: &str) -> Option<ResponseStateRecord> {
        self.response_storage
            .lock()
            .volatile
            .get(adapter_response_id)
            .filter(|record| !self.response_storage.is_expired(record))
            .map(|record| (**record).clone())
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
        self.response_storage.prune_locked(&mut storage);
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

    fn prune_locked(&self, storage: &mut ResponseStorageInner) {
        let ttl = self.ttl;
        storage
            .volatile
            .retain(|_, record| !is_expired(ttl, record));
        storage
            .public_retrievable
            .retain(|_, record| !is_expired(ttl, record));
        prune_map_to_max(&mut storage.public_retrievable, self.max_entries);
        prune_map_to_max(&mut storage.volatile, self.max_entries);
    }
}

fn build_specter_client(runtime: &RuntimeConfig) -> specter::Client {
    let capacity_policy = specter::CapacityPolicy::bounded(runtime.specter_max_pending_per_origin)
        .with_streaming_body_buffer_slots(runtime.specter_stream_body_buffer_slots)
        .with_h3_tunnel_byte_budget(runtime.specter_h3_tunnel_byte_budget);

    specter::Client::builder()
        .streaming_timeouts()
        .pool_acquire_timeout(Duration::from_millis(250))
        .prefer_http2(true)
        .capacity_policy(capacity_policy)
        .h3_upgrade(runtime.specter_h3_upgrade)
        .http_tls_early_data(runtime.specter_http_tls_early_data)
        .hickory_dns(runtime.specter_dns_cache)
        .dns_cache_ttl(runtime.specter_dns_cache_ttl)
        .build()
        .expect("failed to construct Specter client")
}

fn is_expired(ttl: Duration, record: &ResponseStateRecord) -> bool {
    record
        .updated_at
        .elapsed()
        .map(|elapsed| elapsed > ttl)
        .unwrap_or(false)
}

fn prune_map_to_max(map: &mut HashMap<String, Arc<ResponseStateRecord>>, max_entries: usize) {
    while map.len() > max_entries {
        let Some(oldest_key) = map
            .iter()
            .min_by_key(|(_, record)| record.updated_at)
            .map(|(key, _)| key.clone())
        else {
            break;
        };
        map.remove(&oldest_key);
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
            codex_models_url: var("UMP_V2_CODEX_MODELS_URL")
                .unwrap_or_else(|_| CODEX_MODELS_URL.to_string()),
            codex_wss_connect_timeout: duration_ms_env(
                &mut var,
                "UMP_V2_CODEX_WSS_CONNECT_TIMEOUT_MS",
                5000,
            )?,
            codex_max_concurrent: usize_env(&mut var, "UMP_V2_CODEX_MAX_CONCURRENT", 20)?,
            codex_handshakes_per_min: u32_env(&mut var, "UMP_V2_CODEX_HANDSHAKES_PER_MIN", 55)?,
            codex_wss_pool_size: usize_env(
                &mut var,
                "UMP_V2_CODEX_WSS_POOL_SIZE",
                DEFAULT_CODEX_WSS_POOL_SIZE,
            )?,
            codex_catalog_client_version: codex_catalog_config.client_version,
            codex_catalog_ttl: codex_catalog_config.ttl,
            specter_h3_upgrade: bool_env(&mut var, "UMP_V2_SPECTER_H3_UPGRADE", true)?,
            specter_http_tls_early_data: bool_env(
                &mut var,
                "UMP_V2_SPECTER_HTTP_TLS_EARLY_DATA",
                true,
            )?,
            specter_dns_cache: bool_env(&mut var, "UMP_V2_SPECTER_DNS_CACHE", true)?,
            specter_dns_cache_ttl: duration_ms_env(
                &mut var,
                "UMP_V2_SPECTER_DNS_CACHE_TTL_MS",
                DEFAULT_SPECTER_DNS_CACHE_TTL_MS,
            )?,
            specter_max_pending_per_origin: usize_env(
                &mut var,
                "UMP_V2_SPECTER_MAX_PENDING_PER_ORIGIN",
                DEFAULT_SPECTER_MAX_PENDING_PER_ORIGIN,
            )?,
            specter_stream_body_buffer_slots: usize_env(
                &mut var,
                "UMP_V2_SPECTER_STREAM_BODY_BUFFER_SLOTS",
                DEFAULT_SPECTER_STREAM_BODY_BUFFER_SLOTS,
            )?,
            specter_h3_tunnel_byte_budget: usize_env(
                &mut var,
                "UMP_V2_SPECTER_H3_TUNNEL_BYTE_BUDGET",
                DEFAULT_SPECTER_H3_TUNNEL_BYTE_BUDGET,
            )?,
            google_generate_base_url: var("UMP_V2_GOOGLE_GENERATE_BASE_URL")
                .unwrap_or_else(|_| GOOGLE_GENERATE_BASE_URL.to_string()),
            windsurf_cloud_base_url: var("UMP_V2_WINDSURF_CLOUD_BASE_URL")
                .unwrap_or_else(|_| WINDSURF_CLOUD_BASE_URL.to_string()),
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
            codex_models_url: CODEX_MODELS_URL.to_string(),
            codex_wss_connect_timeout: Duration::from_millis(5000),
            codex_max_concurrent: 20,
            codex_handshakes_per_min: 55,
            codex_wss_pool_size: DEFAULT_CODEX_WSS_POOL_SIZE,
            codex_catalog_client_version: CodexCatalogConfig::default().client_version,
            codex_catalog_ttl: CodexCatalogConfig::default().ttl,
            specter_h3_upgrade: true,
            specter_http_tls_early_data: true,
            specter_dns_cache: true,
            specter_dns_cache_ttl: Duration::from_millis(DEFAULT_SPECTER_DNS_CACHE_TTL_MS),
            specter_max_pending_per_origin: DEFAULT_SPECTER_MAX_PENDING_PER_ORIGIN,
            specter_stream_body_buffer_slots: DEFAULT_SPECTER_STREAM_BODY_BUFFER_SLOTS,
            specter_h3_tunnel_byte_budget: DEFAULT_SPECTER_H3_TUNNEL_BYTE_BUDGET,
            google_generate_base_url: GOOGLE_GENERATE_BASE_URL.to_string(),
            windsurf_cloud_base_url: WINDSURF_CLOUD_BASE_URL.to_string(),
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

fn bool_env<F>(var: &mut F, name: &str, default_value: bool) -> Result<bool, String>
where
    F: FnMut(&str) -> Result<String, env::VarError>,
{
    match var(name) {
        Ok(value) => match value.trim().to_ascii_lowercase().as_str() {
            "1" | "true" | "yes" | "on" => Ok(true),
            "0" | "false" | "no" | "off" => Ok(false),
            other => Err(format!("invalid {name}: {other}")),
        },
        Err(_) => Ok(default_value),
    }
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
            config.codex_models_url,
            "https://chatgpt.com/backend-api/codex/models"
        );
        assert_eq!(
            config.codex_wss_connect_timeout,
            Duration::from_millis(5000)
        );
        assert_eq!(config.codex_max_concurrent, 20);
        assert_eq!(config.codex_handshakes_per_min, 55);
        assert_eq!(config.codex_wss_pool_size, 4);
        assert!(config.specter_h3_upgrade);
        assert!(config.specter_http_tls_early_data);
        assert!(config.specter_dns_cache);
        assert_eq!(config.specter_dns_cache_ttl, Duration::from_millis(300_000));
        assert_eq!(config.specter_max_pending_per_origin, 20);
        assert_eq!(config.specter_stream_body_buffer_slots, 32);
        assert_eq!(config.specter_h3_tunnel_byte_budget, 262_144);
        assert_eq!(
            config.google_generate_base_url,
            "https://generativelanguage.googleapis.com"
        );
        assert_eq!(config.windsurf_cloud_base_url, "https://server.codeium.com");
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
            ("UMP_V2_CODEX_MODELS_URL", "http://127.0.0.1:1/models"),
            ("UMP_V2_CODEX_WSS_CONNECT_TIMEOUT_MS", "250"),
            ("UMP_V2_CODEX_MAX_CONCURRENT", "7"),
            ("UMP_V2_CODEX_HANDSHAKES_PER_MIN", "9"),
            ("UMP_V2_CODEX_WSS_POOL_SIZE", "3"),
            ("UMP_V2_SPECTER_H3_UPGRADE", "false"),
            ("UMP_V2_SPECTER_HTTP_TLS_EARLY_DATA", "false"),
            ("UMP_V2_SPECTER_DNS_CACHE", "false"),
            ("UMP_V2_SPECTER_DNS_CACHE_TTL_MS", "1234"),
            ("UMP_V2_SPECTER_MAX_PENDING_PER_ORIGIN", "11"),
            ("UMP_V2_SPECTER_STREAM_BODY_BUFFER_SLOTS", "17"),
            ("UMP_V2_SPECTER_H3_TUNNEL_BYTE_BUDGET", "65536"),
            ("UMP_V2_GOOGLE_GENERATE_BASE_URL", "http://127.0.0.1:9999"),
            ("UMP_V2_WINDSURF_CLOUD_BASE_URL", "http://127.0.0.1:19999"),
            ("UMP_V2_BEDROCK_DISCOVERY_TIMEOUT_MS", "125"),
        ])
        .unwrap();

        assert_eq!(config.listen_addr, "127.0.0.1:19000".parse().unwrap());
        assert_eq!(config.codex_transport, CodexTransport::Http);
        assert_eq!(config.codex_responses_wss_url, "ws://127.0.0.1:1/ws");
        assert_eq!(config.codex_responses_http_url, "http://127.0.0.1:1/http");
        assert_eq!(config.codex_models_url, "http://127.0.0.1:1/models");
        assert_eq!(config.codex_wss_connect_timeout, Duration::from_millis(250));
        assert_eq!(config.codex_max_concurrent, 7);
        assert_eq!(config.codex_handshakes_per_min, 9);
        assert_eq!(config.codex_wss_pool_size, 3);
        assert!(!config.specter_h3_upgrade);
        assert!(!config.specter_http_tls_early_data);
        assert!(!config.specter_dns_cache);
        assert_eq!(config.specter_dns_cache_ttl, Duration::from_millis(1234));
        assert_eq!(config.specter_max_pending_per_origin, 11);
        assert_eq!(config.specter_stream_body_buffer_slots, 17);
        assert_eq!(config.specter_h3_tunnel_byte_budget, 65536);
        assert_eq!(config.google_generate_base_url, "http://127.0.0.1:9999");
        assert_eq!(config.windsurf_cloud_base_url, "http://127.0.0.1:19999");
        assert_eq!(config.bedrock_discovery_timeout, Duration::from_millis(125));
    }

    #[test]
    fn runtime_config_rejects_invalid_values() {
        assert!(config_from(&[("UMP_V2_LISTEN_ADDR", "not an address")]).is_err());
        assert!(config_from(&[("UMP_V2_CODEX_TRANSPORT", "ftp")]).is_err());
        assert!(config_from(&[("UMP_V2_CODEX_MAX_CONCURRENT", "nope")]).is_err());
        assert!(config_from(&[("UMP_V2_SPECTER_H3_UPGRADE", "maybe")]).is_err());
        assert!(config_from(&[("UMP_V2_SPECTER_DNS_CACHE_TTL_MS", "nope")]).is_err());
        assert!(config_from(&[("UMP_V2_SPECTER_MAX_PENDING_PER_ORIGIN", "nope")]).is_err());
    }

    #[test]
    fn build_specter_client_applies_capacity_policy() {
        let config = RuntimeConfig {
            specter_max_pending_per_origin: 9,
            specter_stream_body_buffer_slots: 33,
            specter_h3_tunnel_byte_budget: 128 * 1024,
            ..RuntimeConfig::default()
        };

        let client = build_specter_client(&config);

        assert_eq!(client.h1_max_connections_per_origin(), 9);
        assert_eq!(client.h2_max_concurrent_streams_per_connection(), Some(9));
        assert_eq!(client.h2_streaming_body_buffer_slots(), 33);
        assert_eq!(client.h3_streaming_body_buffer_slots(), 33);
        assert_eq!(client.h3_tunnel_outbound_byte_budget(), 128 * 1024);
        assert_eq!(client.h3_tunnel_inbound_byte_budget(), 128 * 1024);
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

    #[tokio::test]
    async fn codex_handshake_permit_is_held_until_dropped() {
        let temp = tempfile::tempdir().unwrap();
        let mut state = AppState::for_tests(temp.path().join("codex"), temp.path().join("ump"));
        state.codex_handshake_sem = Some(Arc::new(tokio::sync::Semaphore::new(1)));
        Arc::make_mut(&mut state.runtime).codex_handshakes_per_min = 0;

        let first = state.codex_acquire_handshake().await.unwrap();
        assert!(first.is_some());

        let blocked =
            tokio::time::timeout(Duration::from_millis(10), state.codex_acquire_handshake()).await;
        assert!(
            blocked.is_err(),
            "second acquire should wait while permit is held"
        );

        drop(first);
        assert!(state.codex_acquire_handshake().await.unwrap().is_some());
    }
}
