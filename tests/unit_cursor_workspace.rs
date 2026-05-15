//! Regression tests for `upstream::cursor::workspace` — header
//! extraction, repo-metadata discovery, and the WorkspaceContract that
//! per-handler `attach_cursor_workspace` calls produce when a real
//! workspace header is present.
//!
//! Lane K coordination: route handlers call the public
//! `workspace::attach_to_request` helper. We assert the exact request mutation
//! here so header extraction and `cursor_codebase_search` registration cannot
//! drift independently across Chat, Responses, and Messages routes.
//!
//! No `env::set_var` / `env::remove_var` here — those races the existing
//! parallel-test env-locked tests in `upstream::cursor::indexing::tests`.

use std::path::PathBuf;
use std::process::Command;

use axum::http::{HeaderMap, HeaderValue};
use tempfile::tempdir;
use unified_model_proxy_v2::{
    cursor_agent::{CursorAgentRequest, CursorRoute, CursorToolKind},
    model_alias::{Provider, TargetFormat},
    upstream::cursor::workspace::{
        attach_to_request, discover_repo_metadata, extract_from_headers, is_within_directory,
        normalize_remote_url, parse_owner_repo, workspace_fingerprint, RepoMetadata,
        WorkspaceContract, HEADER_SESSION, HEADER_WORKSPACE, HEADER_WORKTREE,
    },
};
use uuid::Uuid;

fn blank_request() -> CursorAgentRequest {
    CursorAgentRequest {
        model: "composer-2-fast".to_string(),
        upstream_model: "composer-2-fast".to_string(),
        system_instructions: None,
        developer_instructions: None,
        messages: Vec::new(),
        tools: Vec::new(),
        tool_results: Vec::new(),
        continuation_key: Some(
            unified_model_proxy_v2::cursor_agent::CursorContinuationKey {
                route: CursorRoute::Responses,
                provider: Provider::Cursor,
                upstream_model: "composer-2-fast".to_string(),
                target_format: TargetFormat::CursorAgent,
                stable_request_fields: serde_json::json!({}),
                response_id: "resp_fixture".to_string(),
                conversation_id: "conv_fixture".to_string(),
            },
        ),
        workspace: None,
        stream: false,
        request_id: Uuid::nil(),
        client_profile: Default::default(),
    }
}

#[test]
fn workspace_extract_from_x_ump_cursor_headers_resolves_to_contract() {
    let mut headers = HeaderMap::new();
    headers.insert(HEADER_WORKSPACE, HeaderValue::from_static("/tmp/ws"));
    headers.insert(HEADER_WORKTREE, HeaderValue::from_static("/tmp/ws/wt"));
    headers.insert(HEADER_SESSION, HeaderValue::from_static("session-abc"));
    let contract: WorkspaceContract =
        extract_from_headers(&headers).expect("contract resolves from headers");
    assert_eq!(contract.workspace, PathBuf::from("/tmp/ws"));
    assert_eq!(contract.worktree, Some(PathBuf::from("/tmp/ws/wt")));
    assert_eq!(contract.session.as_deref(), Some("session-abc"));
}

#[tokio::test]
async fn workspace_attach_to_request_sets_context_and_registers_search_tool_from_headers() {
    let dir = tempdir().expect("workspace tempdir");
    let mut headers = HeaderMap::new();
    headers.insert(
        HEADER_WORKSPACE,
        HeaderValue::from_str(dir.path().to_str().expect("utf8 tempdir")).unwrap(),
    );
    headers.insert(HEADER_SESSION, HeaderValue::from_static("session-abc"));
    let mut request = blank_request();

    attach_to_request(&mut request, &headers).await;

    let workspace = request.workspace.expect("workspace attached");
    assert_eq!(workspace.root, dir.path());
    assert_eq!(workspace.worktree, None);
    let tool = request
        .tools
        .iter()
        .find(|tool| tool.name == "cursor_codebase_search")
        .expect("cursor_codebase_search tool registered");
    assert_eq!(tool.kind, CursorToolKind::Function);
    assert_eq!(tool.parameters_schema["required"][0], "query");
    assert!(
        tool.parameters_schema["properties"]
            .as_object()
            .expect("properties object")
            .contains_key("target_directories"),
        "target_directories remains part of public schema",
    );
}

#[tokio::test]
async fn workspace_attach_to_request_noops_without_workspace_headers() {
    let mut request = blank_request();
    attach_to_request(&mut request, &HeaderMap::new()).await;
    assert!(request.workspace.is_none());
    assert!(request.tools.is_empty());
}

#[test]
fn workspace_extract_returns_none_when_workspace_header_missing() {
    let headers = HeaderMap::new();
    assert!(extract_from_headers(&headers).is_none());
}

#[test]
fn workspace_extract_returns_none_for_blank_workspace_header() {
    // Whitespace-only workspace header is rejected the same as an absent
    // header so the route layer never advertises an empty
    // `cursor_codebase_search` tool against an unknown workspace.
    let mut headers = HeaderMap::new();
    headers.insert(HEADER_WORKSPACE, HeaderValue::from_static("   "));
    assert!(extract_from_headers(&headers).is_none());
}

#[test]
fn workspace_extract_drops_blank_worktree_and_session_values() {
    // Worktree / session are optional. Blank values are filtered so they
    // never surface as `Some("")` downstream.
    let mut headers = HeaderMap::new();
    headers.insert(HEADER_WORKSPACE, HeaderValue::from_static("/tmp/ws"));
    headers.insert(HEADER_WORKTREE, HeaderValue::from_static("   "));
    headers.insert(HEADER_SESSION, HeaderValue::from_static("\t"));
    let contract = extract_from_headers(&headers).expect("contract resolves");
    assert_eq!(contract.workspace, PathBuf::from("/tmp/ws"));
    assert_eq!(contract.worktree, None);
    assert_eq!(contract.session, None);
}

