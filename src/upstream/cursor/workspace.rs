//! Workspace contract and repo-metadata discovery for Cursor indexing.
//!
//! Two layers:
//!
//! - `WorkspaceContract` is the route-extraction shape. Routes hand it to the
//!   indexing layer either via the `x-ump-cursor-*` headers or via the
//!   `UMP_CURSOR_WORKSPACE_DIR` + `UMP_CURSOR_WORKSPACE_ALLOWLIST` env
//!   fallback. There is no `process.cwd()` fallback (see ralplan-cursor-
//!   composer-e2e §8 step 1, and indexing-extraction.md "Workspace input
//!   contract for Rust port").
//! - `RepoMetadata` is the populated git/discovery shape that the cloud and
//!   local search routines consume. It is built by `discover_repo_metadata`
//!   off a workspace path that already passed the allowlist check.
//!
//! The allowlist is enforced via `enforce_allowlist`. Every workspace,
//! worktree, file read, and upload candidate must canonicalize via
//! `std::fs::canonicalize` (realpath equivalent) to a descendant of one of
//! the configured roots; symlink escapes are rejected.

use std::path::{Path, PathBuf};
use std::time::Duration;

use axum::http::HeaderMap;
use tokio::process::Command;
use tokio::time::timeout;

use crate::{
    cursor_agent::{CursorAgentRequest, CursorTool, CursorToolKind, CursorWorkspaceContext},
    AppError,
};

/// Header name carrying the absolute repo root.
pub const HEADER_WORKSPACE: &str = "x-ump-cursor-workspace";
/// Header name carrying the absolute worktree path. Defaults to workspace.
pub const HEADER_WORKTREE: &str = "x-ump-cursor-worktree";
/// Header name carrying the opaque caller-owned correlation id.
pub const HEADER_SESSION: &str = "x-ump-cursor-session";

/// Sole non-header workspace fallback environment variable.
pub const ENV_WORKSPACE_DIR: &str = "UMP_CURSOR_WORKSPACE_DIR";
/// Allowlist environment variable. Colon-separated absolute paths.
pub const ENV_ALLOWLIST: &str = "UMP_CURSOR_WORKSPACE_ALLOWLIST";

/// Hard timeout for shelling out to git plumbing.
const GIT_TIMEOUT: Duration = Duration::from_millis(1_000);

/// Workspace input contract extracted at the route boundary.
///
/// `workspace` is the absolute repo root, `worktree` defaults to the same
/// path when the route omits the worktree header, `session` is opaque
/// caller-owned, and `allowlist` is the per-request allowlist sourced from
/// `UMP_CURSOR_WORKSPACE_ALLOWLIST`.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct WorkspaceContract {
    pub workspace: PathBuf,
    pub worktree: Option<PathBuf>,
    pub session: Option<String>,
    pub allowlist: Vec<PathBuf>,
}

/// Repository metadata produced by `discover_repo_metadata`.
///
/// All fields are best-effort; failures collapse to `None` rather than
/// surfacing an error so a partially indexed repo (e.g. no remote) keeps
/// working. The fingerprint is the redacted log analogue of the TS
/// `cursorIndexFingerprint`.
#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub struct RepoMetadata {
    pub workspace: PathBuf,
    pub worktree: PathBuf,
    pub relative_workspace_path: String,
    pub repo_root: Option<PathBuf>,
    pub branch: Option<String>,
    pub remote: Option<String>,
    pub remote_url_normalized: Option<String>,
    pub remote_name: Option<String>,
    pub repo_owner: Option<String>,
    pub repo_name: Option<String>,
    pub status_summary: Option<String>,
    pub is_tracked: bool,
    pub is_local: bool,
}

/// Extract a workspace contract from request headers. Returns `None` when
/// no `x-ump-cursor-workspace` header is present.
pub fn extract_from_headers(headers: &HeaderMap) -> Option<WorkspaceContract> {
    let workspace_raw = headers.get(HEADER_WORKSPACE)?.to_str().ok()?.trim();
    if workspace_raw.is_empty() {
        return None;
    }
    let workspace = PathBuf::from(workspace_raw);
    let worktree = headers
        .get(HEADER_WORKTREE)
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from);
    let session = headers
        .get(HEADER_SESSION)
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned);
    Some(WorkspaceContract {
        workspace,
        worktree,
        session,
        allowlist: read_allowlist_env(),
    })
}

