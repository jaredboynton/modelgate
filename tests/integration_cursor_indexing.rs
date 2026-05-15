//! Cursor workspace + cloud indexing tests.

use tempfile::TempDir;
use unified_model_proxy_v2::upstream::cursor::{indexing, workspace};

const LOCAL_FALLBACK_SENTINEL: &str = "found no matching files";

struct EnvVarGuard {
    key: &'static str,
    previous: Option<std::ffi::OsString>,
}

impl EnvVarGuard {
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

impl Drop for EnvVarGuard {
    fn drop(&mut self) {
        match &self.previous {
            Some(value) => std::env::set_var(self.key, value),
            None => std::env::remove_var(self.key),
        }
    }
}

fn fresh_tempdir(name: &str) -> TempDir {
    tempfile::Builder::new()
        .prefix(&format!("cursor-indexing-{name}-"))
        .tempdir()
        .expect("create indexing temp dir")
}

#[tokio::test]
async fn workspace_allowlist_rejects_symlink_escape() {
    let workspace = fresh_tempdir("ws");
    let outside = fresh_tempdir("outside");
    let symlink_target = outside.path().join("secret.txt");
    std::fs::write(&symlink_target, "do not leak").unwrap();
    let symlink_inside_ws = workspace.path().join("escape");
    #[cfg(unix)]
    std::os::unix::fs::symlink(&symlink_target, &symlink_inside_ws)
        .expect("create symlink for escape test");
    std::fs::write(workspace.path().join("safe.rs"), "fn main() {}").unwrap();

    let allowlist = vec![workspace.path().to_path_buf()];
    assert!(workspace::enforce_allowlist(&workspace.path().join("safe.rs"), &allowlist).is_ok());
    assert!(workspace::enforce_allowlist(&symlink_inside_ws, &allowlist).is_err());
}

#[tokio::test]
async fn local_fallback_emits_found_no_matching_files_sentinel() {
    let workspace = fresh_tempdir("empty-ws");
    let hits = indexing::local::local_search(
        workspace.path(),
        "nothing-here",
        &[workspace.path().to_path_buf()],
    )
    .await;
    let body = indexing::render_body(&hits);
    assert!(body.contains(LOCAL_FALLBACK_SENTINEL), "{body}");
}

#[test]
fn metadata_json_env_is_attached_to_workspace_context() {
    let _guard = EnvVarGuard::set(
        indexing::ENV_INDEX_METADATA_JSON,
        r#"{"workspaceUri":"file:///tmp/ws","pathEncryptionKey":"abc","repoName":"repo","repoOwner":"owner"}"#,
    );
    let metadata = indexing::resolve_index_metadata_value().expect("metadata json");
    assert_eq!(metadata["workspaceUri"], "file:///tmp/ws");
    assert_eq!(metadata["pathEncryptionKey"], "abc");
}

#[test]
fn cloud_indexing_disabled_by_default_no_cloud_mode() {
    let _guard = EnvVarGuard::unset(indexing::ENV_INDEX_CLOUD);
    assert!(!indexing::cloud_enabled_env());
}

#[tokio::test]
async fn cursor_codebase_search_tool_is_registered_when_workspace_present() {
    let workspace = fresh_tempdir("registered");
    let _workspace_guard = EnvVarGuard::set(
        workspace::ENV_WORKSPACE_DIR,
        workspace.path().to_str().unwrap(),
    );
    let _allow_guard =
        EnvVarGuard::set(workspace::ENV_ALLOWLIST, workspace.path().to_str().unwrap());
    let mut request = unified_model_proxy_v2::cursor_agent::CursorAgentRequest {
        model: "composer-2-fast".into(),
        upstream_model: "composer-2-fast".into(),
        system_instructions: None,
        developer_instructions: None,
        messages: Vec::new(),
        tools: Vec::new(),
        tool_results: Vec::new(),
        continuation_key: None,
        workspace: None,
        stream: false,
        request_id: uuid::Uuid::nil(),
    };

    workspace::attach_to_request(&mut request, &axum::http::HeaderMap::new()).await;
    assert!(request.workspace.is_some());
    assert!(request
        .tools
        .iter()
        .any(|tool| tool.name == "cursor_codebase_search"));
}
