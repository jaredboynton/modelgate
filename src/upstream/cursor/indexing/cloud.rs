//! Cursor cloud index adapter.
//!
//! Implements the five RepositoryService RPCs against `repo42.cursor.sh`:
//! `SearchRepositoryV2`, `FastRepoInitHandshakeV2`, `FastUpdateFileV2`,
//! `EnsureIndexCreated`, `FastRepoSyncComplete`. Field tags and constants
//! come from `indexing-extraction.md` "Cloud index RPCs".
//!
//! Headers for `repo42.cursor.sh` use `x-cursor-client-type: ide` and
//! `x-cursor-client-version: 3.3.8` (NOT `cli`). Auth uses the Cursor
//! access token shared with AgentService.
//!
//! Bootstrap orchestration: handshake → chunked uploads → ensure → sync-
//! complete. Failure of any phase rolls up to a single
//! `EnsureResult::Failed`; the caller treats indexing as best-effort and
//! never propagates indexing errors into normal chat.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use bytes::Bytes;
use h2::client::SendRequest;
use http::{Request, StatusCode};
use rustls::pki_types::ServerName;
use sha2::{Digest, Sha256};
use tokio::net::TcpStream;
use tokio::task::JoinHandle;
use tokio::time::timeout;
use tokio_rustls::TlsConnector;

use crate::upstream::cursor::indexing::wire::{
    codebase_status, decode_fast_repo_init_handshake_v2_response,
    decode_fast_update_file_v2_response_status, decode_search_repository_response,
    encode_ensure_index_created_request, encode_fast_repo_init_handshake_v2_request,
    encode_fast_repo_sync_complete_request, encode_fast_update_file_v2_request,
    encode_search_repository_request, is_fast_update_file_v2_success, parse_connect_error,
    strip_connect_unary_body, CodeResult, MetadataSource, RepositoryContext,
    RepositoryIndexMetadata, SyncCodebaseStatus, UploadFile, FAST_UPDATE_STATUS_SUCCESS,
};
use crate::upstream::cursor::transport::{tls_connector, TransportError};
use crate::upstream::cursor::workspace::{is_within_directory, RepoMetadata};
use crate::upstream::cursor::{cursor_client_version, CURSOR_REPO_HOST};

/// Hard-coded RPC paths. The TS plugin allows env overrides; UMP holds
/// these to the canonical values until live Phase 0 confirms an alternate
/// floor is acceptable.
pub const SEARCH_REPOSITORY_V2_PATH: &str = "/aiserver.v1.RepositoryService/SearchRepositoryV2";
pub const FAST_REPO_INIT_HANDSHAKE_V2_PATH: &str =
    "/aiserver.v1.RepositoryService/FastRepoInitHandshakeV2";
pub const FAST_UPDATE_FILE_V2_PATH: &str = "/aiserver.v1.RepositoryService/FastUpdateFileV2";
pub const ENSURE_INDEX_CREATED_PATH: &str = "/aiserver.v1.RepositoryService/EnsureIndexCreated";
pub const FAST_REPO_SYNC_COMPLETE_PATH: &str =
    "/aiserver.v1.RepositoryService/FastRepoSyncComplete";

/// Default IDE-channel client version used in `x-cursor-client-version`
/// for repo42 traffic. TS plugin default; UMP exposes the
/// `UMP_CURSOR_INDEX_CLIENT_VERSION` env override.
pub const DEFAULT_INDEX_CLIENT_VERSION: &str = "3.3.8";

pub const ENV_INDEX_CLIENT_VERSION: &str = "UMP_CURSOR_INDEX_CLIENT_VERSION";

pub const DEFAULT_SEARCH_TIMEOUT_MS: u64 = 8_000;
pub const DEFAULT_BOOTSTRAP_TIMEOUT_MS: u64 = 180_000;
pub const DEFAULT_TOP_K: i32 = 10;
pub const DEFAULT_UPLOAD_MAX_FILES: usize = 300;
pub const DEFAULT_UPLOAD_MAX_FILE_BYTES: u64 = 256_000;
pub const DEFAULT_UPLOAD_MAX_BATCH_BYTES: usize = 900_000;

// Sensitive-filename allowlist (deny). Aligned with TS `SENSITIVE_FILENAMES`.
const SENSITIVE_FILENAMES: &[&str] = &[
    ".env",
    ".env.local",
    ".env.development",
    ".env.production",
    ".npmrc",
    ".pypirc",
    "credentials.json",
    "secrets.json",
];

// Cert/key extension deny list applied alongside the sensitive filenames.
const CERT_EXTENSION_DENY: &[&str] = &["pem", "key", "p12", "pfx"];

