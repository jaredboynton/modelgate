#![cfg(feature = "mcp")]

use serde_json::{json, Value};
use unified_model_proxy_v2::fast_context::{
    cascade_wire::{
        build_initial_payload, build_swe_grep_request, FAST_CONTEXT_MODEL, FAST_CONTEXT_SENTINEL,
        FAST_CONTEXT_TOOL_NAME, SWE_GREP_MINI_MODEL, SWE_GREP_MODEL,
    },
    mcp::handle_request,
    run_fast_context, ExecutionMode, FastContextModel, FastContextRequest,
};

#[tokio::test]
async fn mcp_lists_fast_context_tool() {
    let response = handle_request(json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/list"
    }))
    .await
    .unwrap();

    assert_eq!(response["result"]["tools"][0]["name"], "fast_context");
    assert_eq!(
        response["result"]["tools"][0]["inputSchema"]["required"],
        json!(["search_string", "repo_path"])
    );
    assert_eq!(
        response["result"]["tools"][0]["inputSchema"]["properties"]["model"]["enum"],
        json!(["both", "swe-grep-mini", "swe-grep"])
    );
}

#[tokio::test]
async fn mcp_call_returns_matching_repo_context() {
    let temp = tempfile::tempdir().unwrap();
    std::fs::write(
        temp.path().join("assign_model_probe.rs"),
        "fn assign_model_probe() {}\n",
    )
    .unwrap();

    let response = handle_request(json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/call",
        "params": {
            "name": "fast_context",
            "arguments": {
                "search_string": "where is the assign model probe",
                "repo_path": temp.path(),
                "execution_mode": "local"
            }
        }
    }))
    .await
    .unwrap();

    assert_eq!(response["error"], Value::Null);
    assert_eq!(
        response["result"]["structuredContent"]["snippets"][0]["path"],
        "assign_model_probe.rs"
    );
}

#[tokio::test]
async fn engine_rejects_repo_escape() {
    let temp = tempfile::tempdir().unwrap();

    let error = run_fast_context(FastContextRequest {
        search_string: "anything".into(),
        repo_path: temp.path().join("missing").display().to_string(),
        search_type: Default::default(),
        execution_mode: ExecutionMode::Local,
        model: FastContextModel::Both,
        fallback_local: false,
        max_files: 16,
        max_turns: 4,
    })
    .await
    .unwrap_err();

    assert!(error.to_string().contains("repo_root is not accessible"));
}

#[test]
fn swe_grep_request_can_target_both_fast_context_models() {
    let mini = build_swe_grep_request("find assign model", "/repo", SWE_GREP_MINI_MODEL);
    let full = build_swe_grep_request("find assign model", "/repo", SWE_GREP_MODEL);

    assert_eq!(mini["model"], SWE_GREP_MINI_MODEL);
    assert_eq!(full["model"], SWE_GREP_MODEL);
    assert!(mini["messages"][0]["content"]
        .as_str()
        .unwrap()
        .contains("/repo"));
}

#[test]
fn cascade_initial_payload_contains_fast_context_markers() {
    let payload = build_initial_payload("find assign model", "fake_windsurf_key", "1.13.104")
        .expect("payload");
    let text = String::from_utf8_lossy(&payload);

    assert!(text.contains(FAST_CONTEXT_MODEL));
    assert!(text.contains(FAST_CONTEXT_SENTINEL));
    assert!(text.contains(FAST_CONTEXT_TOOL_NAME));
    assert!(text.contains("fake_windsurf_key"));
}
