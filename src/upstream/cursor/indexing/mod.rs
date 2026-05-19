//! Cursor indexing module entry point.
//!
//! Three sub-modules:
//! - `wire` — protobuf encoders/decoders for the five RepositoryService
//!   RPCs plus the per-segment AES-256-CTR path cipher.
//! - `cloud` — `repo42.cursor.sh` adapter (search, handshake, upload,
//!   ensure, sync-complete).
//! - `local` — bounded local lexical fallback used when the cloud layer
//!   is disabled or yields nothing.
//!
//! Three opt-in env gates govern observable behavior:
//!
//! - `UMP_CURSOR_INDEX_CLOUD=1` enables `SearchRepositoryV2` traffic.
//!   Default OFF: no `repo42.cursor.sh` traffic at all when unset.
//! - `UMP_CURSOR_INDEX_BOOTSTRAP=1` enables the upload chain (handshake +
//!   `FastUpdateFileV2` + ensure + sync-complete). Default OFF.
//! - `UMP_CURSOR_INDEX_ALLOW_PLUGIN_GENERATED_KEYS=1` permits cloud calls
//!   against plugin-generated path keys. Default OFF (TS defaults ON; UMP
//!   does not trust plugin-generated keys).
//!
//! A failure in indexing never fails normal chat. Callers treat the
//! returned hits as best-effort context.

use std::{
    fs,
    path::{Path, PathBuf},
};

use serde_json::Value;

pub mod cloud;
pub mod local;
pub mod wire;

pub use local::{SearchHit, EMPTY_RESULT_SENTINEL};

/// Env knob: enable cloud `SearchRepositoryV2` traffic. Default OFF.
pub const ENV_INDEX_CLOUD: &str = "UMP_CURSOR_INDEX_CLOUD";

/// Env knob: enable bootstrap (handshake + upload + ensure + sync). Default OFF.
pub const ENV_INDEX_BOOTSTRAP: &str = "UMP_CURSOR_INDEX_BOOTSTRAP";
pub const ENV_INDEX_METADATA_JSON: &str = "UMP_CURSOR_INDEX_METADATA_JSON";
pub const ENV_INDEX_METADATA_FILE: &str = "UMP_CURSOR_INDEX_METADATA_FILE";

/// Diagnostic record emitted alongside a search call. Caller is
/// responsible for redacting paths/tokens before logging.
#[derive(Debug, Clone, Default)]
pub struct IndexDiagnostic {
    pub mode: IndexMode,
    pub outcome: Option<cloud::SearchOutcome>,
    pub result_count: usize,
    pub elapsed_ms: u128,
    pub server_error_code: Option<String>,
    pub server_error_message: Option<String>,
    pub bootstrap_attempted: bool,
}

/// Index path taken for a given request.
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq)]
pub enum IndexMode {
    /// Cursor cloud RPC returned a non-empty body.
    Cloud,
    /// Cloud disabled or fell through; local lexical fallback was used.
    #[default]
    LocalFallback,
    /// Indexing was disabled (e.g. workspace missing, allowlist failed,
    /// metadata source rejected).
    Unavailable,
}

impl IndexMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Cloud => "cloud",
            Self::LocalFallback => "local-fallback",
            Self::Unavailable => "unavailable",
        }
    }
}

/// Resolve the cloud-search opt-in env. Anything other than `"1"` returns
/// `false`.
pub fn cloud_enabled_env() -> bool {
    matches!(std::env::var(ENV_INDEX_CLOUD).ok().as_deref(), Some("1"))
}

/// Resolve the bootstrap opt-in env. Anything other than `"1"` returns
/// `false`.
pub fn bootstrap_enabled_env() -> bool {
    matches!(
        std::env::var(ENV_INDEX_BOOTSTRAP).ok().as_deref(),
        Some("1")
    )
}

/// Run a workspace search end-to-end.
///
/// Behavior:
/// - When `UMP_CURSOR_INDEX_CLOUD=1` and `metadata` is present, attempt
///   cloud first; fall back to local on any non-Ok outcome.
/// - Otherwise run local search only.
/// - Indexing failure never propagates as an error; the function returns
///   an empty `Vec` instead.
pub async fn search(
    token: &str,
    workspace: &Path,
    query: &str,
    allowlist: &[PathBuf],
) -> Vec<SearchHit> {
    if !cloud_enabled_env() {
        return local::local_search(workspace, query, allowlist).await;
    }
    // Token gate. If we're cloud-enabled but missing a token we simply
    // fall back to local so the caller sees consistent behavior whether
    // the JWT path is broken or the env is unset.
    if token.trim().is_empty() {
        return local::local_search(workspace, query, allowlist).await;
    }
    if let Some(metadata) = resolve_index_metadata() {
        let repo_meta = crate::upstream::cursor::workspace::discover_repo_metadata(workspace).await;
        let context = cloud::build_repository_context(&repo_meta);
        let config = cloud::CloudConfig::default();
        if bootstrap_enabled_env() {
            let _ = cloud::bootstrap_cloud_index(
                token, workspace, allowlist, &context, &metadata, false, &config,
            )
            .await;
        }
        let results = cloud::search_repository_v2(token, query, &context, &metadata, &config).await;
        if results.outcome == cloud::SearchOutcome::Ok && !results.body.trim().is_empty() {
            return vec![SearchHit {
                path: "cursor-cloud-index".into(),
                score: results.result_count as u32,
                excerpt: results.body,
            }];
        }
    }
    local::local_search(workspace, query, allowlist).await
}

