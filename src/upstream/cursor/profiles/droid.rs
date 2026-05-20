//! Droid profile renderer.
//!
//! Maps Cursor exec requests to Factory Droid native tool calls.

use super::native_tools;
use super::proto_helpers::read_string_field;
use super::{refuse_code, RenderedToolCall};
use crate::upstream::cursor::client_profile::ClientProfile;
use crate::upstream::cursor::proto::{decode_exec_public_tool_call, ExecKind, ExecRequest};
use serde_json::json;

pub fn render(exec: &ExecRequest) -> RenderedToolCall {
    if matches!(exec.kind, ExecKind::Mcp) {
        return render_mcp(exec);
    }

    match exec.kind {
        ExecKind::Read => emit_read(exec),
        ExecKind::Ls => emit_ls(exec),
        ExecKind::Grep => emit_grep(exec),
        ExecKind::Shell | ExecKind::ShellStream => emit_execute_shell(exec, false),
        ExecKind::BackgroundShellSpawn => emit_execute_shell(exec, true),
        ExecKind::WriteShellStdin => RenderedToolCall::Refuse {
            exec_id: exec.exec_id.clone(),
            reason: "Droid has no analog for WriteShellStdin; cannot address a running shell PID"
                .into(),
            code: refuse_code::CLIENT_CAPABILITY_UNSUPPORTED,
        },
        ExecKind::Write => RenderedToolCall::Refuse {
            exec_id: exec.exec_id.clone(),
            reason: "Droid Create requires content; Cursor Write exec carries path only".into(),
            code: refuse_code::MISSING_REQUIRED_FIELD,
        },
        ExecKind::Delete => emit_execute_delete(exec),
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
        ExecKind::ListMcpResources => RenderedToolCall::Refuse {
            exec_id: exec.exec_id.clone(),
            reason: "Droid has no list_mcp_resources analog; MCP servers expose their own listing"
                .into(),
            code: refuse_code::CLIENT_CAPABILITY_UNSUPPORTED,
        },
        ExecKind::ReadMcpResource => RenderedToolCall::Refuse {
            exec_id: exec.exec_id.clone(),
            reason: "Droid has no read_mcp_resource analog; MCP servers expose their own resources"
                .into(),
            code: refuse_code::CLIENT_CAPABILITY_UNSUPPORTED,
        },
        ExecKind::Fetch => emit_fetch(exec),
        ExecKind::RecordScreen | ExecKind::ComputerUse => RenderedToolCall::Refuse {
            exec_id: exec.exec_id.clone(),
            reason: format!(
                "Cursor exec kind {:?} unsupported by Droid profile",
                exec.kind
            ),
            code: refuse_code::UNSUPPORTED_EXEC_KIND,
        },
        ExecKind::Other(field) => RenderedToolCall::Refuse {
            exec_id: exec.exec_id.clone(),
            reason: format!("Cursor exec kind Other({field}) unsupported by Droid profile"),
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

fn emit_ls(exec: &ExecRequest) -> RenderedToolCall {
    let path = read_string_field(&exec.args, 1).unwrap_or_default();
    RenderedToolCall::Emit {
        tool_name: "LS".into(),
        arguments: json!({ "path": path }),
        tool_call_id: exec.exec_id.clone(),
    }
}

fn path_to_glob(path: &str) -> String {
    if path.is_empty() {
        return String::new();
    }
    let has_wildcard = path.contains('*') || path.contains('?');
    let last_segment = path.split('/').next_back().unwrap_or(path);
    let is_file = last_segment.contains('.') && !last_segment.starts_with('.');
    if has_wildcard || is_file {
        path.to_string()
    } else {
        let trimmed = path.trim_end_matches('/');
        format!("{trimmed}/**/*")
    }
}

fn emit_grep(exec: &ExecRequest) -> RenderedToolCall {
    let pattern = read_string_field(&exec.args, 1).unwrap_or_default();
    let path = read_string_field(&exec.args, 2).unwrap_or_default();
    let mut arguments = serde_json::Map::new();
    arguments.insert("pattern".into(), json!(pattern));
    let glob = path_to_glob(&path);
    if !glob.is_empty() {
        arguments.insert("glob".into(), json!(glob));
    }
    RenderedToolCall::Emit {
        tool_name: "Grep".into(),
        arguments: serde_json::Value::Object(arguments),
        tool_call_id: exec.exec_id.clone(),
    }
}

fn emit_execute_shell(exec: &ExecRequest, background: bool) -> RenderedToolCall {
    let command = read_string_field(&exec.args, 1).unwrap_or_default();
    let mut arguments = serde_json::Map::new();
    arguments.insert("command".into(), json!(command));
    if background {
        arguments.insert("fireAndForget".into(), json!(true));
    }
    arguments.insert("riskLevel".into(), json!("medium"));
    arguments.insert(
        "riskLevelReason".into(),
        json!("automated proxy invocation"),
    );
    tracing::debug!(
        target: "cursor.profiles",
        cursor_synthetic_default = "droid_execute_risk",
        cursor_exec_kind = ?exec.kind,
        cursor_tool_call_id = %exec.exec_id,
        cursor_tool_name_emitted = "Execute",
        "synthesized Droid Execute risk metadata",
    );
    RenderedToolCall::Emit {
        tool_name: "Execute".into(),
        arguments: serde_json::Value::Object(arguments),
        tool_call_id: exec.exec_id.clone(),
    }
}

fn emit_execute_delete(exec: &ExecRequest) -> RenderedToolCall {
    let path = read_string_field(&exec.args, 1).unwrap_or_default();
    let command = format!("rm {path}");
    let arguments = json!({
        "command": command,
        "riskLevel": "high",
        "riskLevelReason": "file deletion requested by Cursor exec",
    });
    tracing::debug!(
        target: "cursor.profiles",
        cursor_synthetic_default = "droid_execute_risk",
        cursor_exec_kind = ?exec.kind,
        cursor_tool_call_id = %exec.exec_id,
        cursor_tool_name_emitted = "Execute",
        "synthesized Droid Execute risk metadata for delete",
    );
    RenderedToolCall::Emit {
        tool_name: "Execute".into(),
        arguments,
        tool_call_id: exec.exec_id.clone(),
    }
}

fn emit_fetch(exec: &ExecRequest) -> RenderedToolCall {
    let url = read_string_field(&exec.args, 1).unwrap_or_default();
    RenderedToolCall::Emit {
        tool_name: "FetchUrl".into(),
        arguments: json!({ "url": url }),
        tool_call_id: exec.exec_id.clone(),
    }
}

fn render_mcp(exec: &ExecRequest) -> RenderedToolCall {
    let (mcp_tool_name, tool_call_id, arguments) = decode_exec_public_tool_call(exec);
    let server = read_string_field(&exec.args, 4).unwrap_or_default();
    if native_tools::is_cursor_codebase_search(&mcp_tool_name) {
        return RenderedToolCall::Refuse {
            exec_id: exec.exec_id.clone(),
            reason: "cursor_codebase_search is proxy-internal and must be handled before Droid rendering"
                .into(),
            code: refuse_code::CLIENT_CAPABILITY_UNSUPPORTED,
        };
    }
    if native_tools::is_synthetic_mcp_native_leak(ClientProfile::Droid, &server, &mcp_tool_name) {
        return RenderedToolCall::Refuse {
            exec_id: exec.exec_id.clone(),
            reason: format!("Droid native tool {mcp_tool_name} was leaked as Cursor MCP"),
            code: refuse_code::NATIVE_TOOL_LEAKED_AS_MCP,
        };
    }
    let namespaced =
        native_tools::profile_mcp_tool_name(ClientProfile::Droid, &server, &mcp_tool_name);
    RenderedToolCall::Emit {
        tool_name: namespaced,
        arguments,
        tool_call_id,
    }
}
