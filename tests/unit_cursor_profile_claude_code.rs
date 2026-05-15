use unified_model_proxy_v2::upstream::cursor::profiles::{self, refuse_code, RenderedToolCall};
use unified_model_proxy_v2::upstream::cursor::proto::{
    encode_message_field, encode_string_field, ExecKind, ExecRequest,
};

fn build_exec(kind: ExecKind, args: Vec<u8>) -> ExecRequest {
    ExecRequest {
        id: 42,
        exec_id: "exec-fixture-id".to_string(),
        kind,
        args,
    }
}

fn unwrap_emit(rendered: RenderedToolCall) -> (String, String, serde_json::Value) {
    match rendered {
        RenderedToolCall::Emit {
            tool_name,
            tool_call_id,
            arguments,
        } => (tool_name, tool_call_id, arguments),
        RenderedToolCall::Refuse {
            exec_id,
            reason,
            code,
        } => panic!("expected Emit, got Refuse(exec_id={exec_id}, code={code}, reason={reason})"),
    }
}

fn unwrap_refuse(rendered: RenderedToolCall) -> (String, String, &'static str) {
    match rendered {
 RenderedToolCall::Refuse {
 exec_id,
 reason,
 code,
 } => (exec_id, reason, code),
 RenderedToolCall::Emit {
 tool_name,
 tool_call_id,
 arguments,
 } => panic!(
 "expected Refuse, got Emit(tool_name={tool_name}, tool_call_id={tool_call_id}, arguments={arguments})"
 ),
 }
}

#[test]
fn claude_read_emits_read_with_file_path() {
    let args = encode_string_field(1, "/tmp/file.txt");
    let exec = build_exec(ExecKind::Read, args);
    let (name, call_id, arguments) = unwrap_emit(profiles::claude_code::render(&exec));
    assert_eq!(name, "Read");
    assert_eq!(call_id, "exec-fixture-id");
    assert_eq!(arguments["file_path"], "/tmp/file.txt");
}

#[test]
fn claude_ls_emits_bash_with_ls_command() {
    let args = encode_string_field(1, "/tmp");
    let exec = build_exec(ExecKind::Ls, args);
    let (name, call_id, arguments) = unwrap_emit(profiles::claude_code::render(&exec));
    assert_eq!(name, "Bash");
    assert_eq!(call_id, "exec-fixture-id");
    assert_eq!(arguments["command"], "ls /tmp");
}

#[test]
fn claude_grep_emits_grep_with_pattern_and_path() {
    let args = [
        encode_string_field(1, "needle"),
        encode_string_field(2, "/repo"),
        encode_string_field(3, "files_with_matches"),
    ]
    .concat();
    let exec = build_exec(ExecKind::Grep, args);
    let (name, call_id, arguments) = unwrap_emit(profiles::claude_code::render(&exec));
    assert_eq!(name, "Grep");
    assert_eq!(call_id, "exec-fixture-id");
    assert_eq!(arguments["pattern"], "needle");
    assert_eq!(arguments["path"], "/repo");
    assert_eq!(arguments["output_mode"], "files_with_matches");
}

#[test]
fn claude_shell_emits_bash_foreground() {
    let args = [
        encode_string_field(1, "ls -la"),
        encode_string_field(2, "/repo"),
    ]
    .concat();
    let exec = build_exec(ExecKind::Shell, args);
    let (name, call_id, arguments) = unwrap_emit(profiles::claude_code::render(&exec));
    assert_eq!(name, "Bash");
    assert_eq!(call_id, "exec-fixture-id");
    assert_eq!(arguments["command"], "ls -la");
    assert!(
        arguments.get("run_in_background").is_none(),
        "foreground Bash must omit run_in_background per policy"
    );
}

