//! ClaudeCode profile renderer.
//!
//! Maps Cursor exec requests to Claude Code native tool calls.

use super::proto_helpers::read_string_field;
use super::{refuse_code, RenderedToolCall};
use crate::upstream::cursor::proto::{decode_exec_public_tool_call, ExecKind, ExecRequest};
use serde_json::json;

const WEBFETCH_DEFAULT_PROMPT: &str =
    "Summarize the page contents and return the relevant section for the user's request.";

pub fn render(exec: &ExecRequest) -> RenderedToolCall {
    if matches!(exec.kind, ExecKind::Mcp) {
        return render_mcp(exec);
    }

    match exec.kind {
        ExecKind::Read => emit_read(exec),
        ExecKind::Ls => emit_ls_via_bash(exec),
        ExecKind::Grep => emit_grep(exec),
        ExecKind::Shell | ExecKind::ShellStream => emit_bash_foreground(exec),
        ExecKind::BackgroundShellSpawn => emit_bash_background(exec),
        ExecKind::WriteShellStdin => RenderedToolCall::Refuse {
            exec_id: exec.exec_id.clone(),
            reason: "Claude Code BashOutput reads output only; cannot write stdin".into(),
            code: refuse_code::CLIENT_CAPABILITY_UNSUPPORTED,
        },
        ExecKind::Write => RenderedToolCall::Refuse {
            exec_id: exec.exec_id.clone(),
            reason: "Claude Code Write requires content; Cursor wire lacks body bytes".into(),
            code: refuse_code::MISSING_REQUIRED_FIELD,
        },
        ExecKind::Delete => RenderedToolCall::Refuse {
            exec_id: exec.exec_id.clone(),
            reason:
                "Claude Code has no Delete tool; refusing to synthesize destructive Bash without context"
                    .into(),
            code: refuse_code::CLIENT_CAPABILITY_UNSUPPORTED,
        },
        ExecKind::Diagnostics => RenderedToolCall::Refuse {
            exec_id: exec.exec_id.clone(),
            reason: "Cursor Diagnostics arg shape is not decoded".into(),
            code: refuse_code::SHAPE_UNKNOWN_PENDING_LIVE_PHASE0,
        },
        ExecKind::RequestContext => RenderedToolCall::Refuse {
            exec_id: exec.exec_id.clone(),
            reason: "RequestContext is proxy-internal".into(),
            code: refuse_code::CLIENT_CAPABILITY_UNSUPPORTED,
        },
        ExecKind::ListMcpResources => RenderedToolCall::Emit {
            tool_name: "ListMcpResourcesTool".into(),
            arguments: json!({ "server": read_string_field(&exec.args, 1).unwrap_or_default() }),
            tool_call_id: exec.exec_id.clone(),
        },
        ExecKind::ReadMcpResource => emit_read_mcp_resource(exec),
        ExecKind::Fetch => emit_webfetch(exec),
        ExecKind::RecordScreen | ExecKind::ComputerUse => RenderedToolCall::Refuse {
            exec_id: exec.exec_id.clone(),
            reason: format!(
                "Cursor exec kind {:?} unsupported by ClaudeCode profile",
                exec.kind
            ),
            code: refuse_code::UNSUPPORTED_EXEC_KIND,
        },
        ExecKind::Other(field) => RenderedToolCall::Refuse {
            exec_id: exec.exec_id.clone(),
            reason: format!("Cursor exec kind Other({field}) unsupported by ClaudeCode profile"),
            code: refuse_code::UNSUPPORTED_EXEC_KIND,
        },
        ExecKind::Mcp => unreachable!(),
    }
}

fn emit_read(exec: &ExecRequest) -> RenderedToolCall {
    let path = read_string_field(&exec.args, 1).unwrap_or_default();
    RenderedToolCall::Emit {
        tool_name: "Read".into(),
        arguments: json!({ "file_path": path }),
        tool_call_id: exec.exec_id.clone(),
    }
}

fn emit_ls_via_bash(exec: &ExecRequest) -> RenderedToolCall {
    let path = read_string_field(&exec.args, 1).unwrap_or_default();
    RenderedToolCall::Emit {
        tool_name: "Bash".into(),
        arguments: json!({ "command": format!("ls {path}") }),
        tool_call_id: exec.exec_id.clone(),
    }
}

fn emit_grep(exec: &ExecRequest) -> RenderedToolCall {
    let pattern = read_string_field(&exec.args, 1).unwrap_or_default();
    let path = read_string_field(&exec.args, 2).unwrap_or_default();
    let output_mode = read_string_field(&exec.args, 3).unwrap_or_default();
    let mut args = serde_json::Map::new();
    args.insert("pattern".into(), json!(pattern));
    if !path.is_empty() {
        args.insert("path".into(), json!(path));
    }
    if !output_mode.is_empty() {
        args.insert("output_mode".into(), json!(output_mode));
    }
    RenderedToolCall::Emit {
        tool_name: "Grep".into(),
        arguments: serde_json::Value::Object(args),
        tool_call_id: exec.exec_id.clone(),
    }
}

fn emit_bash_foreground(exec: &ExecRequest) -> RenderedToolCall {
    let command = read_string_field(&exec.args, 1).unwrap_or_default();
    RenderedToolCall::Emit {
        tool_name: "Bash".into(),
        arguments: json!({ "command": command }),
        tool_call_id: exec.exec_id.clone(),
    }
}

fn emit_bash_background(exec: &ExecRequest) -> RenderedToolCall {
    let command = read_string_field(&exec.args, 1).unwrap_or_default();
    RenderedToolCall::Emit {
        tool_name: "Bash".into(),
        arguments: json!({ "command": command, "run_in_background": true }),
        tool_call_id: exec.exec_id.clone(),
    }
}

fn emit_read_mcp_resource(exec: &ExecRequest) -> RenderedToolCall {
    let server = read_string_field(&exec.args, 1).unwrap_or_default();
    let uri = read_string_field(&exec.args, 2).unwrap_or_default();
    RenderedToolCall::Emit {
        tool_name: "ReadMcpResourceTool".into(),
        arguments: json!({ "server": server, "uri": uri }),
        tool_call_id: exec.exec_id.clone(),
    }
}

fn emit_webfetch(exec: &ExecRequest) -> RenderedToolCall {
    let url = read_string_field(&exec.args, 1).unwrap_or_default();
    tracing::debug!(
        target: "cursor.profiles.claude_code",
        cursor_synthetic_default = "claude_webfetch_prompt",
        "WebFetch synthesized default prompt",
    );
    RenderedToolCall::Emit {
        tool_name: "WebFetch".into(),
        arguments: json!({ "url": url, "prompt": WEBFETCH_DEFAULT_PROMPT }),
        tool_call_id: exec.exec_id.clone(),
    }
}

fn render_mcp(exec: &ExecRequest) -> RenderedToolCall {
    let (mcp_tool_name, tool_call_id, arguments) = decode_exec_public_tool_call(exec);
    let server = read_string_field(&exec.args, 4).unwrap_or_default();
    let namespaced = if server.is_empty() {
        mcp_tool_name
    } else {
        format!("mcp__{server}__{mcp_tool_name}")
    };
    RenderedToolCall::Emit {
        tool_name: namespaced,
        arguments,
        tool_call_id,
    }
}
