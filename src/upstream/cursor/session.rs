//! Cursor conversation session store.
//!
//! Owns per-conversation continuation state for the Cursor upstream:
//! checkpoint blobs, pending tool calls, response/conversation IDs, and the
//! continuation-key fingerprint that binds a stored entry to a specific
//! route/provider/model/target-format/stable-field tuple.
//!
//! Per ralplan Section 4 plan items 8-13, all Cursor session business logic
//! lives here, not in `state.rs`. `AppState` owns only `Arc<CursorSessionStore>`.
//!
//! Continuation keys are bound to (route, provider, upstream_model,
//! target_format, stable_field_hash, response_id, conversation_id). Lookup
//! must verify every component before merging prior state. Mismatch returns
//! `None`, never stale data.

use std::{
    collections::{HashMap, VecDeque},
    sync::{Arc, RwLock},
    time::{Duration, Instant},
};

use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::{
    cursor_agent::{CursorContinuationKey, CursorRoute, CursorToolCall},
    model_alias::{Provider, TargetFormat},
};

/// SHA-256 digest used as the lookup key in the session map.
pub type ContinuationHash = [u8; 32];

/// Persistent state for a Cursor conversation between turns.
///
/// Stored after the run engine observes a `Checkpoint` event or whenever a
/// pending tool call is emitted. The continuation key fingerprint is checked
/// on every lookup; a mismatch on any field causes the lookup to return
/// `None` so callers cannot accidentally merge state across drift.
#[derive(Clone, Debug)]
pub struct ConversationState {
    pub checkpoint: Option<String>,
    pub pending_tool_calls: Vec<CursorToolCall>,
    pub last_access: Instant,
    pub route: CursorRoute,
    pub provider: Provider,
    pub upstream_model: String,
    pub target_format: TargetFormat,
    pub stable_field_hash: [u8; 32],
    pub response_id: String,
    pub conversation_id: String,
    pub blob_store: HashMap<String, Vec<u8>>,
}

#[derive(Clone, Copy, Debug)]
pub struct CursorSessionConfig {
    pub max_active: usize,
    pub ttl: Duration,
    pub cleanup_interval: Duration,
}

impl CursorSessionConfig {
    pub const DEFAULT_MAX_ACTIVE: usize = 1000;
    pub const DEFAULT_TTL: Duration = Duration::from_secs(60 * 60);
    pub const DEFAULT_CLEANUP_INTERVAL: Duration = Duration::from_secs(60);

    pub fn defaults() -> Self {
        Self {
            max_active: Self::DEFAULT_MAX_ACTIVE,
            ttl: Self::DEFAULT_TTL,
            cleanup_interval: Self::DEFAULT_CLEANUP_INTERVAL,
        }
    }
}

impl Default for CursorSessionConfig {
    fn default() -> Self {
        Self::defaults()
    }
}

struct StoreInner {
    map: HashMap<ContinuationHash, ConversationState>,
    lru: VecDeque<ContinuationHash>,
    max_active: usize,
}

impl StoreInner {
    fn new(max_active: usize) -> Self {
        Self {
            map: HashMap::new(),
            lru: VecDeque::new(),
            max_active: max_active.max(1),
        }
    }

    fn touch(&mut self, hash: &ContinuationHash) {
        if let Some(idx) = self.lru.iter().position(|item| item == hash) {
            self.lru.remove(idx);
        }
        self.lru.push_back(*hash);
    }

    fn forget(&mut self, hash: &ContinuationHash) {
        if let Some(idx) = self.lru.iter().position(|item| item == hash) {
            self.lru.remove(idx);
        }
    }

    fn evict_oldest(&mut self) {
        if let Some(oldest) = self.lru.pop_front() {
            self.map.remove(&oldest);
        }
    }
}

/// Concurrent in-memory store for Cursor conversation continuations.
///
/// Cloning is cheap; the inner `Arc<RwLock<...>>` is shared. Drop the last
/// clone to stop the background cleanup task.
#[derive(Clone)]
pub struct CursorSessionStore {
    inner: Arc<RwLock<StoreInner>>,
    config: CursorSessionConfig,
}

impl CursorSessionStore {
    /// Build a session store with default capacity and TTL, spawning the
    /// background cleanup task on the current Tokio runtime if available.
    pub fn new() -> Self {
        Self::with_config(CursorSessionConfig::defaults())
    }

    /// Build a session store with a custom configuration. The background
    /// cleanup task is best-effort: it only spawns when a Tokio runtime is
    /// active so non-async tests can still construct the store.
    pub fn with_config(config: CursorSessionConfig) -> Self {
        let store = Self {
            inner: Arc::new(RwLock::new(StoreInner::new(config.max_active))),
            config,
        };
        store.spawn_cleanup_task();
        store
    }