#[test]
fn claude_shell_stream_emits_bash_foreground() {
    let args = [
        encode_string_field(1, "tail -f log"),
        encode_string_field(2, "/var/log"),
    ]
    .concat();
    let exec = build_exec(ExecKind::ShellStream, args);
    let (name, call_id, arguments) = unwrap_emit(profiles::claude_code::render(&exec));
    assert_eq!(name, "Bash");
    assert_eq!(call_id, "exec-fixture-id");
    assert_eq!(arguments["command"], "tail -f log");
    assert!(arguments.get("run_in_background").is_none());
}

#[test]
fn claude_background_shell_spawn_emits_bash_with_run_in_background() {
    let args = encode_string_field(1, "long-running &");
    let exec = build_exec(ExecKind::BackgroundShellSpawn, args);
    let (name, call_id, arguments) = unwrap_emit(profiles::claude_code::render(&exec));
    assert_eq!(name, "Bash");
    assert_eq!(call_id, "exec-fixture-id");
    assert_eq!(arguments["command"], "long-running &");
    assert_eq!(arguments["run_in_background"], true);
}

#[test]
fn claude_write_shell_stdin_refuses_with_capability_unsupported() {
    let args = encode_string_field(2, "ignored");
    let exec = build_exec(ExecKind::WriteShellStdin, args);
    let (exec_id, reason, code) = unwrap_refuse(profiles::claude_code::render(&exec));
    assert_eq!(exec_id, "exec-fixture-id");
    assert_eq!(code, refuse_code::CLIENT_CAPABILITY_UNSUPPORTED);
    assert!(
        reason.contains("BashOutput"),
        "refuse reason must name BashOutput: {reason}"
    );
}

#[test]
fn claude_write_refuses_with_missing_required_field() {
    let args = encode_string_field(1, "/tmp/out.txt");
    let exec = build_exec(ExecKind::Write, args);
    let (exec_id, reason, code) = unwrap_refuse(profiles::claude_code::render(&exec));
    assert_eq!(exec_id, "exec-fixture-id");
    assert_eq!(code, refuse_code::MISSING_REQUIRED_FIELD);
    assert!(
        reason.contains("content"),
        "refuse reason must mention missing content: {reason}"
    );
}

#[test]
fn claude_delete_refuses_with_capability_unsupported() {
    let args = encode_string_field(1, "/tmp/garbage.txt");
    let exec = build_exec(ExecKind::Delete, args);
    let (exec_id, reason, code) = unwrap_refuse(profiles::claude_code::render(&exec));
    assert_eq!(exec_id, "exec-fixture-id");
    assert_eq!(code, refuse_code::CLIENT_CAPABILITY_UNSUPPORTED);
    assert!(
        reason.contains("Delete"),
        "refuse reason must name Delete: {reason}"
    );
}

#[test]
fn claude_diagnostics_refuses_with_shape_unknown() {
    let args = encode_string_field(1, "/tmp/diag.txt");
    let exec = build_exec(ExecKind::Diagnostics, args);
    let (exec_id, reason, code) = unwrap_refuse(profiles::claude_code::render(&exec));
    assert_eq!(exec_id, "exec-fixture-id");
    assert_eq!(code, refuse_code::SHAPE_UNKNOWN_PENDING_LIVE_PHASE0);
    assert!(
        reason.contains("Diagnostics"),
        "refuse reason must name Diagnostics: {reason}"
    );
}

#[test]
fn claude_request_context_refuses_internal() {
    let exec = build_exec(ExecKind::RequestContext, Vec::new());
    let (exec_id, reason, code) = unwrap_refuse(profiles::claude_code::render(&exec));
    assert_eq!(exec_id, "exec-fixture-id");
    assert_eq!(code, refuse_code::CLIENT_CAPABILITY_UNSUPPORTED);
    assert!(
        reason.contains("proxy-internal"),
        "refuse reason must flag proxy-internal: {reason}"
    );
}

