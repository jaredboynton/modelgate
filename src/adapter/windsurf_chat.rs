use std::time::{SystemTime, UNIX_EPOCH};

use bytes::Bytes;
use serde_json::{json, Map, Value};
use uuid::Uuid;

use crate::{AppError, AppResult};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum WindsurfClientProfile {
    CodexCli,
    ClaudeCode,
    Droid,
    Devin,
    Other,
}

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

pub fn tool_planning_request(
    value: &Value,
    upstream_model: &str,
    profile: WindsurfClientProfile,
) -> AppResult<Value> {
    validate_request(value)?;
    Ok(json!({
        "model": upstream_model,
        "messages": [{
            "role": "user",
            "content": build_tool_prompt(value, profile)?
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

    if output.trim_start().starts_with("ASSISTANT TOOL_CALLS:") {
        return parse_assistant_tool_calls_block(output);
    }
    if output.contains("ASSISTANT TOOL_CALLS:") {
        return None;
    }

    if let (Some(start), Some(end)) = (output.find('{'), output.rfind('}')) {
        if end > start {
            if let Ok(parsed) = serde_json::from_str::<Value>(&output[start..=end]) {
                if let Some(object) = parsed.as_object() {
                    if object.get("action").and_then(Value::as_str) == Some("final") {
                        if let Some(content) = object.get("content").and_then(Value::as_str) {
                            return Some(ToolPlan::Final(content.to_string()));
                        }
                    }
                    if let Some(call) = tool_call_from_value(&parsed) {
                        return Some(ToolPlan::ToolCalls(vec![call]));
                    }
                    if let Some(calls) =
                        object
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
                }
            }
        }
    }

    parse_droid_style_tool_call_tags(output)
}

fn parse_assistant_tool_calls_block(output: &str) -> Option<ToolPlan> {
    let trimmed = output.trim();
    let calls_json = trimmed.strip_prefix("ASSISTANT TOOL_CALLS:")?.trim();
    let calls = serde_json::from_str::<Value>(calls_json)
        .ok()?
        .as_array()?
        .iter()
        .filter_map(tool_call_from_value)
        .collect::<Vec<_>>();
    if calls.is_empty() {
        None
    } else {
        Some(ToolPlan::ToolCalls(calls))
    }
}

fn parse_droid_style_tool_call_tags(output: &str) -> Option<ToolPlan> {
    const TAG: &str = "<tool_call>";

    let mut calls = Vec::new();
    let mut offset = skip_whitespace(output, 0);
    while offset < output.len() {
        if !output[offset..].starts_with(TAG) {
            return None;
        }
        offset += TAG.len();

        let name_start = offset;
        while offset < output.len() && is_tool_name_continue(output.as_bytes()[offset]) {
            offset += 1;
        }
        if name_start == offset {
            return None;
        }
        let name = &output[name_start..offset];
        if !is_tool_name_start(name.as_bytes()[0]) {
            return None;
        }

        if !output[offset..].starts_with('{') {
            return None;
        }
        let object_end = balanced_json_object_end(output, offset)?;
        let arguments: Value = serde_json::from_str(&output[offset..object_end]).ok()?;
        if !arguments.is_object() {
            return None;
        }
        calls.push(ToolCallPlan {
            name: name.to_string(),
            arguments,
        });
        offset = skip_whitespace(output, object_end);
    }

    if calls.is_empty() {
        None
    } else {
        Some(ToolPlan::ToolCalls(calls))
    }
}

fn balanced_json_object_end(input: &str, start: usize) -> Option<usize> {
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;

    for (relative_index, ch) in input[start..].char_indices() {
        if in_string {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }

        match ch {
            '"' => in_string = true,
            '{' => depth += 1,
            '}' => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(start + relative_index + ch.len_utf8());
                }
            }
            _ => {}
        }
    }
    None
}

fn skip_whitespace(input: &str, mut offset: usize) -> usize {
    while offset < input.len() {
        let byte = input.as_bytes()[offset];
        if !byte.is_ascii_whitespace() {
            break;
        }
        offset += 1;
    }
    offset
}

fn is_tool_name_start(byte: u8) -> bool {
    byte.is_ascii_alphabetic() || byte == b'_'
}

fn is_tool_name_continue(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-'
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

fn build_tool_prompt(value: &Value, profile: WindsurfClientProfile) -> AppResult<String> {
    let object = value
        .as_object()
        .ok_or_else(|| AppError::BadRequest("request body must be a JSON object".into()))?;

    let tools = object
        .get("tools")
        .and_then(Value::as_array)
        .map(|tools_arr| {
            let mut tools_cloned = tools_arr.clone();
            map_client_tools_to_windsurf(&mut tools_cloned, profile);
            tools_cloned
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
                        let mut tc = tool_calls.clone();
                        if let Some(arr) = tc.as_array_mut() {
                            for call in arr {
                                if let Some(func) = call.get_mut("function") {
                                    map_client_tool_call_to_windsurf(func, profile);
                                }
                            }
                        }
                        return format!("ASSISTANT TOOL_CALLS: {tc}");
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
        "After a TOOL RESULT, return action final unless more tool data is required.",
        "",
        "Available tools:",
        &tools,
        "",
        "Conversation:",
        &conversation,
    ]
    .join("\n"))
}

pub fn map_client_tools_to_windsurf(tools: &mut [Value], profile: WindsurfClientProfile) {
    if profile == WindsurfClientProfile::Devin {
        for tool in tools {
            if let Some(function) = tool.get_mut("function") {
                if let Some(name) = function.get("name").and_then(Value::as_str) {
                    match name {
                        "read" => {
                            function["name"] = json!("Read");
                            if let Some(parameters) = function.get_mut("parameters") {
                                if let Some(properties) = parameters
                                    .get_mut("properties")
                                    .and_then(Value::as_object_mut)
                                {
                                    if let Some(path) = properties.remove("path") {
                                        properties.insert("file_path".into(), path);
                                    }
                                    if let Some(Value::Array(required)) =
                                        parameters.get_mut("required")
                                    {
                                        for val in required.iter_mut() {
                                            if val.as_str() == Some("path") {
                                                *val = json!("file_path");
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        "ls" => {
                            function["name"] = json!("LS");
                            if let Some(parameters) = function.get_mut("parameters") {
                                if let Some(properties) = parameters
                                    .get_mut("properties")
                                    .and_then(Value::as_object_mut)
                                {
                                    if let Some(path) = properties.remove("path") {
                                        properties.insert("directory_path".into(), path);
                                    }
                                    if let Some(Value::Array(required)) =
                                        parameters.get_mut("required")
                                    {
                                        for val in required.iter_mut() {
                                            if val.as_str() == Some("path") {
                                                *val = json!("directory_path");
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        "search" | "grep" => {
                            function["name"] = json!("Grep");
                            if let Some(parameters) = function.get_mut("parameters") {
                                if let Some(properties) = parameters
                                    .get_mut("properties")
                                    .and_then(Value::as_object_mut)
                                {
                                    if let Some(path) = properties.remove("path") {
                                        properties.insert("glob_pattern".into(), path);
                                    }
                                    if let Some(Value::Array(required)) =
                                        parameters.get_mut("required")
                                    {
                                        for val in required.iter_mut() {
                                            if val.as_str() == Some("path") {
                                                *val = json!("glob_pattern");
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        "execute" | "shell" => {
                            function["name"] = json!("Execute");
                            if let Some(parameters) = function.get_mut("parameters") {
                                if let Some(properties) = parameters
                                    .get_mut("properties")
                                    .and_then(Value::as_object_mut)
                                {
                                    if let Some(working_directory) =
                                        properties.remove("working_directory")
                                    {
                                        properties.insert("cwd".into(), working_directory);
                                    }
                                    if let Some(Value::Array(required)) =
                                        parameters.get_mut("required")
                                    {
                                        for val in required.iter_mut() {
                                            if val.as_str() == Some("working_directory") {
                                                *val = json!("cwd");
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        "fetch" => {
                            function["name"] = json!("FetchUrl");
                        }
                        "edit" => {
                            function["name"] = json!("Edit");
                            if let Some(parameters) = function.get_mut("parameters") {
                                if let Some(properties) = parameters
                                    .get_mut("properties")
                                    .and_then(Value::as_object_mut)
                                {
                                    if let Some(path) = properties.remove("path") {
                                        properties.insert("file_path".into(), path);
                                    }
                                    if let Some(Value::Array(required)) =
                                        parameters.get_mut("required")
                                    {
                                        for val in required.iter_mut() {
                                            if val.as_str() == Some("path") {
                                                *val = json!("file_path");
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    } else if profile == WindsurfClientProfile::ClaudeCode {
        for tool in tools {
            if let Some(function) = tool.get_mut("function") {
                if let Some(name) = function.get("name").and_then(Value::as_str) {
                    match name {
                        "Bash" => {
                            function["name"] = json!("Execute");
                        }
                        "Grep" => {
                            if let Some(parameters) = function.get_mut("parameters") {
                                if let Some(properties) = parameters
                                    .get_mut("properties")
                                    .and_then(Value::as_object_mut)
                                {
                                    if let Some(path) = properties.remove("path") {
                                        properties.insert("glob_pattern".into(), path);
                                    }
                                    if let Some(Value::Array(required)) =
                                        parameters.get_mut("required")
                                    {
                                        for val in required.iter_mut() {
                                            if val.as_str() == Some("path") {
                                                *val = json!("glob_pattern");
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        "WebFetch" => {
                            function["name"] = json!("FetchUrl");
                        }
                        _ => {}
                    }
                }
            }
        }
    } else if profile == WindsurfClientProfile::CodexCli {
        for tool in tools {
            if let Some(function) = tool.get_mut("function") {
                if let Some(name) = function.get("name").and_then(Value::as_str) {
                    match name {
                        "read_file" => {
                            function["name"] = json!("Read");
                            if let Some(parameters) = function.get_mut("parameters") {
                                if let Some(properties) = parameters
                                    .get_mut("properties")
                                    .and_then(Value::as_object_mut)
                                {
                                    if let Some(path) = properties.remove("path") {
                                        properties.insert("file_path".into(), path);
                                    }
                                    if let Some(Value::Array(required)) =
                                        parameters.get_mut("required")
                                    {
                                        for val in required.iter_mut() {
                                            if val.as_str() == Some("path") {
                                                *val = json!("file_path");
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        "ls" => {
                            function["name"] = json!("LS");
                            if let Some(parameters) = function.get_mut("parameters") {
                                if let Some(properties) = parameters
                                    .get_mut("properties")
                                    .and_then(Value::as_object_mut)
                                {
                                    if let Some(path) = properties.remove("path") {
                                        properties.insert("directory_path".into(), path);
                                    }
                                    if let Some(Value::Array(required)) =
                                        parameters.get_mut("required")
                                    {
                                        for val in required.iter_mut() {
                                            if val.as_str() == Some("path") {
                                                *val = json!("directory_path");
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        "grep" => {
                            function["name"] = json!("Grep");
                            if let Some(parameters) = function.get_mut("parameters") {
                                if let Some(properties) = parameters
                                    .get_mut("properties")
                                    .and_then(Value::as_object_mut)
                                {
                                    if let Some(path) = properties.remove("path") {
                                        properties.insert("glob_pattern".into(), path);
                                    }
                                    if let Some(Value::Array(required)) =
                                        parameters.get_mut("required")
                                    {
                                        for val in required.iter_mut() {
                                            if val.as_str() == Some("path") {
                                                *val = json!("glob_pattern");
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        "shell" => {
                            function["name"] = json!("Execute");
                            if let Some(parameters) = function.get_mut("parameters") {
                                if let Some(properties) = parameters
                                    .get_mut("properties")
                                    .and_then(Value::as_object_mut)
                                {
                                    if let Some(working_directory) =
                                        properties.remove("working_directory")
                                    {
                                        properties.insert("cwd".into(), working_directory);
                                    }
                                    if let Some(Value::Array(required)) =
                                        parameters.get_mut("required")
                                    {
                                        for val in required.iter_mut() {
                                            if val.as_str() == Some("working_directory") {
                                                *val = json!("cwd");
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        "web_fetch" => {
                            function["name"] = json!("FetchUrl");
                        }
                        "edit_file" => {
                            function["name"] = json!("Edit");
                            if let Some(parameters) = function.get_mut("parameters") {
                                if let Some(properties) = parameters
                                    .get_mut("properties")
                                    .and_then(Value::as_object_mut)
                                {
                                    if let Some(path) = properties.remove("path") {
                                        properties.insert("file_path".into(), path);
                                    }
                                    if let Some(Value::Array(required)) =
                                        parameters.get_mut("required")
                                    {
                                        for val in required.iter_mut() {
                                            if val.as_str() == Some("path") {
                                                *val = json!("file_path");
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}

pub fn map_windsurf_tool_call_to_client(call: &mut ToolCallPlan, profile: WindsurfClientProfile) {
    if profile == WindsurfClientProfile::Droid {
        if call.name == "Execute" {
            if let Some(obj) = call.arguments.as_object_mut() {
                obj.entry("riskLevel").or_insert_with(|| json!("medium"));
                obj.entry("riskLevelReason")
                    .or_insert_with(|| json!("automated proxy invocation"));
            }
        }
    } else if profile == WindsurfClientProfile::Devin {
        match call.name.as_str() {
            "Read" => {
                call.name = "read".to_string();
                if let Some(obj) = call.arguments.as_object_mut() {
                    if let Some(file_path) = obj.remove("file_path") {
                        obj.insert("path".into(), file_path);
                    }
                }
            }
            "LS" => {
                call.name = "ls".to_string();
                if let Some(obj) = call.arguments.as_object_mut() {
                    if let Some(directory_path) = obj.remove("directory_path") {
                        obj.insert("path".into(), directory_path);
                    }
                }
            }
            "Grep" => {
                call.name = "grep".to_string();
                if let Some(obj) = call.arguments.as_object_mut() {
                    if let Some(glob_pattern) = obj.remove("glob_pattern") {
                        obj.insert("path".into(), glob_pattern);
                    }
                }
            }
            "Execute" => {
                call.name = "execute".to_string();
                if let Some(obj) = call.arguments.as_object_mut() {
                    if let Some(cwd) = obj.remove("cwd") {
                        obj.insert("working_directory".into(), cwd);
                    }
                }
            }
            "FetchUrl" => {
                call.name = "fetch".to_string();
            }
            "Edit" => {
                call.name = "edit".to_string();
                if let Some(obj) = call.arguments.as_object_mut() {
                    if let Some(file_path) = obj.remove("file_path") {
                        obj.insert("path".into(), file_path);
                    }
                }
            }
            _ => {}
        }
    } else if profile == WindsurfClientProfile::ClaudeCode {
        match call.name.as_str() {
            "Execute" => {
                call.name = "Bash".to_string();
            }
            "Grep" => {
                if let Some(obj) = call.arguments.as_object_mut() {
                    if let Some(glob_pattern) = obj.remove("glob_pattern") {
                        obj.insert("path".into(), glob_pattern);
                    }
                }
            }
            "FetchUrl" => {
                call.name = "WebFetch".to_string();
            }
            _ => {}
        }
    } else if profile == WindsurfClientProfile::CodexCli {
        match call.name.as_str() {
            "Read" => {
                call.name = "read_file".to_string();
                if let Some(obj) = call.arguments.as_object_mut() {
                    if let Some(file_path) = obj.remove("file_path") {
                        obj.insert("path".into(), file_path);
                    }
                }
            }
            "LS" => {
                call.name = "ls".to_string();
                if let Some(obj) = call.arguments.as_object_mut() {
                    if let Some(directory_path) = obj.remove("directory_path") {
                        obj.insert("path".into(), directory_path);
                    }
                }
            }
            "Grep" => {
                call.name = "grep".to_string();
                if let Some(obj) = call.arguments.as_object_mut() {
                    if let Some(glob_pattern) = obj.remove("glob_pattern") {
                        obj.insert("path".into(), glob_pattern);
                    }
                }
            }
            "Execute" => {
                call.name = "shell".to_string();
                if let Some(obj) = call.arguments.as_object_mut() {
                    if let Some(cwd) = obj.remove("cwd") {
                        obj.insert("working_directory".into(), cwd);
                    }
                }
            }
            "FetchUrl" => {
                call.name = "web_fetch".to_string();
            }
            "Edit" => {
                call.name = "edit_file".to_string();
                if let Some(obj) = call.arguments.as_object_mut() {
                    if let Some(file_path) = obj.remove("file_path") {
                        obj.insert("path".into(), file_path);
                    }
                }
            }
            _ => {}
        }
    }
}

fn map_client_tool_call_to_windsurf(func_val: &mut Value, profile: WindsurfClientProfile) {
    if profile == WindsurfClientProfile::Devin {
        if let Some(obj) = func_val.as_object_mut() {
            if let Some(name) = obj.get("name").and_then(Value::as_str).map(String::from) {
                let mapped_name = match name.as_str() {
                    "read" => "Read",
                    "ls" => "LS",
                    "search" | "grep" => "Grep",
                    "execute" | "shell" => "Execute",
                    "fetch" => "FetchUrl",
                    "edit" => "Edit",
                    other => other,
                };
                if mapped_name != name {
                    obj.insert("name".into(), json!(mapped_name));
                    if let Some(args_val) = obj.get_mut("arguments") {
                        let parsed_args = match &*args_val {
                            Value::String(s) => serde_json::from_str::<Value>(s).ok(),
                            other => Some(other.clone()),
                        };
                        if let Some(mut args_obj) =
                            parsed_args.and_then(|v| v.is_object().then_some(v))
                        {
                            let obj_mut = args_obj.as_object_mut().unwrap();
                            match name.as_str() {
                                "read" | "edit" => {
                                    if let Some(path) = obj_mut.remove("path") {
                                        obj_mut.insert("file_path".into(), path);
                                    }
                                }
                                "ls" => {
                                    if let Some(path) = obj_mut.remove("path") {
                                        obj_mut.insert("directory_path".into(), path);
                                    }
                                }
                                "search" | "grep" => {
                                    if let Some(path) = obj_mut.remove("path") {
                                        obj_mut.insert("glob_pattern".into(), path);
                                    }
                                }
                                "execute" | "shell" => {
                                    if let Some(working_directory) =
                                        obj_mut.remove("working_directory")
                                    {
                                        obj_mut.insert("cwd".into(), working_directory);
                                    }
                                }
                                _ => {}
                            }
                            if let Value::String(_) = args_val {
                                *args_val = json!(args_obj.to_string());
                            } else {
                                *args_val = args_obj;
                            }
                        }
                    }
                }
            }
        }
    } else if profile == WindsurfClientProfile::ClaudeCode {
        if let Some(obj) = func_val.as_object_mut() {
            if let Some(name) = obj.get("name").and_then(Value::as_str).map(String::from) {
                let mapped_name = match name.as_str() {
                    "Bash" => "Execute",
                    "WebFetch" => "FetchUrl",
                    other => other,
                };
                if mapped_name != name {
                    obj.insert("name".into(), json!(mapped_name));
                }
                if name == "Grep" {
                    if let Some(args_val) = obj.get_mut("arguments") {
                        let parsed_args = match &*args_val {
                            Value::String(s) => serde_json::from_str::<Value>(s).ok(),
                            other => Some(other.clone()),
                        };
                        if let Some(mut args_obj) =
                            parsed_args.and_then(|v| v.is_object().then_some(v))
                        {
                            let obj_mut = args_obj.as_object_mut().unwrap();
                            if let Some(path) = obj_mut.remove("path") {
                                obj_mut.insert("glob_pattern".into(), path);
                            }
                            if let Value::String(_) = args_val {
                                *args_val = json!(args_obj.to_string());
                            } else {
                                *args_val = args_obj;
                            }
                        }
                    }
                }
            }
        }
    } else if profile == WindsurfClientProfile::CodexCli {
        if let Some(obj) = func_val.as_object_mut() {
            if let Some(name) = obj.get("name").and_then(Value::as_str).map(String::from) {
                let mapped_name = match name.as_str() {
                    "read_file" => "Read",
                    "ls" => "LS",
                    "grep" => "Grep",
                    "shell" => "Execute",
                    "web_fetch" => "FetchUrl",
                    "edit_file" => "Edit",
                    other => other,
                };
                if mapped_name != name {
                    obj.insert("name".into(), json!(mapped_name));
                    if let Some(args_val) = obj.get_mut("arguments") {
                        let parsed_args = match &*args_val {
                            Value::String(s) => serde_json::from_str::<Value>(s).ok(),
                            other => Some(other.clone()),
                        };
                        if let Some(mut args_obj) =
                            parsed_args.and_then(|v| v.is_object().then_some(v))
                        {
                            let obj_mut = args_obj.as_object_mut().unwrap();
                            match name.as_str() {
                                "read_file" | "edit_file" => {
                                    if let Some(path) = obj_mut.remove("path") {
                                        obj_mut.insert("file_path".into(), path);
                                    }
                                }
                                "ls" => {
                                    if let Some(path) = obj_mut.remove("path") {
                                        obj_mut.insert("directory_path".into(), path);
                                    }
                                }
                                "grep" => {
                                    if let Some(path) = obj_mut.remove("path") {
                                        obj_mut.insert("glob_pattern".into(), path);
                                    }
                                }
                                "shell" => {
                                    if let Some(working_directory) =
                                        obj_mut.remove("working_directory")
                                    {
                                        obj_mut.insert("cwd".into(), working_directory);
                                    }
                                }
                                _ => {}
                            }
                            if let Value::String(_) = args_val {
                                *args_val = json!(args_obj.to_string());
                            } else {
                                *args_val = args_obj;
                            }
                        }
                    }
                }
            }
        }
    }
}

fn tool_call_from_value(value: &Value) -> Option<ToolCallPlan> {
    let object = value.as_object()?;
    if let Some(name) = object.get("name").and_then(Value::as_str) {
        return Some(ToolCallPlan {
            name: name.to_string(),
            arguments: parse_tool_arguments(object.get("arguments").cloned().unwrap_or(json!({}))),
        });
    }

    let function = object.get("function")?.as_object()?;
    let name = function.get("name").and_then(Value::as_str)?;
    Some(ToolCallPlan {
        name: name.to_string(),
        arguments: parse_tool_arguments(function.get("arguments").cloned().unwrap_or(json!({}))),
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
    fn tool_plan_parser_accepts_droid_style_tool_call_tags() {
        assert_eq!(
            parse_tool_plan(r#"<tool_call>Read{"file_path":"/tmp/README.md"}"#),
            Some(ToolPlan::ToolCalls(vec![ToolCallPlan {
                name: "Read".into(),
                arguments: json!({ "file_path": "/tmp/README.md" })
            }]))
        );
        assert_eq!(
            parse_tool_plan(
                r#"<tool_call>Read{"file_path":"/tmp/README.md"}
                  <tool_call>LS{"directory_path":"/tmp"}"#
            ),
            Some(ToolPlan::ToolCalls(vec![
                ToolCallPlan {
                    name: "Read".into(),
                    arguments: json!({ "file_path": "/tmp/README.md" })
                },
                ToolCallPlan {
                    name: "LS".into(),
                    arguments: json!({ "directory_path": "/tmp" })
                }
            ]))
        );
    }

    #[test]
    fn tool_plan_parser_rejects_unsafe_droid_style_tags() {
        assert_eq!(
            parse_tool_plan(r#"I'll read it. <tool_call>Read{"file_path":"/tmp/README.md"}"#),
            None
        );
        assert_eq!(
            parse_tool_plan(r#"<tool_call>Read{"file_path":"/tmp/README.md"} done"#),
            None
        );
        assert_eq!(
            parse_tool_plan(r#"<tool_call>Read{"file_path":"/tmp""#),
            None
        );
        assert_eq!(parse_tool_plan(r#"<tool_call>Read"/tmp/README.md""#), None);
        assert_eq!(
            parse_tool_plan(r#"<tool_call>Read["/tmp/README.md"]"#),
            None
        );
    }

    #[test]
    fn tool_plan_parser_accepts_exact_assistant_tool_calls_block() {
        assert_eq!(
            parse_tool_plan(
                r#"ASSISTANT TOOL_CALLS: [{"id":"call_1","type":"function","function":{"name":"Execute","arguments":"{\"command\":\"git status --short\"}"}}]"#
            ),
            Some(ToolPlan::ToolCalls(vec![ToolCallPlan {
                name: "Execute".into(),
                arguments: json!({ "command": "git status --short" })
            }]))
        );
    }

    #[test]
    fn tool_plan_parser_rejects_mixed_assistant_tool_call_transcripts() {
        assert_eq!(
            parse_tool_plan(
                r#"ASSISTANT TOOL_CALLS: [{"id":"call_1","type":"function","function":{"name":"Execute","arguments":"{\"command\":\"git status\"}"}}]

TOOL RESULT call_1: ok"#
            ),
            None
        );
        assert_eq!(
            parse_tool_plan(
                r#"I will run it.
ASSISTANT TOOL_CALLS: [{"id":"call_1","type":"function","function":{"name":"Execute","arguments":"{\"command\":\"git status\"}"}}]"#
            ),
            None
        );
    }

    #[test]
    fn tool_prompt_preserves_tool_result_loop_context() {
        use super::WindsurfClientProfile;
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
        }), WindsurfClientProfile::Other)
        .unwrap();

        assert!(prompt.contains("- lookup: search"));
        assert!(prompt.contains("ASSISTANT TOOL_CALLS"));
        assert!(prompt.contains("TOOL RESULT call_1: result"));
        assert!(prompt.contains("After a TOOL RESULT"));
    }

    #[test]
    fn test_map_client_tools_to_windsurf_devin() {
        use super::WindsurfClientProfile;

        let mut tools = vec![
            json!({
                "type": "function",
                "function": {
                    "name": "read",
                    "parameters": {
                        "type": "object",
                        "properties": {
                            "path": { "type": "string" }
                        },
                        "required": ["path"]
                    }
                }
            }),
            json!({
                "type": "function",
                "function": {
                    "name": "execute",
                    "parameters": {
                        "type": "object",
                        "properties": {
                            "command": { "type": "string" },
                            "working_directory": { "type": "string" }
                        },
                        "required": ["command", "working_directory"]
                    }
                }
            }),
        ];

        map_client_tools_to_windsurf(&mut tools, WindsurfClientProfile::Devin);

        assert_eq!(tools[0]["function"]["name"], "Read");
        assert!(tools[0]["function"]["parameters"]["properties"]
            .get("file_path")
            .is_some());
        assert!(tools[0]["function"]["parameters"]["properties"]
            .get("path")
            .is_none());
        assert_eq!(
            tools[0]["function"]["parameters"]["required"][0],
            "file_path"
        );

        assert_eq!(tools[1]["function"]["name"], "Execute");
        assert!(tools[1]["function"]["parameters"]["properties"]
            .get("cwd")
            .is_some());
        assert!(tools[1]["function"]["parameters"]["properties"]
            .get("working_directory")
            .is_none());
    }

    #[test]
    fn test_map_windsurf_tool_call_to_client_devin() {
        use super::WindsurfClientProfile;

        let mut call = ToolCallPlan {
            name: "Read".to_string(),
            arguments: json!({ "file_path": "/tmp/README.md" }),
        };

        map_windsurf_tool_call_to_client(&mut call, WindsurfClientProfile::Devin);

        assert_eq!(call.name, "read");
        assert_eq!(call.arguments["path"], "/tmp/README.md");
        assert!(call.arguments.get("file_path").is_none());
    }

    #[test]
    fn test_map_client_tools_to_windsurf_claude_and_codex() {
        use super::WindsurfClientProfile;

        // Claude Code tools mapping
        let mut claude_tools = vec![json!({
            "type": "function",
            "function": {
                "name": "Bash",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "command": { "type": "string" }
                    }
                }
            }
        })];
        map_client_tools_to_windsurf(&mut claude_tools, WindsurfClientProfile::ClaudeCode);
        assert_eq!(claude_tools[0]["function"]["name"], "Execute");

        // Codex CLI tools mapping
        let mut codex_tools = vec![json!({
            "type": "function",
            "function": {
                "name": "read_file",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "path": { "type": "string" }
                    },
                    "required": ["path"]
                }
            }
        })];
        map_client_tools_to_windsurf(&mut codex_tools, WindsurfClientProfile::CodexCli);
        assert_eq!(codex_tools[0]["function"]["name"], "Read");
        assert!(codex_tools[0]["function"]["parameters"]["properties"]
            .get("file_path")
            .is_some());
        assert_eq!(
            codex_tools[0]["function"]["parameters"]["required"][0],
            "file_path"
        );
    }

    #[test]
    fn test_map_windsurf_tool_call_to_client_claude_and_codex() {
        use super::WindsurfClientProfile;

        // Claude
        let mut claude_call = ToolCallPlan {
            name: "Execute".to_string(),
            arguments: json!({ "command": "ls" }),
        };
        map_windsurf_tool_call_to_client(&mut claude_call, WindsurfClientProfile::ClaudeCode);
        assert_eq!(claude_call.name, "Bash");

        // Codex
        let mut codex_call = ToolCallPlan {
            name: "Read".to_string(),
            arguments: json!({ "file_path": "/tmp/README.md" }),
        };
        map_windsurf_tool_call_to_client(&mut codex_call, WindsurfClientProfile::CodexCli);
        assert_eq!(codex_call.name, "read_file");
        assert_eq!(codex_call.arguments["path"], "/tmp/README.md");
    }

    #[test]
    fn test_map_windsurf_tool_call_to_client_droid_execute_adds_risk_defaults() {
        let mut call = ToolCallPlan {
            name: "Execute".to_string(),
            arguments: json!({ "command": "git status --short" }),
        };

        map_windsurf_tool_call_to_client(&mut call, WindsurfClientProfile::Droid);

        assert_eq!(call.name, "Execute");
        assert_eq!(call.arguments["command"], "git status --short");
        assert_eq!(call.arguments["riskLevel"], "medium");
        assert_eq!(
            call.arguments["riskLevelReason"],
            "automated proxy invocation"
        );
    }
}
