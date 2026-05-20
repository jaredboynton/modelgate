//! CodexCli profile renderer.
//!
//! Maps Cursor exec requests to Codex CLI native tool calls.

use super::native_tools;
use super::proto_helpers::{read_string_field, read_u64_field};
use super::{refuse_code, RenderedToolCall};
use crate::upstream::cursor::client_profile::ClientProfile;
use crate::upstream::cursor::proto::{decode_exec_public_tool_call, ExecKind, ExecRequest};
use serde_json::json;

pub fn render(exec: &ExecRequest) -> RenderedToolCall {
    if matches!(exec.kind, ExecKind::Mcp) {
        return render_mcp(exec);
    }

    match exec.kind {
        ExecKind::Read => emit_shell(exec, &["cat"]),
        ExecKind::Ls => emit_shell(exec, &["ls"]),
        ExecKind::Grep => emit_grep(exec),
        ExecKind::Shell => emit_shell_command(exec),
        ExecKind::ShellStream => emit_exec_command(exec),
        ExecKind::BackgroundShellSpawn => RenderedToolCall::Refuse {
            exec_id: exec.exec_id.clone(),
            reason:
                "Codex CLI exec_command does not natively background; use a separate orchestration"
                    .into(),
            code: refuse_code::CLIENT_CAPABILITY_UNSUPPORTED,
        },
        ExecKind::WriteShellStdin => emit_write_stdin(exec),
        ExecKind::Write => RenderedToolCall::Refuse {
            exec_id: exec.exec_id.clone(),
            reason: "Cursor Write exec carries path only; body bytes are not decoded".into(),
            code: refuse_code::MISSING_REQUIRED_FIELD,
        },
        ExecKind::Delete => emit_delete_patch(exec),
        ExecKind::Diagnostics => RenderedToolCall::Refuse {
            exec_id: exec.exec_id.clone(),
            reason: "Cursor Diagnostics exec args shape is not decoded".into(),
            code: refuse_code::SHAPE_UNKNOWN_PENDING_LIVE_PHASE0,
        },
        ExecKind::RequestContext => RenderedToolCall::Refuse {
            exec_id: exec.exec_id.clone(),
            reason: "RequestContext is proxy-internal and should not reach the renderer".into(),
            code: refuse_code::CLIENT_CAPABILITY_UNSUPPORTED,
        },
        ExecKind::ListMcpResources => RenderedToolCall::Emit {
            tool_name: "list_mcp_resources".into(),
            arguments: json!({}),
            tool_call_id: exec.exec_id.clone(),
        },
        ExecKind::ReadMcpResource => emit_read_mcp_resource(exec),
        ExecKind::Fetch => emit_fetch(exec),
        ExecKind::RecordScreen | ExecKind::ComputerUse => RenderedToolCall::Refuse {
            exec_id: exec.exec_id.clone(),
            reason: format!(
                "Cursor exec kind {:?} unsupported by CodexCli profile",
                exec.kind
            ),
            code: refuse_code::UNSUPPORTED_EXEC_KIND,
        },
        ExecKind::Other(field) => RenderedToolCall::Refuse {
            exec_id: exec.exec_id.clone(),
            reason: format!("Cursor exec kind Other({field}) unsupported by CodexCli profile"),
            code: refuse_code::UNSUPPORTED_EXEC_KIND,
        },
        ExecKind::Mcp => unreachable!(),
    }
}

fn emit_shell(exec: &ExecRequest, prefix: &[&str]) -> RenderedToolCall {
    let path = read_string_field(&exec.args, 1).unwrap_or_default();
    let mut cmd: Vec<String> = prefix.iter().map(|s| (*s).to_string()).collect();
    cmd.push(path);
    RenderedToolCall::Emit {
        tool_name: "shell_command".into(),
        arguments: json!({ "cmd": cmd }),
        tool_call_id: exec.exec_id.clone(),
    }
}

fn emit_grep(exec: &ExecRequest) -> RenderedToolCall {
    let pattern = read_string_field(&exec.args, 1).unwrap_or_default();
    let path = read_string_field(&exec.args, 2).unwrap_or_default();
    RenderedToolCall::Emit {
        tool_name: "shell_command".into(),
        arguments: json!({ "cmd": ["grep", "-rn", pattern, path] }),
        tool_call_id: exec.exec_id.clone(),
    }
}