#[test]
fn claude_list_mcp_resources_emits_built_in_tool() {
    let args = encode_string_field(1, "filesystem");
    let exec = build_exec(ExecKind::ListMcpResources, args);
    let (name, call_id, arguments) = unwrap_emit(profiles::claude_code::render(&exec));
    assert_eq!(name, "ListMcpResourcesTool");
    assert_eq!(call_id, "exec-fixture-id");
    assert_eq!(arguments["server"], "filesystem");
}

#[test]
fn claude_read_mcp_resource_emits_built_in_tool() {
    let args = [
        encode_string_field(1, "filesystem"),
        encode_string_field(2, "file:///tmp/x.txt"),
    ]
    .concat();
    let exec = build_exec(ExecKind::ReadMcpResource, args);
    let (name, call_id, arguments) = unwrap_emit(profiles::claude_code::render(&exec));
    assert_eq!(name, "ReadMcpResourceTool");
    assert_eq!(call_id, "exec-fixture-id");
    assert_eq!(arguments["server"], "filesystem");
    assert_eq!(arguments["uri"], "file:///tmp/x.txt");
}

#[test]
fn claude_mcp_namespaces_with_double_underscore() {
    let argument_entry = [
        encode_string_field(1, "key"),
        encode_message_field(2, br#""value""#),
    ]
    .concat();
    let args = [
        encode_message_field(2, &argument_entry),
        encode_string_field(3, "mcp-call-id-123"),
        encode_string_field(4, "filesystem"),
        encode_string_field(5, "search_file"),
    ]
    .concat();
    let exec = build_exec(ExecKind::Mcp, args);
    let (name, call_id, arguments) = unwrap_emit(profiles::claude_code::render(&exec));
    assert_eq!(name, "mcp__filesystem__search_file");
    assert_eq!(call_id, "mcp-call-id-123");
    assert_eq!(arguments["key"], "value");
}

#[test]
fn claude_fetch_emits_webfetch_with_synth_default_prompt() {
    let args = encode_string_field(1, "https://example.com");
    let exec = build_exec(ExecKind::Fetch, args);
    let (name, call_id, arguments) = unwrap_emit(profiles::claude_code::render(&exec));
    assert_eq!(name, "WebFetch");
    assert_eq!(call_id, "exec-fixture-id");
    assert_eq!(arguments["url"], "https://example.com");
    assert_eq!(
        arguments["prompt"],
        "Summarize the page contents and return the relevant section for the user's request."
    );
}

#[test]
fn claude_record_screen_refuses_with_unsupported() {
    let exec = build_exec(ExecKind::RecordScreen, Vec::new());
    let (exec_id, reason, code) = unwrap_refuse(profiles::claude_code::render(&exec));
    assert_eq!(exec_id, "exec-fixture-id");
    assert_eq!(code, refuse_code::UNSUPPORTED_EXEC_KIND);
    assert!(
        reason.contains("RecordScreen"),
        "refuse reason must name RecordScreen: {reason}"
    );
}

#[test]
fn claude_computer_use_refuses_with_unsupported() {
    let exec = build_exec(ExecKind::ComputerUse, Vec::new());
    let (exec_id, reason, code) = unwrap_refuse(profiles::claude_code::render(&exec));
    assert_eq!(exec_id, "exec-fixture-id");
    assert_eq!(code, refuse_code::UNSUPPORTED_EXEC_KIND);
    assert!(
        reason.contains("ComputerUse"),
        "refuse reason must name ComputerUse: {reason}"
    );
}

#[test]
fn claude_other_field_refuses_with_unsupported_and_field_number() {
    let exec = build_exec(ExecKind::Other(99), encode_string_field(1, "opaque"));
    let (exec_id, reason, code) = unwrap_refuse(profiles::claude_code::render(&exec));
    assert_eq!(exec_id, "exec-fixture-id");
    assert_eq!(code, refuse_code::UNSUPPORTED_EXEC_KIND);
    assert!(
        reason.contains("Other(99)"),
        "refuse reason must include the field number: {reason}"
    );
}