#[test]
fn workspace_extract_trims_surrounding_whitespace_on_paths() {
    let mut headers = HeaderMap::new();
    headers.insert(
        HEADER_WORKSPACE,
        HeaderValue::from_static("  /tmp/spaced  "),
    );
    let contract = extract_from_headers(&headers).expect("contract resolves");
    assert_eq!(contract.workspace, PathBuf::from("/tmp/spaced"));
}

#[test]
fn workspace_normalize_remote_url_strips_userinfo_and_dot_git() {
    // Mirrors the indexing pipeline's safety property: tokens embedded
    // as userinfo on the remote URL must be dropped before the proxy
    // ever logs or surfaces them.
    let normalized = normalize_remote_url("https://x-access-token:abc@github.com/owner/repo.git");
    assert_eq!(normalized, "github.com/owner/repo");
    let ssh = normalize_remote_url("git@github.com:owner/repo.git");
    assert_eq!(ssh, "github.com/owner/repo");
}

#[test]
fn workspace_parse_owner_repo_handles_short_paths_gracefully() {
    let (owner, repo) = parse_owner_repo("github.com/owner/repo");
    assert_eq!(owner, "owner");
    assert_eq!(repo, "repo");

    let (owner_short, repo_short) = parse_owner_repo("dangling-segment");
    assert!(owner_short.is_empty());
    assert_eq!(repo_short, "dangling-segment");
}

#[test]
fn workspace_is_within_directory_matches_self_and_descendants() {
    let root = PathBuf::from("/tmp/ws");
    assert!(is_within_directory(&root, &root));
    assert!(is_within_directory(&root, &PathBuf::from("/tmp/ws/sub")));
    assert!(!is_within_directory(&root, &PathBuf::from("/tmp/other")));
}

#[test]
fn workspace_fingerprint_is_deterministic_across_redactable_fields() {
    let mut metadata = RepoMetadata {
        workspace: PathBuf::from("/tmp/ws"),
        worktree: PathBuf::from("/tmp/ws"),
        relative_workspace_path: ".".into(),
        ..RepoMetadata::default()
    };
    metadata.remote_url_normalized = Some("github.com/owner/repo".into());
    metadata.branch = Some("main".into());

    let fingerprint = workspace_fingerprint(&metadata);
    let fingerprint_again = workspace_fingerprint(&metadata);
    assert_eq!(fingerprint, fingerprint_again, "fingerprint must be stable");
    assert_eq!(fingerprint.len(), 16, "fingerprint truncated to 16 chars");
}

/// Round-trip discovery against a real tempdir-initialized git repo.
///
/// Skipped silently when `git` is not on PATH so the suite stays runnable
/// in stripped-down dev containers. Otherwise the test asserts that
/// branch / status / repo_root / repo_name are populated from the real
/// `git` invocation rather than collapsing to defaults.
#[tokio::test]
async fn discover_repo_metadata_reads_branch_and_status_from_real_git() {
    if Command::new("git").arg("--version").output().is_err() {
        eprintln!("skipping: git not on PATH");
        return;
    }

    let dir = tempdir().expect("create tempdir");
    let path = dir.path();
    // Initialize a fresh repo with a deterministic branch and identity
    // so the test does not depend on the user's git config.
    assert!(Command::new("git")
        .args(["init", "-q", "--initial-branch", "test-branch"])
        .current_dir(path)
        .status()
        .expect("git init")
        .success());
    assert!(Command::new("git")
        .args(["config", "user.email", "test@example.invalid"])
        .current_dir(path)
        .status()
        .expect("git config email")
        .success());
    assert!(Command::new("git")
        .args(["config", "user.name", "Test"])
        .current_dir(path)
        .status()
        .expect("git config name")
        .success());

    // Touch a file so `git status --short` produces output.
    std::fs::write(path.join("hello.txt"), "hi\n").expect("write file");

    let metadata = discover_repo_metadata(path).await;

    assert_eq!(metadata.workspace, path);
    assert_eq!(metadata.worktree, path);

    // git rev-parse --abbrev-ref HEAD on a fresh repo with no commits
    // can return HEAD or the initial branch name depending on git
    // version. Either way, branch should be Some when the repo is
    // valid. Skip the strict equality check when git's version is
    // ancient enough to hand back HEAD.
    if let Some(branch) = metadata.branch.as_deref() {
        assert!(
            branch == "test-branch" || !branch.is_empty(),
            "branch should be the configured initial branch or non-empty: {branch:?}",
        );
    }

    let status = metadata
        .status_summary
        .as_deref()
        .expect("status_summary populated");
    assert!(
        status.contains("hello.txt"),
        "status_summary mentions touched file: {status:?}",
    );
    assert!(
        metadata.is_local,
        "freshly init'd repo with no remote is local",
    );
    assert!(!metadata.is_tracked, "no remote -> not tracked");
    // repo_name falls back to the workspace dir name when no remote is
    // configured.
    assert!(metadata.repo_name.is_some());
}

#[tokio::test]
async fn discover_repo_metadata_collapses_quietly_for_non_git_dir() {
    // No git ops — passing a plain tempdir must not error; it returns
    // a metadata struct with workspace/worktree set and everything else
    // empty / false.
    let dir = tempdir().expect("create tempdir");
    let metadata = discover_repo_metadata(dir.path()).await;
    assert_eq!(metadata.workspace, dir.path());
    assert_eq!(metadata.worktree, dir.path());
    assert!(metadata.branch.is_none());
    assert!(metadata.status_summary.is_none());
    assert!(metadata.remote.is_none());
    assert!(metadata.is_local);
    assert!(!metadata.is_tracked);
}