fn emit_shell_command(exec: &ExecRequest) -> RenderedToolCall {
    let command = read_string_field(&exec.args, 1).unwrap_or_default();
    let workdir = read_string_field(&exec.args, 2).unwrap_or_default();
    let cmd = if command.is_empty() {
        vec!["true".to_string()]
    } else {
        vec!["bash".to_string(), "-c".to_string(), command]
    };
    let args = if workdir.is_empty() {
        json!({ "cmd": cmd })
    } else {
        json!({ "cmd": cmd, "workdir": workdir })
    };
    RenderedToolCall::Emit {
        tool_name: "shell_command".into(),
        arguments: args,
        tool_call_id: exec.exec_id.clone(),
    }
}

fn emit_exec_command(exec: &ExecRequest) -> RenderedToolCall {
    let command = read_string_field(&exec.args, 1).unwrap_or_default();
    let workdir = read_string_field(&exec.args, 2).unwrap_or_default();
    let cmd = if command.is_empty() {
        vec!["true".to_string()]
    } else {
        vec!["bash".to_string(), "-c".to_string(), command]
    };
    let args = if workdir.is_empty() {
        json!({ "cmd": cmd })
    } else {
        json!({ "cmd": cmd, "workdir": workdir })
    };
    RenderedToolCall::Emit {
        tool_name: "exec_command".into(),
        arguments: args,
        tool_call_id: exec.exec_id.clone(),
    }
}

fn emit_write_stdin(exec: &ExecRequest) -> RenderedToolCall {
    let shell_id = read_u64_field(&exec.args, 1).unwrap_or_default();
    let input = read_string_field(&exec.args, 2).unwrap_or_default();
    RenderedToolCall::Emit {
        tool_name: "write_stdin".into(),
        arguments: json!({ "shell_id": shell_id, "input": input }),
        tool_call_id: exec.exec_id.clone(),
    }
}

fn emit_delete_patch(exec: &ExecRequest) -> RenderedToolCall {
    let path = read_string_field(&exec.args, 1).unwrap_or_default();
    let patch = format!("*** Begin Patch\n*** Delete File: {path}\n*** End Patch\n");
    RenderedToolCall::Emit {
        tool_name: "apply_patch".into(),
        arguments: json!({ "patch": patch }),
        tool_call_id: exec.exec_id.clone(),
    }
}

fn emit_read_mcp_resource(exec: &ExecRequest) -> RenderedToolCall {
    let server = read_string_field(&exec.args, 1).unwrap_or_default();
    let uri = read_string_field(&exec.args, 2).unwrap_or_default();
    RenderedToolCall::Emit {
        tool_name: "read_mcp_resource".into(),
        arguments: json!({ "server": server, "uri": uri }),
        tool_call_id: exec.exec_id.clone(),
    }
}

fn emit_fetch(exec: &ExecRequest) -> RenderedToolCall {
    let url = read_string_field(&exec.args, 1).unwrap_or_default();
    RenderedToolCall::Emit {
        tool_name: "shell_command".into(),
        arguments: json!({ "cmd": ["curl", url] }),
        tool_call_id: exec.exec_id.clone(),
    }
}

fn render_mcp(exec: &ExecRequest) -> RenderedToolCall {
    let (mcp_tool_name, tool_call_id, arguments) = decode_exec_public_tool_call(exec);
    let server = read_string_field(&exec.args, 4).unwrap_or_default();
    if native_tools::is_cursor_codebase_search(&mcp_tool_name) {
        return RenderedToolCall::Refuse {
            exec_id: exec.exec_id.clone(),
            reason: "cursor_codebase_search is proxy-internal and must be handled before Codex rendering"
                .into(),
            code: refuse_code::CLIENT_CAPABILITY_UNSUPPORTED,
        };
    }
    if native_tools::is_synthetic_mcp_native_leak(ClientProfile::CodexCli, &server, &mcp_tool_name)
    {
        return RenderedToolCall::Refuse {
            exec_id: exec.exec_id.clone(),
            reason: format!("Codex native tool {mcp_tool_name} was leaked as Cursor MCP"),
            code: refuse_code::NATIVE_TOOL_LEAKED_AS_MCP,
        };
    }
    let namespaced =
        native_tools::profile_mcp_tool_name(ClientProfile::CodexCli, &server, &mcp_tool_name);
    RenderedToolCall::Emit {
        tool_name: namespaced,
        arguments,
        tool_call_id,
    }
}