const TEXT_EXTENSIONS: &[&str] = &[
    "cjs", "css", "go", "html", "js", "json", "jsonc", "jsx", "md", "mjs", "py", "rs", "sh", "sql",
    "ts", "tsx", "txt", "yaml", "yml",
];

const DEFAULT_IGNORES: &[&str] = &[
    ".git",
    ".hg",
    ".svn",
    ".cursor",
    ".omc",
    ".sisyphus",
    ".wire-harness-runs",
    "node_modules",
    "dist",
    "build",
    ".next",
    ".turbo",
    "coverage",
];

/// Cloud search outcome. Mirrors the TS `CursorCloudSearchOutcome` union.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum SearchOutcome {
    Ok,
    NoToken,
    NoResults,
    CodebaseNotFound,
    RpcError,
    TransportError,
    Timeout,
    Disabled,
    BootstrapFailed,
}

impl SearchOutcome {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::NoToken => "no-token",
            Self::NoResults => "no-results",
            Self::CodebaseNotFound => "codebase-not-found",
            Self::RpcError => "rpc-error",
            Self::TransportError => "transport-error",
            Self::Timeout => "timeout",
            Self::Disabled => "disabled",
            Self::BootstrapFailed => "bootstrap-failed",
        }
    }
}

/// Per-call diagnostic record. Caller is expected to log a redacted subset.
#[derive(Debug, Clone, Default)]
pub struct CloudDiagnostic {
    pub outcome: Option<SearchOutcome>,
    pub result_count: usize,
    pub server_error_code: Option<String>,
    pub server_error_message: Option<String>,
    pub response_bytes: usize,
    pub elapsed_ms: u128,
}

/// Cloud search response envelope.
#[derive(Debug, Clone)]
pub struct SearchResults {
    pub source: &'static str,
    pub body: String,
    pub outcome: SearchOutcome,
    pub result_count: usize,
    pub diagnostic: CloudDiagnostic,
}

#[derive(Debug, Clone)]
pub struct HandshakeResult {
    pub status: i32,
    pub codebases: Vec<CodebaseTarget>,
}

#[derive(Debug, Clone)]
pub struct CodebaseTarget {
    pub codebase_id: String,
    pub status: i32,
}

#[derive(Debug, Clone)]
pub struct UpdateResult {
    pub success: bool,
    pub failed_chunk_count: usize,
    pub total_chunk_count: usize,
}

