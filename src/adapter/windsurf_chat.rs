use std::time::{SystemTime, UNIX_EPOCH};

use bytes::Bytes;
use serde_json::{json, Map, Value};
use uuid::Uuid;

use crate::{AppError, AppResult};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ToolCallPlan {
    pub name: String,
    pub arguments: Value,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ToolPlan {
    Final(String),
    ToolCalls(Vec<ToolCallPlan>),
}

pub fn validate_request(value: &Value) -> AppResult<()> {
    let object = value
        .as_object()
        .ok_or_else(|| AppError::BadRequest("request body must be a JSON object".into()))?;
    let messages = object
        .get("messages")
        .and_then(Value::as_array)
        .ok_or_else(|| AppError::BadRequest("messages must be an array".into()))?;
    if messages.is_empty() {
        return Err(AppError::BadRequest("messages is required".into()));
    }
    Ok(())
}

pub fn is_stream_request(value: &Value) -> bool {
    value
        .get("stream")
        .and_then(Value::as_bool)
        .unwrap_or(false)
}

pub fn has_tool_context(value: &Value) -> bool {
    let Some(object) = value.as_object() else {
        return false;
    };
    if object
        .get("tools")
        .and_then(Value::as_array)
        .is_some_and(|tools| !tools.is_empty())
    {
        return true;
    }
    object
        .get("messages")
        .and_then(Value::as_array)
        .is_some_and(|messages| {
            messages.iter().any(|message| {
                message.get("role").and_then(Value::as_str) == Some("tool")
                    || message
                        .get("tool_calls")
                        .and_then(Value::as_array)
                        .is_some_and(|calls| !calls.is_empty())
            })
        })
}

pub fn tool_planning_request(value: &Value, upstream_model: &str) -> AppResult<Value> {
    validate_request(value)?;
    Ok(json!({
        "model": upstream_model,
        "messages": [{
            "role": "user",
            "content": build_tool_prompt(value)?
        }],
        "stream": false
    }))
}

pub fn parse_tool_plan(output: &str) -> Option<ToolPlan> {
    let json_lines = output
        .lines()
        .map(str::trim)
        .filter(|line| line.starts_with('{') && line.ends_with('}'))
        .collect::<Vec<_>>();
    if json_lines.len() > 1 {
        let mut calls = Vec::new();
        for line in json_lines {
            let parsed: Value = serde_json::from_str(line).ok()?;
            if let Some(call) = tool_call_from_value(&parsed) {
                calls.push(call);
            }
        }
        if !calls.is_empty() {
            return Some(ToolPlan::ToolCalls(calls));
        }
    }

    let start = output.find('{')?;
    let end = output.rfind('}')?;
    if end <= start {
        return None;
    }
    let parsed: Value = serde_json::from_str(&output[start..=end]).ok()?;
    let object = parsed.as_object()?;

    if object.get("action").and_then(Value::as_str) == Some("final") {
        if let Some(content) = object.get("content").and_then(Value::as_str) {
            return Some(ToolPlan::Final(content.to_string()));
        }
    }
    if let Some(call) = tool_call_from_value(&parsed) {
        return Some(ToolPlan::ToolCalls(vec![call]));
    }
    if let Some(calls) = object
        .get("tool_calls")
        .and_then(Value::as_array)
        .map(|calls| {
            calls
                .iter()
                .filter_map(tool_call_from_value)
                .collect::<Vec<_>>()
        })
    {
        if !calls.is_empty() {
            return Some(ToolPlan::ToolCalls(calls));
        }
    }
    None
}

pub fn non_stream_text_response(model: &str, content: impl Into<String>) -> Value {
    json!({
        "id": chat_completion_id(),
        "object": "chat.completion",
        "created": created_timestamp(),
        "model": model,
        "choices": [{
            "index": 0,
            "message": { "role": "assistant", "content": content.into() },
            "finish_reason": "stop"
        }],
        "usage": zero_usage()
    })
}

pub fn non_stream_tool_response(model: &str, calls: &[ToolCallPlan]) -> Value {
    json!({
        "id": chat_completion_id(),
        "object": "chat.completion",
        "created": created_timestamp(),
        "model": model,
        "choices": [{
            "index": 0,
            "message": {
                "role": "assistant",
                "content": "",
                "tool_calls": chat_tool_calls(calls, false)
            },
            "finish_reason": "tool_calls"
        }],
        "usage": zero_usage()
    })
}

pub fn initial_stream_frame(id: &str, created: u64, model: &str) -> Bytes {
    sse_json(&json!({
        "id": id,
        "object": "chat.completion.chunk",
        "created": created,
        "model": model,
        "choices": [{ "index": 0, "delta": { "role": "assistant" }, "finish_reason": null }]
    }))
}

pub fn content_stream_frame(id: &str, created: u64, model: &str, delta: &str) -> Bytes {
    sse_json(&json!({
        "id": id,
        "object": "chat.completion.chunk",
        "created": created,
        "model": model,
        "choices": [{ "index": 0, "delta": { "content": delta }, "finish_reason": null }]
    }))
}

pub fn tool_calls_stream_frame(
    id: &str,
    created: u64,
    model: &str,
    calls: &[ToolCallPlan],
) -> Bytes {
    sse_json(&json!({
        "id": id,
        "object": "chat.completion.chunk",
        "created": created,
        "model": model,
        "choices": [{
            "index": 0,
            "delta": { "tool_calls": chat_tool_calls(calls, true) },
            "finish_reason": null
        }]
    }))
}

pub fn finish_stream_frame(id: &str, created: u64, model: &str, finish_reason: &str) -> Bytes {
    let mut bytes = sse_json(&json!({
        "id": id,
        "object": "chat.completion.chunk",
        "created": created,
        "model": model,
        "choices": [{ "index": 0, "delta": {}, "finish_reason": finish_reason }]
    }))
    .to_vec();
    bytes.extend_from_slice(b"data: [DONE]\n\n");
    Bytes::from(bytes)
}

pub fn error_stream_frame(id: &str, created: u64, model: &str, error: &AppError) -> Bytes {
    let mut bytes = sse_json(&json!({
        "id": id,
        "object": "chat.completion.chunk",
        "created": created,
        "model": model,
        "choices": [],
        "error": {
            "message": error.to_string(),
            "type": error.error_type(),
            "code": error.code()
        }
    }))
    .to_vec();
    bytes.extend_from_slice(b"data: [DONE]\n\n");
    Bytes::from(bytes)
}

pub fn chat_completion_id() -> String {
    format!("chatcmpl-{}", Uuid::new_v4().simple())
}

pub fn created_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default()
}

