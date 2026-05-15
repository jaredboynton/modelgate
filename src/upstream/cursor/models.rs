//! Cursor model discovery + fallback table.
//!
//! Wraps the unary `GetUsableModels` transport call with a token-keyed
//! cache (10 minute TTL, 16-hex SHA-256 prefix). On any error path we fall
//! back to the pinned `composer-1.5`, `composer-2`, and `composer-2-fast`
//! rows tagged `discovery=Fallback` so callers always get a complete row
//! set even when discovery is unreachable.

use std::collections::HashMap;
use std::sync::OnceLock;
use std::time::{Duration, Instant};

use sha2::{Digest, Sha256};
use tokio::sync::Mutex;

use crate::upstream::cursor::proto::{decode_get_usable_models_response, ModelDescriptorRaw};
use crate::upstream::cursor::transport::{
    strip_optional_connect_envelope, unary_get_usable_models,
};

/// Source the descriptor row was minted from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiscoverySource {
    /// Live response from `GetUsableModels`.
    Live,
    /// Hard-coded fallback row (transport / decode failure, or empty live).
    Fallback,
}

/// Normalized model descriptor used by routing and the public `/v1/models`
/// surface.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelDescriptor {
    pub id: String,
    pub upstream_id: String,
    pub discovery: DiscoverySource,
    pub context_window: u32,
    pub max_output_tokens: u32,
    pub supports_reasoning: bool,
}

/// Default per-model context window (matches Composer 2 family launch
/// defaults; revisited live in Phase 0).
pub const DEFAULT_CONTEXT_WINDOW: u32 = 200_000;
/// Default per-model max output tokens.
pub const DEFAULT_MAX_OUTPUT_TOKENS: u32 = 64_000;

/// Pinned fallback rows. Mirrors the v1/opencode fallback set; row order
/// matters so it stays alphabetical by `id`.
pub static FALLBACK_MODELS: &[FallbackModel] = &[
    FallbackModel {
        id: "composer-1.5",
        upstream_id: "composer-1.5",
        supports_reasoning: false,
    },
    FallbackModel {
        id: "composer-2",
        upstream_id: "composer-2",
        supports_reasoning: true,
    },
    FallbackModel {
        id: "composer-2-fast",
        upstream_id: "composer-2-fast",
        supports_reasoning: true,
    },
];

/// Compact static fallback row. Materialized into a full `ModelDescriptor`
/// on demand via `fallback_descriptors`.
#[derive(Debug, Clone, Copy)]
pub struct FallbackModel {
    pub id: &'static str,
    pub upstream_id: &'static str,
    pub supports_reasoning: bool,
}

impl FallbackModel {
    fn into_descriptor(self) -> ModelDescriptor {
        ModelDescriptor {
            id: self.id.to_string(),
            upstream_id: self.upstream_id.to_string(),
            discovery: DiscoverySource::Fallback,
            context_window: DEFAULT_CONTEXT_WINDOW,
            max_output_tokens: DEFAULT_MAX_OUTPUT_TOKENS,
            supports_reasoning: self.supports_reasoning,
        }
    }
}

/// Materialize the fallback table as a fresh owned vector.
pub fn fallback_descriptors() -> Vec<ModelDescriptor> {
    FALLBACK_MODELS
        .iter()
        .copied()
        .map(FallbackModel::into_descriptor)
        .collect()
}

#[derive(Debug, Clone)]
struct CacheEntry {
    fetched_at: Instant,
    descriptors: Vec<ModelDescriptor>,
}

/// 10-minute cache TTL. Live invalidation is the caller's responsibility
/// when token rotation happens; this just bounds staleness for the same
/// token's hash key.
const CACHE_TTL: Duration = Duration::from_secs(600);

fn cache() -> &'static Mutex<HashMap<String, CacheEntry>> {
    static CELL: OnceLock<Mutex<HashMap<String, CacheEntry>>> = OnceLock::new();
    CELL.get_or_init(|| Mutex::new(HashMap::new()))
}

/// 16-hex SHA-256 prefix keyed on the access token. Matches the v1
/// diagnostics convention.
pub fn cache_key(token: &str) -> String {
    let digest = Sha256::digest(token.as_bytes());
    let full = hex::encode(digest);
    full[..16].to_string()
}

/// Fetch the active `GetUsableModels` row set for `token`. Returns the
/// fallback table on any transport / decode error so callers always get a
/// non-empty list. Successful live rows are tagged `DiscoverySource::Live`,
/// deduped by id, and sorted alphabetically.
pub async fn fetch_usable_models(token: &str) -> Vec<ModelDescriptor> {
    let key = cache_key(token);
    {
        let cache_guard = cache().lock().await;
        if let Some(entry) = cache_guard.get(&key) {
            if entry.fetched_at.elapsed() < CACHE_TTL {
                return entry.descriptors.clone();
            }
        }
    }

    let descriptors = match fetch_live(token).await {
        Some(rows) if !rows.is_empty() => rows,
        _ => fallback_descriptors(),
    };

    let mut cache_guard = cache().lock().await;
    cache_guard.insert(
        key,
        CacheEntry {
            fetched_at: Instant::now(),
            descriptors: descriptors.clone(),
        },
    );
    descriptors
}

async fn fetch_live(token: &str) -> Option<Vec<ModelDescriptor>> {
    let bytes = match unary_get_usable_models(token).await {
        Ok(bytes) => bytes,
        Err(err) => {
            tracing::debug!(?err, "cursor: GetUsableModels failed; using fallback");
            return None;
        }
    };
    let body = strip_optional_connect_envelope(&bytes);
    let raw = decode_get_usable_models_response(body);
    if raw.is_empty() {
        return None;
    }
    Some(normalize(raw))
}

fn normalize(raw: Vec<ModelDescriptorRaw>) -> Vec<ModelDescriptor> {
    let mut deduped: HashMap<String, ModelDescriptor> = HashMap::new();
    for row in raw {
        if row.model_id.is_empty() {
            continue;
        }
        let id = row.model_id.clone();
        deduped.insert(
            id.clone(),
            ModelDescriptor {
                id: id.clone(),
                upstream_id: id,
                discovery: DiscoverySource::Live,
                context_window: DEFAULT_CONTEXT_WINDOW,
                max_output_tokens: DEFAULT_MAX_OUTPUT_TOKENS,
                supports_reasoning: row.supports_reasoning,
            },
        );
    }
    let mut out: Vec<ModelDescriptor> = deduped.into_values().collect();
    out.sort_by(|a, b| a.id.cmp(&b.id));
    out
}
