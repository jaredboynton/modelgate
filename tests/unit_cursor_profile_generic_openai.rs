use unified_model_proxy_v2::upstream::cursor::profiles::{self, RenderedToolCall};
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
        } => panic!("expected Emit, got Refuse(exec_id={exec_id}, code={code}, reason={reason})",),
    }
}

#[test]
fn exec_request_maps_read_args_to_read_tool() {
    let args = encode_string_field(1, "/tmp/file.txt");
    let exec = build_exec(ExecKind::Read, args);
    let (name, call_id, arguments) = unwrap_emit(profiles::generic_openai::render(&exec));
    assert_eq!(name, "read");
    assert_eq!(call_id, "exec-fixture-id");
    assert_eq!(arguments["path"], "/tmp/file.txt");
}

#[test]
fn exec_request_maps_ls_args_to_ls_tool() {
    let args = encode_string_field(1, "/tmp");
    let exec = build_exec(ExecKind::Ls, args);
    let (name, call_id, arguments) = unwrap_emit(profiles::generic_openai::render(&exec));
    assert_eq!(name, "ls");
    assert_eq!(call_id, "exec-fixture-id");
    assert_eq!(arguments["path"], "/tmp");
}

#[test]
fn exec_request_maps_grep_args_to_grep_tool() {
    let args = [
        encode_string_field(1, "needle"),
        encode_string_field(2, "/repo"),
        encode_string_field(3, "files_with_matches"),
    ]
    .concat();
    let exec = build_exec(ExecKind::Grep, args);
    let (name, call_id, arguments) = unwrap_emit(profiles::generic_openai::render(&exec));
    assert_eq!(name, "grep");
    assert_eq!(call_id, "exec-fixture-id");
    assert_eq!(arguments["pattern"], "needle");
    assert_eq!(arguments["path"], "/repo");
    assert_eq!(arguments["output_mode"], "files_with_matches");
}

#[test]
fn exec_request_maps_shell_args_to_shell_tool() {
    let args = [
        encode_string_field(1, "ls -la"),
        encode_string_field(2, "/repo"),
    ]
    .concat();
    let exec = build_exec(ExecKind::Shell, args);
    let (name, call_id, arguments) = unwrap_emit(profiles::generic_openai::render(&exec));
    assert_eq!(name, "shell");
    assert_eq!(call_id, "exec-fixture-id");
    assert_eq!(arguments["command"], "ls -la");
    assert_eq!(arguments["working_directory"], "/repo");
}

#[test]
fn exec_request_maps_shell_stream_args_to_shell_stream_tool() {
    let args = [
        encode_string_field(1, "tail -f log"),
        encode_string_field(2, "/var/log"),
    ]
    .concat();
    let exec = build_exec(ExecKind::ShellStream, args);
    let (name, call_id, arguments) = unwrap_emit(profiles::generic_openai::render(&exec));
    assert_eq!(name, "shell_stream");
    assert_eq!(call_id, "exec-fixture-id");
    assert_eq!(arguments["command"], "tail -f log");
    assert_eq!(arguments["working_directory"], "/var/log");
}

#[test]
fn exec_request_maps_write_args_to_write_tool() {
    let args = encode_string_field(1, "/tmp/out.txt");
    let exec = build_exec(ExecKind::Write, args);
    let (name, call_id, arguments) = unwrap_emit(profiles::generic_openai::render(&exec));
    assert_eq!(name, "write");
    assert_eq!(call_id, "exec-fixture-id");
    assert_eq!(arguments["path"], "/tmp/out.txt");
}

#[test]
fn exec_request_maps_delete_args_to_delete_tool() {
    let args = encode_string_field(1, "/tmp/garbage.txt");
    let exec = build_exec(ExecKind::Delete, args);
    let (name, call_id, arguments) = unwrap_emit(profiles::generic_openai::render(&exec));
    assert_eq!(name, "delete");
    assert_eq!(call_id, "exec-fixture-id");
    assert_eq!(arguments["path"], "/tmp/garbage.txt");
}

#[test]
fn exec_request_maps_diagnostics_to_diagnostics_tool_with_unknown_args_shape() {
    let args = [encode_string_field(1, "/tmp/diag.txt")].concat();
    let exec = build_exec(ExecKind::Diagnostics, args);
    let (name, call_id, arguments) = unwrap_emit(profiles::generic_openai::render(&exec));
    assert_eq!(name, "diagnostics");
    assert_eq!(call_id, "exec-fixture-id");
    assert_eq!(arguments["field_1"], "/tmp/diag.txt");
}

#[test]
fn exec_request_maps_mcp_payload_to_mcp_tool_name_from_field_5() {
    let argument_entry = [
        encode_string_field(1, "key"),
        encode_message_field(2, br#""value""#),
    ]
    .concat();
    let args = [
        encode_message_field(2, &argument_entry),
        encode_string_field(3, "mcp-call-id-123"),
        encode_string_field(5, "third_party_tool"),
    ]
    .concat();
    let exec = build_exec(ExecKind::Mcp, args);
    let (name, call_id, arguments) = unwrap_emit(profiles::generic_openai::render(&exec));
    assert_eq!(name, "third_party_tool");
    assert_eq!(call_id, "mcp-call-id-123");
    assert_eq!(arguments["key"], "value");
}

#[test]
fn exec_request_maps_mcp_payload_for_cursor_codebase_search_public_tool() {
    let argument_entry = [
        encode_string_field(1, "query"),
        encode_message_field(2, br#""how does auth flow""#),
    ]
    .concat();
    let args = [
        encode_message_field(2, &argument_entry),
        encode_string_field(3, "search-call-id"),
        encode_string_field(5, "cursor_codebase_search"),
    ]
    .concat();
    let exec = build_exec(ExecKind::Mcp, args);
    let (name, call_id, arguments) = unwrap_emit(profiles::generic_openai::render(&exec));
    assert_eq!(name, "cursor_codebase_search");
    assert_eq!(call_id, "search-call-id");
    assert_eq!(arguments["query"], "how does auth flow");
}

#[test]
fn exec_request_maps_fetch_args_to_fetch_tool() {
    let args = encode_string_field(1, "https://example.com");
    let exec = build_exec(ExecKind::Fetch, args);
    let (name, call_id, arguments) = unwrap_emit(profiles::generic_openai::render(&exec));
    assert_eq!(name, "fetch");
    assert_eq!(call_id, "exec-fixture-id");
    assert_eq!(arguments["url"], "https://example.com");
}

#[test]
fn exec_request_maps_other_field_to_cursor_exec_fallback() {
    let args = [encode_string_field(1, "opaque payload")].concat();
    let exec = build_exec(ExecKind::Other(99), args);
    let (name, call_id, arguments) = unwrap_emit(profiles::generic_openai::render(&exec));
    assert_eq!(name, "cursor_exec");
    assert_eq!(call_id, "exec-fixture-id");
    assert_eq!(arguments["field_1"], "opaque payload");
}