pub fn resolve_index_metadata_value() -> Option<Value> {
    if let Ok(raw) = std::env::var(ENV_INDEX_METADATA_JSON) {
        let trimmed = raw.trim();
        if !trimmed.is_empty() {
            if let Ok(value) = serde_json::from_str::<Value>(trimmed) {
                return Some(value);
            }
        }
    }
    if let Ok(path) = std::env::var(ENV_INDEX_METADATA_FILE) {
        let trimmed = path.trim();
        if !trimmed.is_empty() {
            if let Ok(raw) = fs::read_to_string(trimmed) {
                if let Ok(value) = serde_json::from_str::<Value>(&raw) {
                    return Some(value);
                }
            }
        }
    }
    None
}

fn resolve_index_metadata() -> Option<wire::RepositoryIndexMetadata> {
    let value = resolve_index_metadata_value()?;
    Some(wire::RepositoryIndexMetadata {
        workspace_uri: string_field(&value, &["workspaceUri", "workspace_uri"])?,
        path_encryption_key: string_field(&value, &["pathEncryptionKey", "path_encryption_key"])?,
        orthogonal_transform_seed: number_field(
            &value,
            &["orthogonalTransformSeed", "orthogonal_transform_seed"],
        ),
        repo_name: string_field(&value, &["repoName", "repo_name"]),
        repo_owner: string_field(&value, &["repoOwner", "repo_owner"]),
        source: Some(if std::env::var(ENV_INDEX_METADATA_FILE).is_ok() {
            wire::MetadataSource::EnvFile
        } else {
            wire::MetadataSource::EnvJson
        }),
    })
}

fn string_field(value: &Value, names: &[&str]) -> Option<String> {
    names
        .iter()
        .find_map(|name| value.get(*name).and_then(Value::as_str))
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
}

fn number_field(value: &Value, names: &[&str]) -> Option<f64> {
    names
        .iter()
        .find_map(|name| value.get(*name).and_then(Value::as_f64))
}

/// Compose a render body for the search response. Empty hits emit the
/// `EMPTY_RESULT_SENTINEL` substring so the context-injection layer
/// suppresses the block (per `context-injection.ts:198-200`).
pub fn render_body(hits: &[SearchHit]) -> String {
    local::render_local_body(hits)
}

/// Encode an `ExecClientMessage` carrying an MCP success result for
/// `cursor_codebase_search`. The body is wrapped as
/// `McpResult.success.content[0].text.text` per `agent.v1.McpResult`.
///
/// Returned bytes are the inner protobuf body for an `AgentClientMessage`;
/// the caller is responsible for Connect framing via
/// `connect::frame_connect_message`.
pub fn encode_cursor_codebase_search_mcp_result(
    exec_id: u64,
    cursor_exec_id: &str,
    body: &str,
) -> Vec<u8> {
    use crate::upstream::cursor::proto::{
        agent_client_message, concat_bytes, encode_bool_field, encode_int64_field,
        encode_message_field, encode_string_field, exec_message,
    };

    // McpTextContent { text = 1 }
    let text_content = encode_string_field(1, body);
    // McpToolResultContentItem { content = oneof { text = 1 } }
    let content_item = encode_message_field(1, &text_content);
    // McpSuccess { content = repeated 1, is_error = 2 }
    let success = concat_bytes(&[
        encode_message_field(1, &content_item),
        encode_bool_field(2, false),
    ]);
    // McpResult { result = oneof { success = 1 } }
    let mcp_result = encode_message_field(1, &success);
    // ExecClientMessage { id = 1, mcp_args = 11 (oneof), exec_id = 15 }
    // Re-uses MCP_ARGS field number for the result side per the
    // ExecClient/ExecServer parallel oneof tags.
    let exec_client = concat_bytes(&[
        encode_int64_field(exec_message::ID, exec_id),
        encode_string_field(exec_message::EXEC_ID, cursor_exec_id),
        encode_message_field(exec_message::MCP_ARGS, &mcp_result),
    ]);
    encode_message_field(agent_client_message::EXEC_CLIENT_MESSAGE, &exec_client)
}

/// Macro-friendly summary used by `render_body` consumers and the
/// system-prompt block.
#[derive(Debug, Clone, Default)]
pub struct RenderedSearch {
    pub mode: IndexMode,
    pub source: &'static str,
    pub body: String,
    pub diagnostic: IndexDiagnostic,
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
        fn set(key: &'static str, value: &str) -> Self {
            let previous = std::env::var_os(key);
            std::env::set_var(key, value);
            Self { key, previous }
        }

        fn unset(key: &'static str) -> Self {
            let previous = std::env::var_os(key);
            std::env::remove_var(key);
            Self { key, previous }
        }
    }

    impl Drop for EnvRestore {
        fn drop(&mut self) {
            match &self.previous {
                Some(value) => std::env::set_var(self.key, value),
                None => std::env::remove_var(self.key),
            }
        }
    }

    #[test]
    fn cloud_enabled_env_defaults_off() {
        let _guard = crate::TEST_ENV_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let _restore = EnvRestore::unset(ENV_INDEX_CLOUD);
        assert!(!cloud_enabled_env());
    }

    #[test]
    fn bootstrap_enabled_env_requires_explicit_one() {
        let _guard = crate::TEST_ENV_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let _restore = EnvRestore::set(ENV_INDEX_BOOTSTRAP, "true");
        assert!(!bootstrap_enabled_env());
        std::env::set_var(ENV_INDEX_BOOTSTRAP, "1");
        assert!(bootstrap_enabled_env());
    }

    #[test]
    fn index_mode_strings_match_diagnostic_alphabet() {
        assert_eq!(IndexMode::Cloud.as_str(), "cloud");
        assert_eq!(IndexMode::LocalFallback.as_str(), "local-fallback");
        assert_eq!(IndexMode::Unavailable.as_str(), "unavailable");
    }
}