fn build_tool_prompt(value: &Value) -> AppResult<String> {
    let object = value
        .as_object()
        .ok_or_else(|| AppError::BadRequest("request body must be a JSON object".into()))?;
    let tools = object
        .get("tools")
        .and_then(Value::as_array)
        .map(|tools| {
            tools
                .iter()
                .map(|tool| {
                    let function = tool.get("function").unwrap_or(&Value::Null);
                    let name = function
                        .get("name")
                        .and_then(Value::as_str)
                        .unwrap_or("unknown");
                    let description = function
                        .get("description")
                        .and_then(Value::as_str)
                        .map(|value| format!(": {value}"))
                        .unwrap_or_default();
                    let parameters = function.get("parameters").unwrap_or(&Value::Null);
                    format!("- {name}{description}\n{parameters}")
                })
                .collect::<Vec<_>>()
                .join("\n")
        })
        .filter(|text| !text.is_empty())
        .unwrap_or_else(|| "(none)".to_string());

    let messages = object
        .get("messages")
        .and_then(Value::as_array)
        .ok_or_else(|| AppError::BadRequest("messages must be an array".into()))?;
    let conversation = messages
        .iter()
        .map(|message| {
            let role = message
                .get("role")
                .and_then(Value::as_str)
                .unwrap_or("user");
            if role == "assistant" {
                if let Some(tool_calls) = message.get("tool_calls") {
                    if tool_calls.as_array().is_some_and(|items| !items.is_empty()) {
                        return format!("ASSISTANT TOOL_CALLS: {tool_calls}");
                    }
                }
            }
            if role == "tool" {
                let suffix = message
                    .get("tool_call_id")
                    .and_then(Value::as_str)
                    .map(|value| format!(" {value}"))
                    .unwrap_or_default();
                return format!(
                    "TOOL RESULT{suffix}: {}",
                    text_from_content(message.get("content"))
                );
            }
            format!(
                "{}: {}",
                role.to_ascii_uppercase(),
                text_from_content(message.get("content"))
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n");

    Ok([
        "You are running inside an OpenAI-compatible tool-calling client and can either call tools or answer directly.",
        "Return exactly one JSON object and no prose.",
        "To call tools: {\"action\":\"tool_call\",\"tool_calls\":[{\"name\":\"tool_name\",\"arguments\":{}}]}",
        "To answer: {\"action\":\"final\",\"content\":\"...\"}",
        "",
        "Available tools:",
        &tools,
        "",
        "Conversation:",
        &conversation,
    ]
    .join("\n"))
}

fn tool_call_from_value(value: &Value) -> Option<ToolCallPlan> {
    let object = value.as_object()?;
    let name = object.get("name").and_then(Value::as_str)?;
    Some(ToolCallPlan {
        name: name.to_string(),
        arguments: parse_tool_arguments(object.get("arguments").cloned().unwrap_or(json!({}))),
    })
}

fn parse_tool_arguments(value: Value) -> Value {
    match value {
        Value::String(text) => serde_json::from_str(&text).unwrap_or(Value::String(text)),
        other => other,
    }
}

fn chat_tool_calls(calls: &[ToolCallPlan], include_index: bool) -> Value {
    Value::Array(
        calls
            .iter()
            .enumerate()
            .map(|(index, call)| {
                let mut object = Map::new();
                if include_index {
                    object.insert("index".into(), json!(index));
                }
                object.insert(
                    "id".into(),
                    Value::String(format!("call_{}_{}", Uuid::new_v4().simple(), index)),
                );
                object.insert("type".into(), Value::String("function".into()));
                object.insert(
                    "function".into(),
                    json!({
                        "name": call.name,
                        "arguments": stringify_tool_arguments(&call.arguments)
                    }),
                );
                Value::Object(object)
            })
            .collect(),
    )
}

fn stringify_tool_arguments(value: &Value) -> String {
    match value {
        Value::String(text) => text.clone(),
        other => other.to_string(),
    }
}

fn text_from_content(value: Option<&Value>) -> String {
    match value {
        Some(Value::String(text)) => text.clone(),
        Some(Value::Array(parts)) => parts
            .iter()
            .filter(|part| part.get("type").and_then(Value::as_str) == Some("text"))
            .filter_map(|part| part.get("text").and_then(Value::as_str))
            .collect::<Vec<_>>()
            .join("\n"),
        _ => String::new(),
    }
}

fn zero_usage() -> Value {
    json!({ "prompt_tokens": 0, "completion_tokens": 0, "total_tokens": 0 })
}

fn sse_json(value: &Value) -> Bytes {
    Bytes::from(format!("data: {value}\n\n"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tool_plan_parser_accepts_action_wrapped_and_direct_calls() {
        assert_eq!(
            parse_tool_plan(r#"{"action":"final","content":"done"}"#),
            Some(ToolPlan::Final("done".into()))
        );
        assert_eq!(
            parse_tool_plan(r#"{"name":"lookup","arguments":"{\"q\":\"x\"}"}"#),
            Some(ToolPlan::ToolCalls(vec![ToolCallPlan {
                name: "lookup".into(),
                arguments: json!({ "q": "x" })
            }]))
        );
        assert_eq!(
            parse_tool_plan(
                r#"{"action":"tool_call","tool_calls":[{"name":"lookup","arguments":{"q":"x"}}]}"#
            ),
            Some(ToolPlan::ToolCalls(vec![ToolCallPlan {
                name: "lookup".into(),
                arguments: json!({ "q": "x" })
            }]))
        );
    }

    #[test]
    fn tool_prompt_preserves_tool_result_loop_context() {
        let prompt = build_tool_prompt(&json!({
            "model": "swe-1.6",
            "tools": [{
                "type": "function",
                "function": {
                    "name": "lookup",
                    "description": "search",
                    "parameters": { "type": "object" }
                }
            }],
            "messages": [
                { "role": "user", "content": "find it" },
                { "role": "assistant", "content": null, "tool_calls": [{ "id": "call_1", "type": "function", "function": { "name": "lookup", "arguments": "{}" }}] },
                { "role": "tool", "tool_call_id": "call_1", "content": "result" }
            ]
        }))
        .unwrap();

        assert!(prompt.contains("- lookup: search"));
        assert!(prompt.contains("ASSISTANT TOOL_CALLS"));
        assert!(prompt.contains("TOOL RESULT call_1: result"));
    }
}
