use unified_model_proxy_v2::upstream::cursor::profiles::{droid, refuse_code, RenderedToolCall};
use unified_model_proxy_v2::upstream::cursor::proto::{
    encode_message_field, encode_string_field, ExecKind, ExecRequest,
};

const FIXTURE_EXEC_ID: &str = "exec-fixture-id";

fn build_exec(kind: ExecKind, args: Vec<u8>) -> ExecRequest {
    ExecRequest {
        id: 42,
        exec_id: FIXTURE_EXEC_ID.to_string(),
        kind,
        args,
    }
}

fn assert_refuse(rendered: RenderedToolCall, expected_code: &'static str) -> (String, String) {
    match rendered {
        RenderedToolCall::Refuse {
            exec_id,
            reason,
            code,
        } => {
            assert_eq!(
                code, expected_code,
                "refuse code mismatch (expected {expected_code}, got {code})",
            );
            (exec_id, reason)
        }
        other => panic!("expected Refuse, got {other:?}"),
    }
}

fn unwrap_emit(rendered: RenderedToolCall) -> (String, serde_json::Value, String) {
    match rendered {
        RenderedToolCall::Emit {
            tool_name,
            arguments,
            tool_call_id,
        } => (tool_name, arguments, tool_call_id),
        other => panic!("expected Emit, got {other:?}"),
    }
}

#[test]
fn droid_read_emits_read_with_file_path() {
    let args = encode_string_field(1, "/tmp/file.txt");
    let exec = build_exec(ExecKind::Read, args);

    let (name, arguments, call_id) = unwrap_emit(droid::render(&exec));

    assert_eq!(name, "Read");
    assert_eq!(call_id, FIXTURE_EXEC_ID);
    assert_eq!(arguments["file_path"], "/tmp/file.txt");
    assert!(
        arguments.get("path").is_none(),
        "Droid Read rejects `path`; it requires `file_path`, got {arguments:?}",
    );
}

#[test]
fn droid_ls_emits_uppercase_ls_with_path() {
    let args = encode_string_field(1, "/tmp");
    let exec = build_exec(ExecKind::Ls, args);

    let (name, arguments, call_id) = unwrap_emit(droid::render(&exec));

    assert_eq!(name, "LS");
    assert_eq!(call_id, FIXTURE_EXEC_ID);
    assert_eq!(arguments["path"], "/tmp");
}

#[test]
fn droid_grep_emits_grep() {
    // 1. Directory path case
    let args = [
        encode_string_field(1, "needle"),
        encode_string_field(2, "/repo"),
        encode_string_field(3, "files"),
    ]
    .concat();
    let exec = build_exec(ExecKind::Grep, args);
    let (name, arguments, call_id) = unwrap_emit(droid::render(&exec));
    assert_eq!(name, "Grep");
    assert_eq!(call_id, FIXTURE_EXEC_ID);
    assert_eq!(arguments["pattern"], "needle");
    assert_eq!(arguments["glob_pattern"], "/repo/**/*");
    assert!(arguments.get("path").is_none());
    assert!(arguments.get("glob").is_none());
    assert!(arguments.get("output_mode").is_none());

    // 2. File path case
    let args_file = [
        encode_string_field(1, "needle"),
        encode_string_field(2, "src/lib.rs"),
        encode_string_field(3, "files"),
    ]
    .concat();
    let exec_file = build_exec(ExecKind::Grep, args_file);
    let (_, arguments_file, _) = unwrap_emit(droid::render(&exec_file));
    assert_eq!(arguments_file["glob_pattern"], "src/lib.rs");

    // 3. Empty path case
    let args_empty = [
        encode_string_field(1, "needle"),
        encode_string_field(2, ""),
        encode_string_field(3, "files"),
    ]
    .concat();
    let exec_empty = build_exec(ExecKind::Grep, args_empty);
    let (_, arguments_empty, _) = unwrap_emit(droid::render(&exec_empty));
    assert!(arguments_empty.get("glob_pattern").is_none());
}

#[test]
fn droid_empty_pattern_grep_emits_glob() {
    let args = [
        encode_string_field(1, ""),
        encode_string_field(2, "/repo"),
        encode_string_field(3, "files_with_matches"),
    ]
    .concat();
    let exec = build_exec(ExecKind::Grep, args);
    let (name, arguments, call_id) = unwrap_emit(droid::render(&exec));

    assert_eq!(name, "Glob");
    assert_eq!(call_id, FIXTURE_EXEC_ID);
    assert_eq!(arguments["patterns"], "**/*");
    assert_eq!(arguments["folder"], "/repo");
    assert!(arguments.get("glob").is_none());
    assert!(arguments.get("pattern").is_none());
}