#[derive(Debug, Clone)]
pub struct EnsureResult {
    pub outcome: EnsureOutcome,
    pub upload_count: usize,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum EnsureOutcome {
    AlreadyIndexed,
    Uploaded,
    NotEnabled,
    NoToken,
    Failed,
}

#[derive(Debug, Clone)]
pub struct SyncResult {
    pub success: bool,
}

/// Cloud adapter configuration. All defaults match the TS plugin; envs
/// follow the UMP-prefixed names per `indexing-extraction.md` "Opt-in env
/// contract".
#[derive(Debug, Clone)]
pub struct CloudConfig {
    pub url: String,
    pub search_path: String,
    pub timeout_ms: u64,
    pub bootstrap_timeout_ms: u64,
    pub top_k: i32,
    pub upload_max_files: usize,
    pub upload_max_file_bytes: u64,
    pub upload_max_batch_bytes: usize,
    pub client_version: String,
}

impl Default for CloudConfig {
    fn default() -> Self {
        Self {
            url: format!("https://{CURSOR_REPO_HOST}"),
            search_path: SEARCH_REPOSITORY_V2_PATH.into(),
            timeout_ms: DEFAULT_SEARCH_TIMEOUT_MS,
            bootstrap_timeout_ms: DEFAULT_BOOTSTRAP_TIMEOUT_MS,
            top_k: DEFAULT_TOP_K,
            upload_max_files: DEFAULT_UPLOAD_MAX_FILES,
            upload_max_file_bytes: DEFAULT_UPLOAD_MAX_FILE_BYTES,
            upload_max_batch_bytes: DEFAULT_UPLOAD_MAX_BATCH_BYTES,
            client_version: std::env::var(ENV_INDEX_CLIENT_VERSION)
                .ok()
                .filter(|value| !value.trim().is_empty())
                .unwrap_or_else(|| DEFAULT_INDEX_CLIENT_VERSION.into()),
        }
    }
}

// ---------------------------------------------------------------------------
// Public RPC entry points
// ---------------------------------------------------------------------------

/// Invoke `SearchRepositoryV2`. Returns `None` when the response carries no
/// usable rows; the caller is responsible for falling back to local search.
pub async fn search_repository_v2(
    token: &str,
    query: &str,
    context: &RepositoryContext,
    metadata: &RepositoryIndexMetadata,
    config: &CloudConfig,
) -> SearchResults {
    let body = encode_search_repository_request(query, context, metadata, config.top_k);
    let started = Instant::now();
    let response = unary_call(
        token,
        &config.search_path,
        body,
        Duration::from_millis(config.timeout_ms),
        config,
    )
    .await;
    let elapsed_ms = started.elapsed().as_millis();
    match response {
        Ok(payload) => decode_search_payload(payload, metadata, elapsed_ms),
        Err(err) => map_transport_error(err, elapsed_ms),
    }
}

fn decode_search_payload(
    payload: Vec<u8>,
    metadata: &RepositoryIndexMetadata,
    elapsed_ms: u128,
) -> SearchResults {
    let response_bytes = payload.len();
    if payload.is_empty() {
        return SearchResults {
            source: "cloud",
            body: String::new(),
            outcome: SearchOutcome::NoResults,
            result_count: 0,
            diagnostic: CloudDiagnostic {
                outcome: Some(SearchOutcome::NoResults),
                response_bytes,
                elapsed_ms,
                ..CloudDiagnostic::default()
            },
        };
    }
    let stripped = strip_connect_unary_body(&payload);
    if let Some(error) = parse_connect_error(stripped) {
        let outcome = if error.is_codebase_not_found() {
            SearchOutcome::CodebaseNotFound
        } else {
            SearchOutcome::RpcError
        };
        return SearchResults {
            source: "cloud",
            body: String::new(),
            outcome,
            result_count: 0,
            diagnostic: CloudDiagnostic {
                outcome: Some(outcome),
                response_bytes,
                elapsed_ms,
                server_error_code: Some(error.code),
                server_error_message: Some(truncate(error.message, 200)),
                ..CloudDiagnostic::default()
            },
        };
    }
    let results = decode_search_repository_response(&payload, &metadata.path_encryption_key);
    let body = render_results(&results);
    if body.is_empty() {
        return SearchResults {
            source: "cloud",
            body,
            outcome: SearchOutcome::NoResults,
            result_count: 0,
            diagnostic: CloudDiagnostic {
                outcome: Some(SearchOutcome::NoResults),
                response_bytes,
                elapsed_ms,
                ..CloudDiagnostic::default()
            },
        };
    }
    SearchResults {
        source: "cloud",
        body,
        outcome: SearchOutcome::Ok,
        result_count: results.len(),
        diagnostic: CloudDiagnostic {
            outcome: Some(SearchOutcome::Ok),
            result_count: results.len(),
            response_bytes,
            elapsed_ms,
            ..CloudDiagnostic::default()
        },
    }
}

fn render_results(results: &[CodeResult]) -> String {
    let filtered: Vec<&CodeResult> = results
        .iter()
        .filter(|result| !result.path.is_empty() || !result.contents.trim().is_empty())
        .collect();
    if filtered.is_empty() {
        return String::new();
    }
    let mut sections: Vec<String> = Vec::new();
    for result in filtered {
        let line_part = match result.start_line {
            Some(line) if line >= 1 => format!(":{}", line),
            _ => String::new(),
        };
        let excerpt = result.contents.trim();
        let excerpt = if excerpt.len() > 1_500 {
            excerpt.chars().take(1_500).collect::<String>()
        } else if excerpt.is_empty() {
            "(no excerpt returned)".to_string()
        } else {
            excerpt.to_string()
        };
        sections.push(format!(
            "### {}{} (score {:.3})\n```\n{}\n```",
            result.path, line_part, result.score, excerpt
        ));
    }
    sections.join("\n\n")
}

fn map_transport_error(err: TransportError, elapsed_ms: u128) -> SearchResults {
    let outcome = match &err {
        TransportError::Timeout | TransportError::ConnectTimeout => SearchOutcome::Timeout,
        _ => SearchOutcome::TransportError,
    };
    SearchResults {
        source: "cloud",
        body: String::new(),
        outcome,
        result_count: 0,
        diagnostic: CloudDiagnostic {
            outcome: Some(outcome),
            elapsed_ms,
            server_error_message: Some(err.to_string()),
            ..CloudDiagnostic::default()
        },
    }
}

/// Invoke `FastRepoInitHandshakeV2`. Used when bootstrapping a new repo.
pub async fn fast_repo_init_handshake_v2(
    token: &str,
    context: &RepositoryContext,
    metadata: &RepositoryIndexMetadata,
    file_count: i32,
    root_hash: &str,
    config: &CloudConfig,
) -> Result<HandshakeResult, TransportError> {
    let body = encode_fast_repo_init_handshake_v2_request(context, metadata, file_count, root_hash);
    let payload = unary_call(
        token,
        FAST_REPO_INIT_HANDSHAKE_V2_PATH,
        body,
        Duration::from_millis(config.bootstrap_timeout_ms),
        config,
    )
    .await?;
    let response = decode_fast_repo_init_handshake_v2_response(&payload);
    Ok(HandshakeResult {
        status: response.status,
        codebases: response
            .codebases
            .into_iter()
            .map(|info| CodebaseTarget {
                codebase_id: info.codebase_id,
                status: info.status,
            })
            .collect(),
    })
}

/// Invoke `FastUpdateFileV2` for one chunk of files.
pub async fn fast_update_file_v2(
    token: &str,
    codebase_id: &str,
    metadata: &RepositoryIndexMetadata,
    files: &[UploadFile],
    config: &CloudConfig,
) -> Result<bool, TransportError> {
    if files.is_empty() {
        return Ok(true);
    }
    let body = encode_fast_update_file_v2_request(codebase_id, metadata, files);
    let payload = unary_call(
        token,
        FAST_UPDATE_FILE_V2_PATH,
        body,
        Duration::from_millis(config.bootstrap_timeout_ms),
        config,
    )
    .await?;
    if is_fast_update_file_v2_success(&payload) {
        Ok(true)
    } else {
        // Decoded status is anything other than FAST_UPDATE_STATUS_SUCCESS.
        let status = decode_fast_update_file_v2_response_status(&payload);
        if status == FAST_UPDATE_STATUS_SUCCESS {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

/// Invoke `EnsureIndexCreated`. Response body is not decoded; only
/// transport status matters.
pub async fn ensure_index_created(
    token: &str,
    context: &RepositoryContext,
    metadata: &RepositoryIndexMetadata,
    config: &CloudConfig,
) -> Result<(), TransportError> {
    let body = encode_ensure_index_created_request(context, metadata);
    let _payload = unary_call(
        token,
        ENSURE_INDEX_CREATED_PATH,
        body,
        Duration::from_millis(config.bootstrap_timeout_ms),
        config,
    )
    .await?;
    Ok(())
}

/// Invoke `FastRepoSyncComplete`. Response body is not decoded.
pub async fn fast_repo_sync_complete(
    token: &str,
    statuses: &[SyncCodebaseStatus],
    metadata: &RepositoryIndexMetadata,
    config: &CloudConfig,
) -> Result<(), TransportError> {
    let body = encode_fast_repo_sync_complete_request(statuses, metadata);
    let _payload = unary_call(
        token,
        FAST_REPO_SYNC_COMPLETE_PATH,
        body,
        Duration::from_millis(config.bootstrap_timeout_ms),
        config,
    )
    .await?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Bootstrap orchestration
// ---------------------------------------------------------------------------

/// Drive the bootstrap chain end-to-end.
///
/// 1. Walk upload candidates rooted at `workspace`.
/// 2. Compute the root hash.
/// 3. Run `FastRepoInitHandshakeV2`.
/// 4. For each non-COPY/non-UP_TO_DATE codebase, run chunked
///    `FastUpdateFileV2` calls.
/// 5. Run `EnsureIndexCreated`.
/// 6. Run `FastRepoSyncComplete` with the per-codebase outcomes.
pub async fn bootstrap_cloud_index(
    token: &str,
    workspace: &Path,
    allowlist: &[PathBuf],
    context: &RepositoryContext,
    metadata: &RepositoryIndexMetadata,
    force_upload: bool,
    config: &CloudConfig,
) -> EnsureResult {
    if !is_cloud_enabled_for_source(metadata) {
        return EnsureResult {
            outcome: EnsureOutcome::NotEnabled,
            upload_count: 0,
        };
    }
    if token.trim().is_empty() {
        return EnsureResult {
            outcome: EnsureOutcome::NoToken,
            upload_count: 0,
        };
    }
    let files = collect_upload_files(workspace, allowlist, config).await;
    if files.is_empty() {
        return EnsureResult {
            outcome: EnsureOutcome::Failed,
            upload_count: 0,
        };
    }
    let root_hash = compute_root_hash(&files);
    let handshake = match fast_repo_init_handshake_v2(
        token,
        context,
        metadata,
        files.len() as i32,
        &root_hash,
        config,
    )
    .await
    {
        Ok(value) => value,
        Err(_) => {
            return EnsureResult {
                outcome: EnsureOutcome::Failed,
                upload_count: 0,
            }
        }
    };
    if handshake.status != 2 {
        return EnsureResult {
            outcome: EnsureOutcome::Failed,
            upload_count: 0,
        };
    }
    let sync_targets: Vec<CodebaseTarget> = handshake
        .codebases
        .iter()
        .filter(|codebase| {
            !codebase.codebase_id.is_empty()
                && codebase.status != codebase_status::COPY_IN_PROGRESS
                && (force_upload || codebase.status != codebase_status::UP_TO_DATE)
        })
        .cloned()
        .collect();
    if sync_targets.is_empty() {
        let any_up_to_date = handshake
            .codebases
            .iter()
            .any(|codebase| codebase.status == codebase_status::UP_TO_DATE);
        return EnsureResult {
            outcome: if any_up_to_date {
                EnsureOutcome::AlreadyIndexed
            } else {
                EnsureOutcome::Failed
            },
            upload_count: 0,
        };
    }
    let chunks = chunk_upload_files(files, config.upload_max_batch_bytes);
    let mut sync_statuses: Vec<SyncCodebaseStatus> = Vec::new();
    let mut total_uploads = 0usize;
    for codebase in sync_targets {
        let mut failed = 0usize;
        let mut total = 0usize;
        for chunk in &chunks {
            total = total.saturating_add(chunk.len());
            match fast_update_file_v2(token, &codebase.codebase_id, metadata, chunk, config).await {
                Ok(true) => {}
                Ok(false) | Err(_) => failed = failed.saturating_add(chunk.len()),
            }
        }
        total_uploads = total_uploads.saturating_add(total);
        sync_statuses.push(SyncCodebaseStatus {
            codebase_id: codebase.codebase_id,
            success: failed == 0 && total > 0,
            total_upload_count: total as i32,
            failed_upload_count: failed as i32,
        });
    }
    if sync_statuses.iter().any(|status| !status.success) {
        return EnsureResult {
            outcome: EnsureOutcome::Failed,
            upload_count: 0,
        };
    }
    if ensure_index_created(token, context, metadata, config)
        .await
        .is_err()
    {
        return EnsureResult {
            outcome: EnsureOutcome::Failed,
            upload_count: 0,
        };
    }
    if fast_repo_sync_complete(token, &sync_statuses, metadata, config)
        .await
        .is_err()
    {
        return EnsureResult {
            outcome: EnsureOutcome::Failed,
            upload_count: 0,
        };
    }
    EnsureResult {
        outcome: EnsureOutcome::Uploaded,
        upload_count: total_uploads,
    }
}

fn is_cloud_enabled_for_source(metadata: &RepositoryIndexMetadata) -> bool {
    if metadata.workspace_uri.trim().is_empty() || metadata.path_encryption_key.trim().is_empty() {
        return false;
    }
    match metadata.source {
        Some(MetadataSource::PluginGenerated) => allow_plugin_generated_keys(),
        _ => true,
    }
}

/// Defaults to `false` for UMP per `indexing-extraction.md` opt-in env
/// contract. Honors `UMP_CURSOR_INDEX_ALLOW_PLUGIN_GENERATED_KEYS=1`.
pub fn allow_plugin_generated_keys() -> bool {
    matches!(
        std::env::var("UMP_CURSOR_INDEX_ALLOW_PLUGIN_GENERATED_KEYS")
            .ok()
            .as_deref(),
        Some("1")
    )
}

/// Build the `WorkspaceFingerprint` analogue from a `RepoMetadata`.
pub fn build_repository_context(meta: &RepoMetadata) -> RepositoryContext {
    RepositoryContext {
        relative_workspace_path: if meta.relative_workspace_path.is_empty() {
            ".".into()
        } else {
            meta.relative_workspace_path.clone()
        },
        remotes: meta
            .remote
            .as_ref()
            .map(|url| {
                vec![crate::upstream::cursor::indexing::wire::GitRemote {
                    name: meta
                        .remote_name
                        .clone()
                        .unwrap_or_else(|| "origin".to_string()),
                    url: url.clone(),
                }]
            })
            .unwrap_or_default(),
        repo_name: meta.repo_name.clone().unwrap_or_default(),
        repo_owner: meta.repo_owner.clone().unwrap_or_default(),
        is_tracked: meta.is_tracked,
        is_local: meta.is_local,
    }
}

// ---------------------------------------------------------------------------
// Upload candidate walk + filters
// ---------------------------------------------------------------------------

/// Walk upload candidates under `workspace`. Returns at most
/// `config.upload_max_files` files, each gated by the sensitive-filename
/// allowlist, ignored-dir set, max-size cap, text-extension allowlist, and
/// allowlist boundary.
pub async fn collect_upload_files(
    workspace: &Path,
    allowlist: &[PathBuf],
    config: &CloudConfig,
) -> Vec<UploadFile> {
    let canonical =
        match crate::upstream::cursor::workspace::enforce_allowlist(workspace, allowlist) {
            Ok(canonical) => canonical,
            Err(_) => return Vec::new(),
        };
    let max_files = config.upload_max_files;
    let max_bytes = config.upload_max_file_bytes;
    let candidates = list_git_candidates(&canonical).await;
    let candidates = match candidates {
        Some(values) => values,
        None => walk_fallback(&canonical, max_files),
    };
    let mut files: Vec<UploadFile> = Vec::new();
    for relative in candidates {
        if files.len() >= max_files {
            break;
        }
        if relative.is_empty() || Path::new(&relative).is_absolute() {
            continue;
        }
        if should_skip_upload_path(&relative) {
            continue;
        }
        if should_skip_upload_file(&relative) {
            continue;
        }
        let candidate = canonical.join(&relative);
        let metadata = match std::fs::symlink_metadata(&candidate) {
            Ok(value) => value,
            Err(_) => continue,
        };
        if metadata.file_type().is_symlink() || !metadata.is_file() || metadata.len() > max_bytes {
            continue;
        }
        let real_path = match std::fs::canonicalize(&candidate) {
            Ok(value) => value,
            Err(_) => continue,
        };
        if !is_within_directory(&canonical, &real_path) {
            continue;
        }
        if !is_text_file(&real_path) {
            continue;
        }
        let contents = match std::fs::read_to_string(&real_path) {
            Ok(value) => value,
            Err(_) => continue,
        };
        let mut hasher = Sha256::new();
        hasher.update(contents.as_bytes());
        let hash = hex_lower(&hasher.finalize());
        let normalized = normalize_relative_path(&canonical, &real_path);
        let ancestors = ancestor_paths(&normalized);
        files.push(UploadFile {
            relative_path: normalized,
            contents,
            hash,
            ancestor_paths: ancestors,
        });
    }
    files.sort_by(|a, b| a.relative_path.cmp(&b.relative_path));
    files
}

async fn list_git_candidates(workspace: &Path) -> Option<Vec<String>> {
    let mut command = tokio::process::Command::new("git");
    command
        .arg("-C")
        .arg(workspace)
        .args(["ls-files", "-co", "--exclude-standard"])
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GCM_INTERACTIVE", "never");
    let output = timeout(Duration::from_millis(2_000), command.output())
        .await
        .ok()?
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let mut entries: Vec<String> = String::from_utf8_lossy(&output.stdout)
        .split(['\n', '\r'])
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_owned)
        .collect();
    entries.sort();
    Some(entries)
}

fn walk_fallback(root: &Path, max: usize) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    walk_fallback_dir(root, root, max, &mut out);
    out.sort();
    out
}

fn walk_fallback_dir(root: &Path, dir: &Path, max: usize, out: &mut Vec<String>) {
    if out.len() >= max {
        return;
    }
    let entries = match std::fs::read_dir(dir) {
        Ok(value) => value,
        Err(_) => return,
    };
    let mut sorted: Vec<std::fs::DirEntry> = entries.filter_map(Result::ok).collect();
    sorted.sort_by_key(|a| a.file_name());
    for entry in sorted {
        if out.len() >= max {
            return;
        }
        let name = entry.file_name();
        let name_str = match name.to_str() {
            Some(value) => value,
            None => continue,
        };
        if DEFAULT_IGNORES.contains(&name_str) {
            continue;
        }
        let path = entry.path();
        let metadata = match entry.metadata() {
            Ok(value) => value,
            Err(_) => continue,
        };
        if metadata.file_type().is_symlink() {
            continue;
        }
        if metadata.is_dir() {
            walk_fallback_dir(root, &path, max, out);
        } else if metadata.is_file() {
            if let Ok(rel) = path.strip_prefix(root) {
                out.push(rel.to_string_lossy().into_owned());
            }
        }
    }
}

fn should_skip_upload_path(path: &str) -> bool {
    path.split(['/', '\\'])
        .any(|part| DEFAULT_IGNORES.contains(&part))
}

fn should_skip_upload_file(path: &str) -> bool {
    let basename = match Path::new(path).file_name().and_then(|name| name.to_str()) {
        Some(value) => value.to_ascii_lowercase(),
        None => return false,
    };
    if SENSITIVE_FILENAMES
        .iter()
        .any(|denied| denied.eq_ignore_ascii_case(&basename))
    {
        return true;
    }
    if let Some((_, extension)) = basename.rsplit_once('.') {
        if CERT_EXTENSION_DENY
            .iter()
            .any(|denied| denied.eq_ignore_ascii_case(extension))
        {
            return true;
        }
    }
    false
}

fn is_text_file(path: &Path) -> bool {
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|s| s.to_ascii_lowercase());
    match extension {
        Some(value) => TEXT_EXTENSIONS.iter().any(|known| *known == value),
        None => false,
    }
}

fn normalize_relative_path(root: &Path, path: &Path) -> String {
    match path.strip_prefix(root) {
        Ok(rel) => {
            let rel = rel.to_string_lossy().replace('\\', "/");
            if rel.is_empty() {
                ".".into()
            } else {
                format!("./{}", rel)
            }
        }
        Err(_) => ".".into(),
    }
}

fn ancestor_paths(relative: &str) -> Vec<String> {
    let normalized = relative.strip_prefix("./").unwrap_or(relative);
    let parent = match std::path::Path::new(normalized).parent() {
        Some(value) => value,
        None => return vec![".".into()],
    };
    let parts: Vec<&str> = parent
        .to_string_lossy()
        .split('/')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .into_iter()
        .map(|s| Box::leak(s.to_owned().into_boxed_str()) as &str)
        .collect();
    if parts.is_empty() {
        return vec![".".into()];
    }
    let mut ancestors = Vec::new();
    for index in (1..=parts.len()).rev() {
        ancestors.push(format!("./{}", parts[..index].join("/")));
    }
    ancestors.push(".".into());
    ancestors
}

fn compute_root_hash(files: &[UploadFile]) -> String {
    let mut hasher = Sha256::new();
    for file in files {
        hasher.update(file.relative_path.as_bytes());
        hasher.update(b"\0");
        hasher.update(file.hash.as_bytes());
        hasher.update(b"\n");
    }
    hex_lower(&hasher.finalize())
}

fn chunk_upload_files(files: Vec<UploadFile>, max_batch_bytes: usize) -> Vec<Vec<UploadFile>> {
    let mut chunks: Vec<Vec<UploadFile>> = Vec::new();
    let mut current: Vec<UploadFile> = Vec::new();
    let mut current_bytes = 0usize;
    for file in files {
        let bytes = file.contents.len();
        if !current.is_empty() && current_bytes + bytes > max_batch_bytes {
            chunks.push(std::mem::take(&mut current));
            current_bytes = 0;
        }
        current_bytes = current_bytes.saturating_add(bytes);
        current.push(file);
    }
    if !current.is_empty() {
        chunks.push(current);
    }
    chunks
}

fn truncate(value: String, max: usize) -> String {
    if value.len() <= max {
        return value;
    }
    value.chars().take(max).collect()
}

fn hex_lower(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push_str(&format!("{:02x}", byte));
    }
    out
}

// ---------------------------------------------------------------------------
// HTTP/2 unary helper
// ---------------------------------------------------------------------------

/// One-shot HTTP/2 unary call against `repo42.cursor.sh`.
///
/// Mirrors `transport::unary_get_usable_models` but with caller-supplied
/// path, host (always `repo42.cursor.sh` for now), Connect-unary content
/// type, and the IDE-channel header set used by the Cursor cloud index.
async fn unary_call(
    token: &str,
    path: &str,
    body: Vec<u8>,
    deadline: Duration,
    config: &CloudConfig,
) -> Result<Vec<u8>, TransportError> {
    let request_id = uuid::Uuid::new_v4().to_string();
    let (mut send_request, _guard) = connect_h2(CURSOR_REPO_HOST).await?;
    let mut request_builder = Request::builder()
        .method("POST")
        .uri(format!("https://{CURSOR_REPO_HOST}{path}"))
        .header("content-type", "application/proto")
        .header("authorization", format!("Bearer {token}"))
        .header("x-request-id", &request_id)
        .header("connect-protocol-version", "1");
    for (name, value) in cursor_repo_headers(config) {
        request_builder = request_builder.header(name, value);
    }
    let request = request_builder
        .body(())
        .map_err(|err| TransportError::Request(err.to_string()))?;
    let (response_fut, mut send_stream) = send_request
        .send_request(request, false)
        .map_err(|err| TransportError::H2(format!("failed to send repo unary request: {err}")))?;
    send_stream
        .send_data(Bytes::from(body), true)
        .map_err(|err| TransportError::H2(format!("failed to flush repo unary body: {err}")))?;
    let response = timeout(deadline, response_fut)
        .await
        .map_err(|_| TransportError::Timeout)?
        .map_err(|err| TransportError::H2(format!("repo unary response failed: {err}")))?;
    let status = response.status();
    let mut body = response.into_body();
    let mut buf: Vec<u8> = Vec::new();
    while let Some(chunk_result) = timeout(deadline, body.data())
        .await
        .map_err(|_| TransportError::Timeout)?
    {
        let chunk = chunk_result
            .map_err(|err| TransportError::H2(format!("repo unary data failed: {err}")))?;
        let chunk_len = chunk.len();
        buf.extend_from_slice(&chunk);
        let _ = body.flow_control().release_capacity(chunk_len);
    }
    if status != StatusCode::OK {
        return Err(TransportError::Upstream {
            status: status.as_u16(),
            body: format!("repo unary status {status}"),
        });
    }
    Ok(buf)
}

fn cursor_repo_headers(config: &CloudConfig) -> Vec<(String, String)> {
    vec![
        (
            "x-cursor-client-version".into(),
            if config.client_version.is_empty() {
                cursor_client_version()
            } else {
                config.client_version.clone()
            },
        ),
        ("x-cursor-client-type".into(), "ide".into()),
        ("x-cursor-client-os".into(), platform_os()),
        ("x-cursor-client-arch".into(), platform_arch()),
        ("x-cursor-client-os-version".into(), os_release()),
        ("x-cursor-client-device-type".into(), "desktop".into()),
        ("x-cursor-timezone".into(), "UTC".into()),
    ]
}

fn platform_os() -> String {
    if cfg!(target_os = "macos") {
        "darwin".into()
    } else if cfg!(target_os = "linux") {
        "linux".into()
    } else if cfg!(target_os = "windows") {
        "win32".into()
    } else {
        "unknown".into()
    }
}

fn platform_arch() -> String {
    if cfg!(target_arch = "aarch64") {
        "arm64".into()
    } else if cfg!(target_arch = "x86_64") {
        "x64".into()
    } else {
        "unknown".into()
    }
}

fn os_release() -> String {
    std::env::var("UMP_CURSOR_INDEX_OS_VERSION").unwrap_or_else(|_| "0".into())
}

struct ConnectionGuard {
    handle: JoinHandle<()>,
}

impl Drop for ConnectionGuard {
    fn drop(&mut self) {
        self.handle.abort();
    }
}

async fn connect_h2(host: &str) -> Result<(SendRequest<Bytes>, ConnectionGuard), TransportError> {
    let connect_deadline = Duration::from_secs(30);
    timeout(connect_deadline, async {
        let tcp = TcpStream::connect((host, 443)).await?;
        let connector: TlsConnector = tls_connector()?;
        let server_name = ServerName::try_from(host.to_string())
            .map_err(|err| TransportError::Tls(format!("invalid repo server name: {err}")))?;
        let tls = connector
            .connect(server_name, tcp)
            .await
            .map_err(|err| TransportError::Tls(format!("repo TLS connect failed: {err}")))?;
        let (client, connection) = h2::client::handshake(tls)
            .await
            .map_err(|err| TransportError::H2(format!("repo h2 handshake failed: {err}")))?;
        let handle = tokio::spawn(async move {
            if let Err(err) = connection.await {
                tracing::debug!(?err, "repo h2 connection ended");
            }
        });
        Ok::<_, TransportError>((client, ConnectionGuard { handle }))
    })
    .await
    .map_err(|_| TransportError::ConnectTimeout)?
}

// ---------------------------------------------------------------------------
// Misc imports: silence unused warnings on builds without bootstrap calls.
// ---------------------------------------------------------------------------

#[allow(dead_code)]
fn _unused_imports(_: Arc<()>) {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_outcome_strings_match_ts_union() {
        assert_eq!(SearchOutcome::Ok.as_str(), "ok");
        assert_eq!(
            SearchOutcome::CodebaseNotFound.as_str(),
            "codebase-not-found"
        );
        assert_eq!(SearchOutcome::TransportError.as_str(), "transport-error");
    }

    #[test]
    fn cloud_config_default_uses_repo_host() {
        let config = CloudConfig::default();
        assert_eq!(config.url, format!("https://{CURSOR_REPO_HOST}"));
        assert_eq!(config.search_path, SEARCH_REPOSITORY_V2_PATH);
    }

    #[test]
    fn should_skip_upload_path_rejects_ignored_dirs() {
        assert!(should_skip_upload_path("node_modules/foo.js"));
        assert!(should_skip_upload_path("src/.git/config"));
        assert!(!should_skip_upload_path("src/lib.rs"));
    }

    #[test]
    fn should_skip_upload_file_rejects_sensitive_filenames() {
        assert!(should_skip_upload_file("./.env"));
        assert!(should_skip_upload_file("./credentials.json"));
        assert!(should_skip_upload_file("./tls/server.pem"));
        assert!(!should_skip_upload_file("./src/lib.rs"));
    }
}