/// Build a workspace contract from environment variables.
///
/// Returns `None` when `UMP_CURSOR_WORKSPACE_DIR` is unset/empty. NO
/// `process.cwd()` fallback (see indexing-extraction.md "Workspace input
/// contract for Rust port" item 4).
pub fn fallback_from_env() -> Option<WorkspaceContract> {
    let raw = std::env::var(ENV_WORKSPACE_DIR).ok()?;
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    Some(WorkspaceContract {
        workspace: PathBuf::from(trimmed),
        worktree: None,
        session: None,
        allowlist: read_allowlist_env(),
    })
}

/// Attach workspace context and the public `cursor_codebase_search` tool to a
/// Cursor agent request when headers or env provide a workspace contract.
pub async fn attach_to_request(request: &mut CursorAgentRequest, headers: &HeaderMap) {
    let Some(contract) = extract_from_headers(headers).or_else(fallback_from_env) else {
        return;
    };
    let repo_meta = discover_repo_metadata(&contract.workspace).await;
    request.workspace = Some(CursorWorkspaceContext {
        root: contract.workspace.clone(),
        worktree: contract.worktree.clone(),
        branch: repo_meta.branch.clone(),
        remote: repo_meta.remote.clone(),
        status_summary: repo_meta.status_summary.clone(),
        index_metadata: crate::upstream::cursor::indexing::resolve_index_metadata_value(),
        allowlist: contract.allowlist.clone(),
    });
    request.tools.push(CursorTool {
        name: "cursor_codebase_search".to_string(),
        description: Some(
            "Search the current workspace codebase using the Cursor index. \
             Returns ranked snippets of relevant code. When Cursor cloud index \
             RPCs are unavailable, returns a clearly-labeled local fallback result."
                .to_string(),
        ),
        parameters_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "Natural-language or keyword query for relevant code."
                },
                "target_directories": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Optional directories to limit the search."
                },
                "explanation": {
                    "type": "string",
                    "description": "Why this search is needed."
                }
            },
            "required": ["query"]
        }),
        kind: CursorToolKind::Function,
    });
}

/// Read the allowlist environment variable as a list of absolute paths.
///
/// The TS plugin uses comma-or-path-separator semantics; the Rust port
/// honors both colon (POSIX path-separator) and comma to keep operator
/// muscle memory portable.
pub fn read_allowlist_env() -> Vec<PathBuf> {
    std::env::var(ENV_ALLOWLIST)
        .ok()
        .map(|raw| {
            raw.split([':', ','])
                .map(str::trim)
                .filter(|entry| !entry.is_empty())
                .map(PathBuf::from)
                .collect()
        })
        .unwrap_or_default()
}

/// Reject a path that is not a canonical descendant of any allowlist entry.
///
/// Both `path` and the allowlist entries are passed through
/// `std::fs::canonicalize` (the standard-library equivalent of `realpath`).
/// A symlink that escapes any allowlist root is rejected. Returns the
/// canonicalized path on success.
pub fn enforce_allowlist(path: &Path, allowlist: &[PathBuf]) -> Result<PathBuf, AppError> {
    if allowlist.is_empty() {
        return Err(AppError::BadRequest(
            "cursor workspace allowlist is empty (set UMP_CURSOR_WORKSPACE_ALLOWLIST)".into(),
        ));
    }
    let canonical = std::fs::canonicalize(path).map_err(|err| {
        AppError::BadRequest(format!("cursor workspace path is not accessible: {err}"))
    })?;
    for entry in allowlist {
        let canonical_entry = match std::fs::canonicalize(entry) {
            Ok(value) => value,
            Err(_) => continue,
        };
        if canonical == canonical_entry || canonical.starts_with(&canonical_entry) {
            return Ok(canonical);
        }
    }
    Err(AppError::BadRequest(
        "cursor workspace path is outside the configured allowlist".into(),
    ))
}