#[test]
fn droid_shell_emits_execute_with_medium_risk() {
    let args = [
        encode_string_field(1, "ls -la /tmp"),
        encode_string_field(2, "/repo"),
    ]
    .concat();
    let exec = build_exec(ExecKind::Shell, args);

    let (name, arguments, call_id) = unwrap_emit(droid::render(&exec));

    assert_eq!(name, "Execute");
    assert_eq!(call_id, FIXTURE_EXEC_ID);
    assert_eq!(arguments["command"], "ls -la /tmp");
    assert_eq!(arguments["riskLevel"], "medium");
    assert_eq!(arguments["riskLevelReason"], "automated proxy invocation");
    assert!(
        arguments.get("background").is_none(),
        "Shell must not set background flag, got {arguments:?}",
    );
}

#[test]
fn droid_shell_stream_emits_execute_with_medium_risk() {
    let args = [
        encode_string_field(1, "tail -f log"),
        encode_string_field(2, "/var/log"),
    ]
    .concat();
    let exec = build_exec(ExecKind::ShellStream, args);

    let (name, arguments, call_id) = unwrap_emit(droid::render(&exec));

    assert_eq!(name, "Execute");
    assert_eq!(call_id, FIXTURE_EXEC_ID);
    assert_eq!(arguments["command"], "tail -f log");
    assert_eq!(arguments["riskLevel"], "medium");
    assert_eq!(arguments["riskLevelReason"], "automated proxy invocation");
    assert!(
        arguments.get("background").is_none(),
        "ShellStream must not set background flag, got {arguments:?}",
    );
}

#[test]
fn droid_background_shell_spawn_emits_execute_with_fire_and_forget_only() {
    let args = [
        encode_string_field(1, "long-running"),
        encode_string_field(2, "/tmp"),
    ]
    .concat();
    let exec = build_exec(ExecKind::BackgroundShellSpawn, args);

    let (name, arguments, call_id) = unwrap_emit(droid::render(&exec));

    assert_eq!(name, "Execute");
    assert_eq!(call_id, FIXTURE_EXEC_ID);
    assert_eq!(arguments["command"], "long-running");
    assert_eq!(arguments["fireAndForget"], true);
    assert!(arguments.get("background").is_none());
    assert_eq!(arguments["riskLevel"], "medium");
    assert_eq!(arguments["riskLevelReason"], "automated proxy invocation");
}

#[test]
fn droid_write_shell_stdin_refuses_with_capability_unsupported() {
    let exec = build_exec(ExecKind::WriteShellStdin, Vec::new());

    let (exec_id, reason) = assert_refuse(
        droid::render(&exec),
        refuse_code::CLIENT_CAPABILITY_UNSUPPORTED,
    );

    assert_eq!(exec_id, FIXTURE_EXEC_ID);
    assert!(
        reason.contains("WriteShellStdin") || reason.contains("shell PID"),
        "reason should mention WriteShellStdin/shell PID, got {reason}",
    );
}

#[test]
fn droid_write_refuses_with_missing_required_field() {
    let args = encode_string_field(1, "/tmp/out.txt");
    let exec = build_exec(ExecKind::Write, args);

    let (exec_id, reason) =
        assert_refuse(droid::render(&exec), refuse_code::MISSING_REQUIRED_FIELD);

    assert_eq!(exec_id, FIXTURE_EXEC_ID);
    assert!(
        reason.contains("content"),
        "reason should reference missing content, got {reason}",
    );
}

#[test]
fn droid_delete_emits_execute_with_high_risk_for_rm() {
    let args = encode_string_field(1, "/tmp/garbage.txt");
    let exec = build_exec(ExecKind::Delete, args);

    let (name, arguments, call_id) = unwrap_emit(droid::render(&exec));

    assert_eq!(name, "Execute");
    assert_eq!(call_id, FIXTURE_EXEC_ID);
    let command = arguments
        .get("command")
        .and_then(|v| v.as_str())
        .expect("command must be a string");
    assert!(
        command.starts_with("rm "),
        "command must start with `rm `, got {command:?}",
    );
    assert!(
        command.contains("/tmp/garbage.txt"),
        "command must include the deletion path, got {command:?}",
    );
    assert_eq!(arguments["riskLevel"], "high");
    assert_eq!(
        arguments["riskLevelReason"],
        "file deletion requested by Cursor exec"
    );
}