    fn spawn_cleanup_task(&self) {
        if tokio::runtime::Handle::try_current().is_err() {
            return;
        }
        let weak = Arc::downgrade(&self.inner);
        let interval = self.config.cleanup_interval;
        let ttl = self.config.ttl;
        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(interval);
            ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
            ticker.tick().await;
            loop {
                ticker.tick().await;
                let Some(inner) = weak.upgrade() else {
                    break;
                };
                Self::cleanup_inner(&inner, ttl);
            }
        });
    }

    fn cleanup_inner(inner: &Arc<RwLock<StoreInner>>, max_age: Duration) {
        let now = Instant::now();
        let Ok(mut guard) = inner.write() else {
            return;
        };
        let stale: Vec<ContinuationHash> = guard
            .map
            .iter()
            .filter_map(|(hash, state)| {
                if now.saturating_duration_since(state.last_access) > max_age {
                    Some(*hash)
                } else {
                    None
                }
            })
            .collect();
        for hash in stale {
            guard.map.remove(&hash);
            guard.forget(&hash);
        }
    }

    /// Insert or refresh a continuation entry. Returns the SHA-256 digest of
    /// the canonical key encoding so callers can later look the entry up.
    pub fn store_continuation(
        &self,
        key: &CursorContinuationKey,
        mut state: ConversationState,
    ) -> ContinuationHash {
        let hash = continuation_hash(key);
        state.last_access = Instant::now();
        state.stable_field_hash = stable_field_hash(key);

        let mut guard = self.write();
        guard.map.insert(hash, state);
        guard.touch(&hash);
        while guard.map.len() > guard.max_active {
            guard.evict_oldest();
        }
        hash
    }

    /// Look up a continuation by key. Verifies route, provider, upstream
    /// model, target format, stable field hash, response ID, and
    /// conversation ID. Any mismatch returns `None` per ralplan Section 4
    /// plan item 12.
    pub fn lookup_continuation(&self, key: &CursorContinuationKey) -> Option<ConversationState> {
        let hash = continuation_hash(key);
        let expected_stable = stable_field_hash(key);
        let mut guard = self.write();
        let entry = guard.map.get_mut(&hash)?;
        if !entry_matches(entry, key, &expected_stable) {
            return None;
        }
        entry.last_access = Instant::now();
        let cloned = entry.clone();
        guard.touch(&hash);
        Some(cloned)
    }

    /// Remove a pending tool call, returning it once and only once. Returns
    /// `None` if the call is missing, already consumed, or the entry's
    /// continuation key no longer matches.
    pub fn consume_pending_tool_call(
        &self,
        key: &CursorContinuationKey,
        call_id: &str,
    ) -> Option<CursorToolCall> {
        let hash = continuation_hash(key);
        let expected_stable = stable_field_hash(key);
        let mut guard = self.write();
        let entry = guard.map.get_mut(&hash)?;
        if !entry_matches(entry, key, &expected_stable) {
            return None;
        }
        let position = entry
            .pending_tool_calls
            .iter()
            .position(|call| call.id == call_id)?;
        let call = entry.pending_tool_calls.remove(position);
        entry.last_access = Instant::now();
        guard.touch(&hash);
        Some(call)
    }

    /// Drop entries older than `max_age`. Public so tests can drive cleanup
    /// without waiting on the background task.
    pub fn cleanup_expired(&self, max_age: Duration) {
        Self::cleanup_inner(&self.inner, max_age);
    }

    /// Number of active conversations. Intended for tests/observability.
    pub fn len(&self) -> usize {
        self.read().map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Configured maximum active conversations.
    pub fn max_active(&self) -> usize {
        self.config.max_active
    }

    fn read(&self) -> std::sync::RwLockReadGuard<'_, StoreInner> {
        self.inner.read().expect("cursor session store poisoned")
    }

    fn write(&self) -> std::sync::RwLockWriteGuard<'_, StoreInner> {
        self.inner.write().expect("cursor session store poisoned")
    }
}

impl Default for CursorSessionStore {
    fn default() -> Self {
        Self::new()
    }
}

fn entry_matches(
    entry: &ConversationState,
    key: &CursorContinuationKey,
    expected_stable: &[u8; 32],
) -> bool {
    entry.route == key.route
        && entry.provider == key.provider
        && entry.upstream_model == key.upstream_model
        && entry.target_format == key.target_format
        && &entry.stable_field_hash == expected_stable
        && entry.response_id == key.response_id
        && entry.conversation_id == key.conversation_id
}

fn continuation_hash(key: &CursorContinuationKey) -> ContinuationHash {
    #[derive(Serialize)]
    struct CanonicalKey<'a> {
        route: &'a CursorRoute,
        provider: &'a Provider,
        upstream_model: &'a str,
        target_format: &'a TargetFormat,
        stable: [u8; 32],
        response_id: &'a str,
        conversation_id: &'a str,
    }
    let canonical = CanonicalKey {
        route: &key.route,
        provider: &key.provider,
        upstream_model: key.upstream_model.as_str(),
        target_format: &key.target_format,
        stable: stable_field_hash(key),
        response_id: key.response_id.as_str(),
        conversation_id: key.conversation_id.as_str(),
    };
    let bytes = serde_json::to_vec(&canonical).unwrap_or_default();
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    let digest = hasher.finalize();
    let mut out = [0u8; 32];
    out.copy_from_slice(&digest);
    out
}

fn stable_field_hash(key: &CursorContinuationKey) -> [u8; 32] {
    let canonical = canonicalize_stable_fields(&key.stable_request_fields);
    let bytes = serde_json::to_vec(&canonical).unwrap_or_default();
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    let digest = hasher.finalize();
    let mut out = [0u8; 32];
    out.copy_from_slice(&digest);
    out
}

fn canonicalize_stable_fields(value: &serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::Object(map) => {
            let mut entries: Vec<(String, serde_json::Value)> = map
                .iter()
                .map(|(k, v)| (k.clone(), canonicalize_stable_fields(v)))
                .collect();
            entries.sort_by(|a, b| a.0.cmp(&b.0));
            let mut sorted = serde_json::Map::with_capacity(entries.len());
            for (k, v) in entries {
                sorted.insert(k, v);
            }
            serde_json::Value::Object(sorted)
        }
        serde_json::Value::Array(items) => {
            serde_json::Value::Array(items.iter().map(canonicalize_stable_fields).collect())
        }
        other => other.clone(),
    }
}