/// Parallel of TS `isWithinDirectory`: return `true` when `candidate` lies
/// inside `root`. Both arguments must already be canonicalized.
pub fn is_within_directory(root: &Path, candidate: &Path) -> bool {
    candidate == root || candidate.starts_with(root)
}

/// Discover repo metadata via shell-out to `git`. Errors collapse to
/// `None` per field; the function does not propagate failures because
/// indexing is best-effort.
pub async fn discover_repo_metadata(workspace: &Path) -> RepoMetadata {
    let mut metadata = RepoMetadata {
        workspace: workspace.to_path_buf(),
        worktree: workspace.to_path_buf(),
        relative_workspace_path: ".".into(),
        ..RepoMetadata::default()
    };

    let repo_root = git_capture(workspace, &["rev-parse", "--show-toplevel"]).await;
    if let Some(root) = repo_root.as_ref().filter(|value| !value.is_empty()) {
        metadata.repo_root = Some(PathBuf::from(root));
    }
    metadata.branch = git_capture(workspace, &["rev-parse", "--abbrev-ref", "HEAD"])
        .await
        .filter(|value| !value.is_empty() && value != "HEAD");
    if metadata.branch.is_none() {
        metadata.branch = git_capture(workspace, &["rev-parse", "--short", "HEAD"])
            .await
            .filter(|value| !value.is_empty());
    }
    let remote = git_capture(workspace, &["remote", "get-url", "origin"])
        .await
        .filter(|value| !value.is_empty());
    if let Some(remote_url) = remote.as_ref() {
        metadata.remote = Some(remote_url.clone());
        metadata.remote_name = Some("origin".into());
        let normalized = normalize_remote_url(remote_url);
        if !normalized.is_empty() {
            metadata.remote_url_normalized = Some(normalized.clone());
            let (owner, repo_name) = parse_owner_repo(&normalized);
            if !owner.is_empty() {
                metadata.repo_owner = Some(owner);
            }
            if !repo_name.is_empty() {
                metadata.repo_name = Some(repo_name);
            }
        }
    }
    metadata.status_summary = git_capture(workspace, &["status", "--short"])
        .await
        .filter(|value| !value.is_empty());

    metadata.is_tracked = metadata.remote.is_some();
    metadata.is_local = !metadata.is_tracked;
    if metadata.repo_name.is_none() {
        metadata.repo_name = workspace
            .file_name()
            .and_then(|name| name.to_str())
            .map(str::to_owned);
    }
    metadata
}

