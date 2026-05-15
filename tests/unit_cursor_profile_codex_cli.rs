//! Unit tests for the CodexCli profile renderer.
//!
//! Locks the Cursor `ExecKind` x CodexCli profile cell matrix per the
//! round-5 design at `.omx/research/cursor-phase0/client-profile-design-v3-deltas.md`
//! and the policy doc at `.omx/research/cursor-phase0/client-profile-policy.md`.
//!
//! Codex CLI built-in tool surface citations live in
//! `.omx/research/cursor-phase0/client-tool-codex.md`:
//! - `shell_command` / `exec_command` / `write_stdin` / `apply_patch`
//!   sourced from `codex-rs/core/src/tools/handlers/shell_spec.rs` and
//!   `codex-rs/core/src/tools/handlers/apply_patch_spec.rs`.
//! - `list_mcp_resources` / `read_mcp_resource` from
//!   `codex-rs/core/src/tools/handlers/mcp_resource_spec.rs`.

use unified_model_proxy_v2::upstream::cursor::profiles::{
    codex_cli, refuse_code, RenderedToolCall,
};
use unified_model_proxy_v2::upstream::cursor::proto::{
    encode_int64_field, encode_message_field, encode_string_field, ExecKind, ExecRequest,
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
fn codex_read_emits_shell_command_cat() {
    let args = encode_string_field(1, "/tmp/file.txt");
    let exec = build_exec(ExecKind::Read, args);

    let (name, arguments, call_id) = unwrap_emit(codex_cli::render(&exec));

    assert_eq!(name, "shell_command");
    assert_eq!(call_id, FIXTURE_EXEC_ID);
    let cmd = arguments
        .get("cmd")
        .and_then(|v| v.as_array())
        .expect("cmd is an array");
    let tokens: Vec<&str> = cmd.iter().filter_map(|v| v.as_str()).collect();
    assert_eq!(tokens, vec!["cat", "/tmp/file.txt"]);
}

#[test]
fn codex_ls_emits_shell_command_ls() {
    let args = encode_string_field(1, "/tmp");
    let exec = build_exec(ExecKind::Ls, args);

    let (name, arguments, call_id) = unwrap_emit(codex_cli::render(&exec));

    assert_eq!(name, "shell_command");
    assert_eq!(call_id, FIXTURE_EXEC_ID);
    let cmd = arguments
        .get("cmd")
        .and_then(|v| v.as_array())
        .expect("cmd is an array");
    let tokens: Vec<&str> = cmd.iter().filter_map(|v| v.as_str()).collect();
    assert_eq!(tokens, vec!["ls", "/tmp"]);
}

#[test]
fn codex_grep_emits_shell_command_grep() {
    let args = [
        encode_string_field(1, "needle"),
        encode_string_field(2, "/repo"),
    ]
    .concat();
    let exec = build_exec(ExecKind::Grep, args);

    let (name, arguments, call_id) = unwrap_emit(codex_cli::render(&exec));

    assert_eq!(name, "shell_command");
    assert_eq!(call_id, FIXTURE_EXEC_ID);
    let cmd = arguments
        .get("cmd")
        .and_then(|v| v.as_array())
        .expect("cmd is an array");
    let tokens: Vec<&str> = cmd.iter().filter_map(|v| v.as_str()).collect();
    assert_eq!(tokens, vec!["grep", "-rn", "needle", "/repo"]);
}

#[test]
fn codex_shell_emits_shell_command_with_bash_dash_c() {
    // ShellArgs `{ command = 1, working_directory = 2 }`. CodexCli wraps the
    // command string in `["bash", "-c", "<raw>"]` because Codex's
    // `shell_command` takes a single command string but Cursor exec args
    // arrive as an opaque shell line.
    let args = [
        encode_string_field(1, "ls -la /tmp"),
        encode_string_field(2, "/repo"),
    ]
    .concat();
    let exec = build_exec(ExecKind::Shell, args);

    let (name, arguments, call_id) = unwrap_emit(codex_cli::render(&exec));

    assert_eq!(name, "shell_command");
    assert_eq!(call_id, FIXTURE_EXEC_ID);
    let cmd = arguments
        .get("cmd")
        .and_then(|v| v.as_array())
        .expect("cmd is an array");
    let tokens: Vec<&str> = cmd.iter().filter_map(|v| v.as_str()).collect();
    assert_eq!(tokens, vec!["bash", "-c", "ls -la /tmp"]);
    assert_eq!(
        arguments.get("workdir").and_then(|v| v.as_str()),
        Some("/repo")
    );
}

#[test]
fn codex_shell_stream_emits_exec_command() {
    let args = [
        encode_string_field(1, "tail -f log"),
        encode_string_field(2, "/var/log"),
    ]
    .concat();
    let exec = build_exec(ExecKind::ShellStream, args);

    let (name, arguments, call_id) = unwrap_emit(codex_cli::render(&exec));

    assert_eq!(name, "exec_command");
    assert_eq!(call_id, FIXTURE_EXEC_ID);
    let cmd = arguments
        .get("cmd")
        .and_then(|v| v.as_array())
        .expect("cmd is an array");
    let tokens: Vec<&str> = cmd.iter().filter_map(|v| v.as_str()).collect();
    assert_eq!(tokens, vec!["bash", "-c", "tail -f log"]);
    assert_eq!(
        arguments.get("workdir").and_then(|v| v.as_str()),
        Some("/var/log")
    );
}

#[test]
fn codex_background_shell_spawn_refuses_with_capability_unsupported() {
    // Codex CLI's `exec_command` does not natively background a shell; the
    // proxy refuses rather than synthesizing a fake session id.
    let args = [
        encode_string_field(1, "long-running"),
        encode_string_field(2, "/tmp"),
    ]
    .concat();
    let exec = build_exec(ExecKind::BackgroundShellSpawn, args);

    let (exec_id, reason) = assert_refuse(
        codex_cli::render(&exec),
        refuse_code::CLIENT_CAPABILITY_UNSUPPORTED,
    );

    assert_eq!(exec_id, FIXTURE_EXEC_ID);
    assert!(
        reason.contains("background"),
        "reason should mention backgrounding, got {reason}",
    );
}

#[test]
fn codex_write_shell_stdin_emits_write_stdin() {
    // WriteShellStdinArgs `{ shell_id = 1 (varint), input = 2 (string) }`.
    let args = [
        encode_int64_field(1, 17),
        encode_string_field(2, "hello stdin"),
    ]
    .concat();
    let exec = build_exec(ExecKind::WriteShellStdin, args);

    let (name, arguments, call_id) = unwrap_emit(codex_cli::render(&exec));

    assert_eq!(name, "write_stdin");
    assert_eq!(call_id, FIXTURE_EXEC_ID);
    assert_eq!(arguments["shell_id"], 17);
    assert_eq!(arguments["input"], "hello stdin");
}

#[test]
fn codex_write_refuses_with_missing_required_field() {
    // Cursor `Write` carries `path` only; Codex `apply_patch` needs the body
    // bytes. Wave 0 refuses pending Live Phase 0.
    let args = encode_string_field(1, "/tmp/out.txt");
    let exec = build_exec(ExecKind::Write, args);

    let (exec_id, reason) = assert_refuse(
        codex_cli::render(&exec),
        refuse_code::MISSING_REQUIRED_FIELD,
    );

    assert_eq!(exec_id, FIXTURE_EXEC_ID);
    assert!(
        reason.contains("Live Phase 0") || reason.contains("body"),
        "reason should reference missing body, got {reason}",
    );
}

#[test]
fn codex_delete_emits_apply_patch_with_delete_file() {
    let args = encode_string_field(1, "/tmp/garbage.txt");
    let exec = build_exec(ExecKind::Delete, args);

    let (name, arguments, call_id) = unwrap_emit(codex_cli::render(&exec));

    assert_eq!(name, "apply_patch");
    assert_eq!(call_id, FIXTURE_EXEC_ID);
    let patch = arguments
        .get("patch")
        .and_then(|v| v.as_str())
        .expect("patch field is a string");
    assert!(
        patch.starts_with("*** Begin Patch\n"),
        "patch must start with Begin Patch sentinel, got {patch:?}",
    );
    assert!(
        patch.contains("*** Delete File: /tmp/garbage.txt\n"),
        "patch must contain Delete File line for the path, got {patch:?}",
    );
    assert!(
        patch.trim_end().ends_with("*** End Patch"),
        "patch must terminate with End Patch sentinel, got {patch:?}",
    );
}

#[test]
fn codex_diagnostics_refuses_with_shape_unknown() {
    let args = encode_string_field(1, "/tmp/diag.txt");
    let exec = build_exec(ExecKind::Diagnostics, args);

    let (exec_id, _reason) = assert_refuse(
        codex_cli::render(&exec),
        refuse_code::SHAPE_UNKNOWN_PENDING_LIVE_PHASE0,
    );

    assert_eq!(exec_id, FIXTURE_EXEC_ID);
}

#[test]
fn codex_request_context_refuses_internal() {
    // RequestContext is proxy-internal; the run engine answers it locally and
    // never dispatches it to a renderer. If it ever reaches the renderer the
    // refusal is the safe outcome.
    let exec = build_exec(ExecKind::RequestContext, Vec::new());

    let (exec_id, reason) = assert_refuse(
        codex_cli::render(&exec),
        refuse_code::CLIENT_CAPABILITY_UNSUPPORTED,
    );

    assert_eq!(exec_id, FIXTURE_EXEC_ID);
    assert!(
        reason.contains("proxy-internal") || reason.contains("RequestContext"),
        "reason should explain the proxy-internal nature, got {reason}",
    );
}

#[test]
fn codex_list_mcp_resources_emits_built_in() {
    let exec = build_exec(ExecKind::ListMcpResources, Vec::new());

    let (name, arguments, call_id) = unwrap_emit(codex_cli::render(&exec));

    assert_eq!(name, "list_mcp_resources");
    assert_eq!(call_id, FIXTURE_EXEC_ID);
    assert!(
        arguments.is_object(),
        "list_mcp_resources args must be a JSON object, got {arguments:?}",
    );
    assert_eq!(
        arguments.as_object().map(|m| m.len()),
        Some(0),
        "list_mcp_resources args object must be empty",
    );
}

#[test]
fn codex_read_mcp_resource_emits_built_in() {
    // ReadMcpResourceExecArgs `{ server = 1, uri = 2 }`.
    let args = [
        encode_string_field(1, "fs"),
        encode_string_field(2, "file:///tmp/notes.md"),
    ]
    .concat();
    let exec = build_exec(ExecKind::ReadMcpResource, args);

    let (name, arguments, call_id) = unwrap_emit(codex_cli::render(&exec));

    assert_eq!(name, "read_mcp_resource");
    assert_eq!(call_id, FIXTURE_EXEC_ID);
    assert_eq!(arguments["server"], "fs");
    assert_eq!(arguments["uri"], "file:///tmp/notes.md");
}

#[test]
fn codex_mcp_namespaces_with_double_underscore() {
    // McpArgs uses field 5 for the bare tool name, field 4 for the server
    // identifier, field 3 for the tool_call_id, and a `repeated MapEntry`
    // arguments list at field 2 (key=1, value=bytes-of-JSON at 2).
    let argument_entry = [
        encode_string_field(1, "query"),
        encode_message_field(2, br#""hello""#),
    ]
    .concat();
    let args = [
        encode_message_field(2, &argument_entry),
        encode_string_field(3, "mcp-call-id"),
        encode_string_field(4, "github"),
        encode_string_field(5, "list_prs"),
    ]
    .concat();
    let exec = build_exec(ExecKind::Mcp, args);

    let (name, arguments, call_id) = unwrap_emit(codex_cli::render(&exec));

    assert_eq!(name, "mcp__github__list_prs");
    assert_eq!(call_id, "mcp-call-id");
    assert_eq!(arguments["query"], "hello");
}

#[test]
fn codex_fetch_emits_shell_command_curl() {
    let args = encode_string_field(1, "https://example.com/page");
    let exec = build_exec(ExecKind::Fetch, args);

    let (name, arguments, call_id) = unwrap_emit(codex_cli::render(&exec));

    assert_eq!(name, "shell_command");
    assert_eq!(call_id, FIXTURE_EXEC_ID);
    let cmd = arguments
        .get("cmd")
        .and_then(|v| v.as_array())
        .expect("cmd is an array");
    let tokens: Vec<&str> = cmd.iter().filter_map(|v| v.as_str()).collect();
    assert_eq!(tokens, vec!["curl", "https://example.com/page"]);
}

#[test]
fn codex_record_screen_refuses_with_unsupported() {
    let exec = build_exec(ExecKind::RecordScreen, Vec::new());

    let (exec_id, reason) =
        assert_refuse(codex_cli::render(&exec), refuse_code::UNSUPPORTED_EXEC_KIND);

    assert_eq!(exec_id, FIXTURE_EXEC_ID);
    assert!(
        reason.contains("RecordScreen"),
        "reason should name the exec kind, got {reason}",
    );
}

#[test]
fn codex_computer_use_refuses_with_unsupported() {
    let exec = build_exec(ExecKind::ComputerUse, Vec::new());

    let (exec_id, reason) =
        assert_refuse(codex_cli::render(&exec), refuse_code::UNSUPPORTED_EXEC_KIND);

    assert_eq!(exec_id, FIXTURE_EXEC_ID);
    assert!(
        reason.contains("ComputerUse"),
        "reason should name the exec kind, got {reason}",
    );
}

#[test]
fn codex_other_field_refuses_with_unsupported_and_field_number() {
    let args = encode_string_field(1, "opaque");
    let exec = build_exec(ExecKind::Other(99), args);

    let (exec_id, reason) =
        assert_refuse(codex_cli::render(&exec), refuse_code::UNSUPPORTED_EXEC_KIND);

    assert_eq!(exec_id, FIXTURE_EXEC_ID);
    assert!(
        reason.contains("Other(99)"),
        "reason should preserve the unknown field number, got {reason}",
    );
}
