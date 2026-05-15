//! GenericOpenAi profile renderer unit tests.
//!
//! Relocated from `tests/unit_cursor_proto.rs` per the round-5 APPROVED
//! design at `.omx/research/cursor-phase0/client-profile-design-v3-deltas.md`
//! MAJ-5 ("Test Migration Plan"). The originals stay in
//! `tests/unit_cursor_proto.rs` during the migration window: those still
//! cover `proto::decode_exec_public_tool_call` directly. This file covers
//! the same 12 cases against the new
//! `profiles::generic_openai::render` entry point so the per-profile
//! dispatch is exercised independently of the proto layer.

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
    // ReadArgs is `{ path = 1 }`. Implementation produces tool_name "read"
    // (NOT "read_file" as the dispatcher contract docs sometimes phrase
    // it) and the call_id mirrors the cursor exec_id, not "".
    let args = encode_string_field(1, "/tmp/file.txt");
    let exec = build_exec(ExecKind::Read, args);
    let (name, call_id, arguments) = unwrap_emit(profiles::generic_openai::render(&exec));
    assert_eq!(name, "read");
    assert_eq!(call_id, "exec-fixture-id");
    assert_eq!(arguments["path"], "/tmp/file.txt");
}

#[test]
fn exec_request_maps_ls_args_to_ls_tool() {
    // LsArgs `{ path = 1 }`. Implementation tool_name is "ls" (NOT
    // "list_directory" — that is a Cursor-side display label, not the
    // proxy-internal mapping).
    let args = encode_string_field(1, "/tmp");
    let exec = build_exec(ExecKind::Ls, args);
    let (name, call_id, arguments) = unwrap_emit(profiles::generic_openai::render(&exec));
    assert_eq!(name, "ls");
    assert_eq!(call_id, "exec-fixture-id");
    assert_eq!(arguments["path"], "/tmp");
}

#[test]
fn exec_request_maps_grep_args_to_grep_tool() {
    // GrepArgs `{ pattern = 1, path = 2, output_mode = 3 }`.
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
    // ShellArgs `{ command = 1, working_directory = 2 }`.
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
    // ShellStream uses the same `{ command, working_directory }` envelope
    // and emits tool_name "shell_stream" (NOT "shell"; the run engine
    // distinguishes the two so the route layer can route stream output
    // back over the wire).
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
    // WriteArgs `{ path = 1, ... }`. The proxy emits tool_name "write"
    // (NOT "apply_patch" as the Codex CLI built-in is named upstream;
    // Lane K future work may align these for the public-API translation,
    // but the proto path keeps the Cursor-native name today).
    let args = encode_string_field(1, "/tmp/out.txt");
    let exec = build_exec(ExecKind::Write, args);
    let (name, call_id, arguments) = unwrap_emit(profiles::generic_openai::render(&exec));
    assert_eq!(name, "write");
    assert_eq!(call_id, "exec-fixture-id");
    assert_eq!(arguments["path"], "/tmp/out.txt");
}

#[test]
fn exec_request_maps_delete_args_to_delete_tool() {
    // DeleteArgs `{ path = 1 }`. tool_name is "delete" (NOT
    // "apply_patch" — see Lane K coordination note above).
    let args = encode_string_field(1, "/tmp/garbage.txt");
    let exec = build_exec(ExecKind::Delete, args);
    let (name, call_id, arguments) = unwrap_emit(profiles::generic_openai::render(&exec));
    assert_eq!(name, "delete");
    assert_eq!(call_id, "exec-fixture-id");
    assert_eq!(arguments["path"], "/tmp/garbage.txt");
}

#[test]
fn exec_request_maps_diagnostics_to_diagnostics_tool_with_unknown_args_shape() {
    // DiagnosticsArgs schema is opaque to the proxy today, so the impl
    // routes them through `decode_unknown_exec_args`, which returns an
    // object keyed `field_<n>`. Lane K coordination: when a public schema
    // for diagnostics surfaces, this test should pin to that shape; for
    // now we lock the fallback shape so any structural drift is loud.
    let args = [encode_string_field(1, "/tmp/diag.txt")].concat();
    let exec = build_exec(ExecKind::Diagnostics, args);
    let (name, call_id, arguments) = unwrap_emit(profiles::generic_openai::render(&exec));
    assert_eq!(name, "diagnostics");
    assert_eq!(call_id, "exec-fixture-id");
    // `decode_unknown_exec_args` keys field 1 as "field_1".
    assert_eq!(arguments["field_1"], "/tmp/diag.txt");
}

#[test]
fn exec_request_maps_mcp_payload_to_mcp_tool_name_from_field_5() {
    // McpArgs uses field 5 for the tool name, field 3 for tool_call_id,
    // field 2 (repeated) for arguments. This is the canonical MCP envelope
    // (NOT a proxy-internal cursor_codebase_search special case; all MCP
    // tools route through the same public tool-call decoder).
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
    // Composer-2 reasoning emits MCP args with `tool: cursor_codebase_search`
    // when it wants to consult the workspace index. The decoder surfaces the
    // same name unchanged so the public adapters emit a normal tool call.
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
    // FetchArgs `{ url = 1 }`. tool_name is "fetch" (NOT "fetch_url" —
    // see exec_public_tool_name in proto.rs).
    let args = encode_string_field(1, "https://example.com");
    let exec = build_exec(ExecKind::Fetch, args);
    let (name, call_id, arguments) = unwrap_emit(profiles::generic_openai::render(&exec));
    assert_eq!(name, "fetch");
    assert_eq!(call_id, "exec-fixture-id");
    assert_eq!(arguments["url"], "https://example.com");
}

#[test]
fn exec_request_maps_other_field_to_cursor_exec_fallback() {
    // Unknown ExecKind variants (proto fields the proxy does not know
    // how to translate) collapse to tool_name "cursor_exec" and an
    // unknown_exec_args shape with field_<n> keys. Lane K coordination:
    // any new ExecKind should add a typed mapping rather than relying on
    // this fallback.
    let args = [encode_string_field(1, "opaque payload")].concat();
    let exec = build_exec(ExecKind::Other(99), args);
    let (name, call_id, arguments) = unwrap_emit(profiles::generic_openai::render(&exec));
    assert_eq!(name, "cursor_exec");
    assert_eq!(call_id, "exec-fixture-id");
    assert_eq!(arguments["field_1"], "opaque payload");
}