#[test]
fn droid_diagnostics_refuses_with_shape_unknown() {
    let args = encode_string_field(1, "/tmp/diag.txt");
    let exec = build_exec(ExecKind::Diagnostics, args);

    let (exec_id, _reason) = assert_refuse(
        droid::render(&exec),
        refuse_code::SHAPE_UNKNOWN_PENDING_LIVE_PHASE0,
    );

    assert_eq!(exec_id, FIXTURE_EXEC_ID);
}

#[test]
fn droid_request_context_refuses_internal() {
    let exec = build_exec(ExecKind::RequestContext, Vec::new());

    let (exec_id, reason) = assert_refuse(
        droid::render(&exec),
        refuse_code::CLIENT_CAPABILITY_UNSUPPORTED,
    );

    assert_eq!(exec_id, FIXTURE_EXEC_ID);
    assert!(
        reason.contains("proxy-internal") || reason.contains("RequestContext"),
        "reason should explain the proxy-internal nature, got {reason}",
    );
}

#[test]
fn droid_list_mcp_resources_refuses_with_capability_unsupported() {
    let exec = build_exec(ExecKind::ListMcpResources, Vec::new());

    let (exec_id, reason) = assert_refuse(
        droid::render(&exec),
        refuse_code::CLIENT_CAPABILITY_UNSUPPORTED,
    );

    assert_eq!(exec_id, FIXTURE_EXEC_ID);
    assert!(
        reason.contains("MCP") || reason.contains("list"),
        "reason should mention MCP listing, got {reason}",
    );
}

#[test]
fn droid_read_mcp_resource_refuses_with_capability_unsupported() {
    let exec = build_exec(ExecKind::ReadMcpResource, Vec::new());

    let (exec_id, reason) = assert_refuse(
        droid::render(&exec),
        refuse_code::CLIENT_CAPABILITY_UNSUPPORTED,
    );

    assert_eq!(exec_id, FIXTURE_EXEC_ID);
    assert!(
        reason.contains("MCP") || reason.contains("read"),
        "reason should mention MCP resources, got {reason}",
    );
}

