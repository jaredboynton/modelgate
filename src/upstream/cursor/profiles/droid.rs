//! Droid profile renderer.
//!
//! Maps Cursor exec requests to Factory Droid native tool calls.

use super::proto_helpers::read_string_field;
use super::{refuse_code, RenderedToolCall};
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
        arguments: json!({ "path": path }),
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

fn emit_grep(exec: &ExecRequest) -> RenderedToolCall {
    let pattern = read_string_field(&exec.args, 1).unwrap_or_default();
    let path = read_string_field(&exec.args, 2).unwrap_or_default();
    let output_mode = read_string_field(&exec.args, 3).unwrap_or_default();
    let mut arguments = serde_json::Map::new();
    arguments.insert("pattern".into(), json!(pattern));
    arguments.insert("path".into(), json!(path));
    if !output_mode.is_empty() {
        arguments.insert("output_mode".into(), json!(output_mode));
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
        arguments.insert("background".into(), json!(true));
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
    if server == "opencode" && mcp_tool_name == "Read" {
        return emit_opencode_read(tool_call_id, arguments);
    }
    if server == "opencode" && mcp_tool_name == "TodoWrite" {
        return emit_opencode_todo_write(tool_call_id, arguments);
    }
    let namespaced = if server.is_empty() {
        mcp_tool_name
    } else {
        format!("{server}___{mcp_tool_name}")
    };
    RenderedToolCall::Emit {
        tool_name: namespaced,
        arguments,
        tool_call_id,
    }
}

fn emit_opencode_read(tool_call_id: String, arguments: serde_json::Value) -> RenderedToolCall {
    let mut out = arguments.as_object().cloned().unwrap_or_default();
    if !out.contains_key("file_path") {
        if let Some(path) = out.remove("path") {
            out.insert("file_path".into(), path);
        }
    }
    RenderedToolCall::Emit {
        tool_name: "Read".into(),
        arguments: serde_json::Value::Object(out),
        tool_call_id,
    }
}

fn emit_opencode_todo_write(
    tool_call_id: String,
    arguments: serde_json::Value,
) -> RenderedToolCall {
    let mut out = serde_json::Map::new();
    if let Some(todos) = arguments.get("todos") {
        let todos = todos
            .as_str()
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| todos.to_string());
        out.insert("todos".into(), json!(todos));
    }
    if let Some(merge) = arguments.get("merge") {
        out.insert("merge".into(), merge.clone());
    }
    RenderedToolCall::Emit {
        tool_name: "TodoWrite".into(),
        arguments: serde_json::Value::Object(out),
        tool_call_id,
    }
}