/// Run `git <args>` inside `workspace`, returning the trimmed stdout on
/// success or `None` on any failure (timeout, non-zero exit, spawn error).
async fn git_capture(workspace: &Path, args: &[&str]) -> Option<String> {
    let mut command = Command::new("git");
    command
        .arg("-C")
        .arg(workspace)
        .args(args)
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GCM_INTERACTIVE", "never");
    let output = timeout(GIT_TIMEOUT, command.output()).await.ok()?.ok()?;
    if !output.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Strip userinfo and `.git` suffixes from a remote URL.
///
/// Mirrors TS `normalizeRemoteUrl` (`workspace-context.ts:73-86`). Tokens
/// embedded as userinfo (`https://x-access-token:abc@github.com/...`) are
/// dropped so the remote can be safely surfaced in diagnostics. Hand-rolled
/// to avoid dragging the `url` crate into the direct dependency graph.
pub fn normalize_remote_url(url: &str) -> String {
    let trimmed = url.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    if let Some(captured) = parse_ssh_remote(trimmed) {
        return strip_dot_git(&captured);
    }
    let after_scheme = match trimmed.find("://") {
        Some(idx) => &trimmed[idx + 3..],
        None => return strip_dot_git(trimmed),
    };
    // Strip userinfo (everything before the last '@' that precedes the host).
    let host_path = match after_scheme.split_once('@') {
        Some((_userinfo, rest)) => rest,
        None => after_scheme,
    };
    let no_query = host_path
        .split_once('?')
        .map(|(prefix, _)| prefix)
        .unwrap_or(host_path);
    let no_fragment = no_query
        .split_once('#')
        .map(|(prefix, _)| prefix)
        .unwrap_or(no_query);
    strip_dot_git(no_fragment.trim_end_matches('/'))
}

fn parse_ssh_remote(value: &str) -> Option<String> {
    // git@host:owner/repo or ssh://git@host/owner/repo
    let stripped = value
        .strip_prefix("ssh://")
        .unwrap_or(value)
        .strip_prefix("git@")?;
    let (host, rest) = stripped.split_once([':', '/'])?;
    if rest.is_empty() {
        return None;
    }
    Some(format!("{host}/{rest}"))
}

fn strip_dot_git(value: &str) -> String {
    value
        .strip_suffix(".git")
        .map(str::to_owned)
        .unwrap_or_else(|| value.to_string())
}

/// Pull `(owner, repo)` out of a normalized remote URL. Falls back to the
/// trailing path segment when the path is too short to carry an owner.
pub fn parse_owner_repo(normalized: &str) -> (String, String) {
    let parts: Vec<&str> = normalized.split('/').filter(|s| !s.is_empty()).collect();
    if parts.len() < 3 {
        let last = parts.last().copied().unwrap_or("");
        return (String::new(), strip_dot_git(last));
    }
    let owner = parts[parts.len() - 2].to_string();
    let repo = parts[parts.len() - 1].to_string();
    (owner, strip_dot_git(&repo))
}

/// Stable diagnostic slug for a workspace. SHA-256 over `workspace ||
/// remote_url_normalized || branch`, truncated to 16 hex chars. Mirrors TS
/// `cursorIndexFingerprint`.
pub fn workspace_fingerprint(metadata: &RepoMetadata) -> String {
    use sha2::{Digest, Sha256};

    let mut hasher = Sha256::new();
    hasher.update(metadata.workspace.to_string_lossy().as_bytes());
    if let Some(remote) = metadata.remote_url_normalized.as_deref() {
        hasher.update(remote.as_bytes());
    }
    if let Some(branch) = metadata.branch.as_deref() {
        hasher.update(branch.as_bytes());
    }
    hex_truncate(&hasher.finalize(), 16)
}

fn hex_truncate(bytes: &[u8], chars: usize) -> String {
    let mut out = String::with_capacity(chars);
    for byte in bytes {
        if out.len() >= chars {
            break;
        }
        out.push_str(&format!("{:02x}", byte));
    }
    out.truncate(chars);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;

    #[test]
    fn extract_from_headers_reads_workspace_and_session() {
        let mut headers = HeaderMap::new();
        headers.insert(HEADER_WORKSPACE, HeaderValue::from_static("/tmp/repo"));
        headers.insert(HEADER_WORKTREE, HeaderValue::from_static("/tmp/repo/wt"));
        headers.insert(HEADER_SESSION, HeaderValue::from_static("sess-1"));
        let extracted = extract_from_headers(&headers).expect("contract");
        assert_eq!(extracted.workspace, PathBuf::from("/tmp/repo"));
        assert_eq!(extracted.worktree, Some(PathBuf::from("/tmp/repo/wt")));
        assert_eq!(extracted.session.as_deref(), Some("sess-1"));
    }

    #[test]
    fn extract_returns_none_without_workspace_header() {
        let headers = HeaderMap::new();
        assert!(extract_from_headers(&headers).is_none());
    }

    #[test]
    fn normalize_remote_strips_userinfo_and_dot_git() {
        let normalized =
            normalize_remote_url("https://x-access-token:abc@github.com/owner/repo.git");
        assert_eq!(normalized, "github.com/owner/repo");
    }

    #[test]
    fn normalize_remote_handles_ssh_form() {
        let normalized = normalize_remote_url("git@github.com:owner/repo.git");
        assert_eq!(normalized, "github.com/owner/repo");
    }

    #[test]
    fn parse_owner_repo_returns_owner_and_repo() {
        let (owner, repo) = parse_owner_repo("github.com/octo/widgets");
        assert_eq!(owner, "octo");
        assert_eq!(repo, "widgets");
    }
}