#[test]
fn droid_mcp_namespaces_with_triple_underscore() {
    let argument_entry = [
        encode_string_field(1, "query"),
        encode_message_field(2, br#""hello""#),
    ]
    .concat();
    let args = [
        encode_message_field(2, &argument_entry),
        encode_string_field(3, "mcp-call-id"),
        encode_string_field(4, "ref"),
        encode_string_field(5, "ref_search_documentation"),
    ]
    .concat();
    let exec = build_exec(ExecKind::Mcp, args);

    let (name, arguments, call_id) = unwrap_emit(droid::render(&exec));

    assert_eq!(name, "ref___ref_search_documentation");
    assert_eq!(call_id, "mcp-call-id");
    assert_eq!(arguments["query"], "hello");
}

#[test]
fn droid_fetch_emits_fetch_url_with_url() {
    let args = encode_string_field(1, "https://example.com/page");
    let exec = build_exec(ExecKind::Fetch, args);

    let (name, arguments, call_id) = unwrap_emit(droid::render(&exec));

    assert_eq!(name, "FetchUrl");
    assert_eq!(call_id, FIXTURE_EXEC_ID);
    assert_eq!(arguments["url"], "https://example.com/page");
}

#[test]
fn droid_record_screen_refuses_with_unsupported() {
    let exec = build_exec(ExecKind::RecordScreen, Vec::new());

    let (exec_id, reason) = assert_refuse(droid::render(&exec), refuse_code::UNSUPPORTED_EXEC_KIND);

    assert_eq!(exec_id, FIXTURE_EXEC_ID);
    assert!(
        reason.contains("RecordScreen"),
        "reason should name the exec kind, got {reason}",
    );
}

#[test]
fn droid_computer_use_refuses_with_unsupported() {
    let exec = build_exec(ExecKind::ComputerUse, Vec::new());

    let (exec_id, reason) = assert_refuse(droid::render(&exec), refuse_code::UNSUPPORTED_EXEC_KIND);

    assert_eq!(exec_id, FIXTURE_EXEC_ID);
    assert!(
        reason.contains("ComputerUse"),
        "reason should name the exec kind, got {reason}",
    );
}

#[test]
fn droid_other_field_refuses_with_unsupported_and_field_number() {
    let args = encode_string_field(1, "opaque");
    let exec = build_exec(ExecKind::Other(99), args);

    let (exec_id, reason) = assert_refuse(droid::render(&exec), refuse_code::UNSUPPORTED_EXEC_KIND);

    assert_eq!(exec_id, FIXTURE_EXEC_ID);
    assert!(
        reason.contains("Other(99)"),
        "reason should preserve the unknown field number, got {reason}",
    );
}

#[test]
fn droid_opencode_native_read_refuses_as_mcp_leak() {
    let argument_entry = [
        encode_string_field(1, "file_path"),
        encode_message_field(2, br#""/tmp/test_file.txt""#),
    ]
    .concat();
    let args = [
        encode_message_field(2, &argument_entry),
        encode_string_field(3, "opencode-read-id"),
        encode_string_field(4, "opencode"),
        encode_string_field(5, "Read"),
    ]
    .concat();
    let exec = build_exec(ExecKind::Mcp, args);

    let (exec_id, reason) =
        assert_refuse(droid::render(&exec), refuse_code::NATIVE_TOOL_LEAKED_AS_MCP);

    assert_eq!(exec_id, FIXTURE_EXEC_ID);
    assert!(reason.contains("Read"));
}

#[test]
fn droid_empty_server_native_read_refuses_as_mcp_leak() {
    let args = [
        encode_string_field(3, "empty-server-read-id"),
        encode_string_field(5, "Read"),
    ]
    .concat();
    let exec = build_exec(ExecKind::Mcp, args);

    let (exec_id, reason) =
        assert_refuse(droid::render(&exec), refuse_code::NATIVE_TOOL_LEAKED_AS_MCP);

    assert_eq!(exec_id, FIXTURE_EXEC_ID);
    assert!(reason.contains("Read"));
}

#[test]
fn droid_opencode_namespaced_external_tool_passes_through_without_double_namespace() {
    let argument_entry = [
        encode_string_field(1, "query"),
        encode_message_field(2, br#""tool docs""#),
    ]
    .concat();
    let args = [
        encode_message_field(2, &argument_entry),
        encode_string_field(3, "opencode-ref-id"),
        encode_string_field(4, "opencode"),
        encode_string_field(5, "ref___ref_search_documentation"),
    ]
    .concat();
    let exec = build_exec(ExecKind::Mcp, args);

    let (name, arguments, call_id) = unwrap_emit(droid::render(&exec));

    assert_eq!(name, "ref___ref_search_documentation");
    assert_eq!(call_id, "opencode-ref-id");
    assert_eq!(arguments["query"], "tool docs");
}

#[test]
fn droid_opencode_non_native_raw_tool_passes_through() {
    let args = [
        encode_string_field(3, "opencode-lookup-id"),
        encode_string_field(4, "opencode"),
        encode_string_field(5, "lookup"),
    ]
    .concat();
    let exec = build_exec(ExecKind::Mcp, args);

    let (name, arguments, call_id) = unwrap_emit(droid::render(&exec));

    assert_eq!(name, "lookup");
    assert_eq!(call_id, "opencode-lookup-id");
    assert!(arguments.is_object());
}

#[test]
fn droid_empty_server_non_native_raw_tool_passes_through() {
    let args = [
        encode_string_field(3, "empty-server-lookup-id"),
        encode_string_field(5, "lookup"),
    ]
    .concat();
    let exec = build_exec(ExecKind::Mcp, args);

    let (name, arguments, call_id) = unwrap_emit(droid::render(&exec));

    assert_eq!(name, "lookup");
    assert_eq!(call_id, "empty-server-lookup-id");
    assert!(arguments.is_object());
}

#[test]
fn droid_third_party_read_collision_still_namespaces() {
    let args = [
        encode_string_field(3, "filesystem-read-id"),
        encode_string_field(4, "filesystem"),
        encode_string_field(5, "Read"),
    ]
    .concat();
    let exec = build_exec(ExecKind::Mcp, args);

    let (name, _arguments, call_id) = unwrap_emit(droid::render(&exec));

    assert_eq!(name, "filesystem___Read");
    assert_eq!(call_id, "filesystem-read-id");
}

#[test]
fn droid_cursor_codebase_search_mcp_refuses_internal_leak() {
    let args = [
        encode_string_field(3, "cursor-search-id"),
        encode_string_field(4, "opencode"),
        encode_string_field(5, "cursor_codebase_search"),
    ]
    .concat();
    let exec = build_exec(ExecKind::Mcp, args);

    let (exec_id, reason) = assert_refuse(
        droid::render(&exec),
        refuse_code::CLIENT_CAPABILITY_UNSUPPORTED,
    );

    assert_eq!(exec_id, FIXTURE_EXEC_ID);
    assert!(reason.contains("cursor_codebase_search"));
}
