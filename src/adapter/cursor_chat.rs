//! `/v1/chat/completions` <-> Cursor agent translation.
//!
//! Converts public OpenAI Chat Completions JSON to/from
//! `crate::cursor_agent::*` DTOs only. Like `cursor_responses.rs`, this
//! module is pure; the route layer wires upstream I/O around `build_request`
//! and `emit_event` / `collect_non_stream`.

use std::collections::HashMap;

use serde_json::{json, Map, Value};
use uuid::Uuid;

use crate::adapter::cursor_events::chat_finish_reason;
use crate::cursor_agent::{
    CursorAgentEvent, CursorAgentRequest, CursorContentBlock, CursorFinishReason, CursorMessage,
    CursorTool, CursorToolCall, CursorToolKind, CursorToolResult,
};
use crate::{AppError, AppResult};

const MAX_TOOL_NAME_LEN: usize = 64;

/// Per-request Chat Completions translation context.
#[derive(Debug, Clone)]
pub struct ChatContext {
    pub model: String,
    pub completion_id: String,
    pub system_fingerprint: Option<String>,
    pub started: bool,
    pub completed: bool,
    pub failed: bool,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub reasoning_tokens: u64,
    pub finish_reason: Option<CursorFinishReason>,
    text_buffer: String,
    reasoning_buffer: String,
    /// Open tool calls keyed by Cursor exec id.
    tool_calls: HashMap<String, ChatToolCallState>,
    /// Insertion order for the `tool_calls[]` array on the assistant message.
    tool_call_order: Vec<String>,
}

#[derive(Debug, Clone)]
struct ChatToolCallState {
    /// `tool_calls[i].index` on the streaming wire.
    index: u32,
    name: String,
    arguments: String,
    /// `tool_calls[i].id` on the response.
    public_id: String,
}

impl ChatContext {
    pub fn new(model: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            completion_id: format!("chatcmpl-{}", Uuid::new_v4().simple()),
            system_fingerprint: None,
            started: false,
            completed: false,
            failed: false,
            input_tokens: 0,
            output_tokens: 0,
            reasoning_tokens: 0,
            finish_reason: None,
            text_buffer: String::new(),
            reasoning_buffer: String::new(),
            tool_calls: HashMap::new(),
            tool_call_order: Vec::new(),
        }
    }
}

/// Build a `CursorAgentRequest` from a public Chat Completions request body.
pub fn build_request(public_json: &Value) -> AppResult<CursorAgentRequest> {
    let object = public_json
        .as_object()
        .ok_or_else(|| AppError::BadRequest("request body must be a JSON object".into()))?;

    let model = required_string(object, "model")?.to_string();
    let upstream_model = model.clone();
    let stream = object
        .get("stream")
        .and_then(Value::as_bool)
        .unwrap_or(false);

    let messages = object
        .get("messages")
        .and_then(Value::as_array)
        .ok_or_else(|| AppError::BadRequest("messages must be an array".into()))?;
    let (system_instructions, developer_instructions, normalized_messages, tool_results) =
        convert_chat_messages(messages)?;

    let mut tools = Vec::new();
    if let Some(raw_tools) = object.get("tools") {
        tools = parse_tools(raw_tools)?;
    }
    if let Some(tool_choice) = object.get("tool_choice") {
        validate_tool_choice(tool_choice)?;
    }

    Ok(CursorAgentRequest {
        model,
        upstream_model,
        system_instructions,
        developer_instructions,
        messages: normalized_messages,
        tools,
        tool_results,
        continuation_key: None,
        workspace: None,
        stream,
        request_id: Uuid::new_v4(),
    })
}

/// Translate a `CursorAgentEvent` into one or more Chat Completions
/// streaming chunks (`chat.completion.chunk` objects).
pub fn emit_event(event: &CursorAgentEvent, ctx: &mut ChatContext) -> Vec<Value> {
    let mut out = Vec::new();
    if !ctx.started {
        out.push(initial_role_chunk(ctx));
        ctx.started = true;
    }

    match event {
        CursorAgentEvent::TextDelta { delta, .. } => {
            ctx.text_buffer.push_str(delta);
            out.push(content_delta_chunk(ctx, delta));
        }
        CursorAgentEvent::ReasoningDelta { delta } => {
            ctx.reasoning_buffer.push_str(delta);
            out.push(reasoning_delta_chunk(ctx, delta));
        }
        CursorAgentEvent::ToolCallStarted { call_id, name, .. } => {
            let index = ctx.tool_call_order.len() as u32;
            ctx.tool_call_order.push(call_id.clone());
            let public_id = format!("call_{}", call_id);
            ctx.tool_calls.insert(
                call_id.clone(),
                ChatToolCallState {
                    index,
                    name: name.clone(),
                    arguments: String::new(),
                    public_id: public_id.clone(),
                },
            );
            out.push(tool_call_open_chunk(ctx, index, &public_id, name));
        }
        CursorAgentEvent::ToolCallArgumentsDelta { call_id, delta } => {
            if !ctx.tool_calls.contains_key(call_id) {
                let index = ctx.tool_call_order.len() as u32;
                ctx.tool_call_order.push(call_id.clone());
                let public_id = format!("call_{}", call_id);
                ctx.tool_calls.insert(
                    call_id.clone(),
                    ChatToolCallState {
                        index,
                        name: call_id.clone(),
                        arguments: String::new(),
                        public_id: public_id.clone(),
                    },
                );
                out.push(tool_call_open_chunk(ctx, index, &public_id, call_id));
            }
            if let Some(state) = ctx.tool_calls.get_mut(call_id) {
                state.arguments.push_str(delta);
                out.push(tool_call_arguments_chunk(ctx, call_id, delta));
            }
        }
        CursorAgentEvent::ToolCallDone { call_id, arguments } => {
            // Final flush: replace prior fragments with the canonical string
            // so downstream consumers always see a parseable JSON body.
            let final_args = match arguments {
                Value::String(text) => text.clone(),
                other => other.to_string(),
            };
            if let Some(state) = ctx.tool_calls.get_mut(call_id) {
                state.arguments = final_args;
            }
        }
        CursorAgentEvent::UsageUpdate {
            input_tokens,
            output_tokens,
            reasoning_tokens,
        } => {
            ctx.input_tokens = *input_tokens;
            ctx.output_tokens = *output_tokens;
            if let Some(reasoning) = reasoning_tokens {
                ctx.reasoning_tokens = *reasoning;
            }
        }
        CursorAgentEvent::Checkpoint { .. } => {}
        CursorAgentEvent::ProviderError { code, message, .. } => {
            ctx.failed = true;
            out.push(error_chunk(ctx, code, message));
        }
        CursorAgentEvent::Done { finish_reason, .. } => {
            ctx.finish_reason = Some(*finish_reason);
            out.push(finish_chunk(ctx, *finish_reason));
            ctx.completed = true;
        }
    }

    out
}

/// Collect a complete `chat.completion` object from a finished event stream.
pub fn collect_non_stream(model: &str, events: Vec<CursorAgentEvent>) -> AppResult<Value> {
    let mut ctx = ChatContext::new(model);
    for event in events {
        emit_event(&event, &mut ctx);
    }

    let finish = ctx.finish_reason.map(chat_finish_reason).unwrap_or("stop");

    let mut tool_calls = Vec::new();
    for call_id in &ctx.tool_call_order {
        if let Some(state) = ctx.tool_calls.get(call_id) {
            tool_calls.push(json!({
                "id": state.public_id.clone(),
                "type": "function",
                "function": {
                    "name": state.name.clone(),
                    "arguments": state.arguments.clone(),
                },
            }));
        }
    }

    let mut message = json!({
        "role": "assistant",
        "content": if ctx.text_buffer.is_empty() {
            Value::Null
        } else {
            Value::String(ctx.text_buffer.clone())
        },
    });
    if !ctx.reasoning_buffer.is_empty() {
        if let Some(object) = message.as_object_mut() {
            object.insert(
                "reasoning_content".into(),
                Value::String(ctx.reasoning_buffer.clone()),
            );
        }
    }
    if !tool_calls.is_empty() {
        if let Some(object) = message.as_object_mut() {
            object.insert("tool_calls".into(), Value::Array(tool_calls));
        }
    }

    Ok(json!({
        "id": ctx.completion_id,
        "object": "chat.completion",
        "created": 0,
        "model": ctx.model.clone(),
        "choices": [{
            "index": 0,
            "message": message,
            "finish_reason": finish,
        }],
        "usage": {
            "prompt_tokens": ctx.input_tokens,
            "completion_tokens": ctx.output_tokens,
            "total_tokens": ctx.input_tokens.saturating_add(ctx.output_tokens),
            "completion_tokens_details": { "reasoning_tokens": ctx.reasoning_tokens },
        },
    }))
}

// ---------------------------------------------------------------------------
// Chunk constructors
// ---------------------------------------------------------------------------

fn initial_role_chunk(ctx: &ChatContext) -> Value {
    json!({
        "id": ctx.completion_id.clone(),
        "object": "chat.completion.chunk",
        "created": 0,
        "model": ctx.model.clone(),
        "choices": [{
            "index": 0,
            "delta": { "role": "assistant", "content": "" },
            "finish_reason": Value::Null,
        }],
    })
}

fn content_delta_chunk(ctx: &ChatContext, delta: &str) -> Value {
    json!({
        "id": ctx.completion_id.clone(),
        "object": "chat.completion.chunk",
        "created": 0,
        "model": ctx.model.clone(),
        "choices": [{
            "index": 0,
            "delta": { "content": delta },
            "finish_reason": Value::Null,
        }],
    })
}

fn reasoning_delta_chunk(ctx: &ChatContext, delta: &str) -> Value {
    json!({
        "id": ctx.completion_id.clone(),
        "object": "chat.completion.chunk",
        "created": 0,
        "model": ctx.model.clone(),
        "choices": [{
            "index": 0,
            "delta": { "reasoning_content": delta },
            "finish_reason": Value::Null,
        }],
    })
}

fn tool_call_open_chunk(ctx: &ChatContext, index: u32, public_id: &str, name: &str) -> Value {
    json!({
        "id": ctx.completion_id.clone(),
        "object": "chat.completion.chunk",
        "created": 0,
        "model": ctx.model.clone(),
        "choices": [{
            "index": 0,
            "delta": {
                "tool_calls": [{
                    "index": index,
                    "id": public_id,
                    "type": "function",
                    "function": { "name": name, "arguments": "" },
                }],
            },
            "finish_reason": Value::Null,
        }],
    })
}

fn tool_call_arguments_chunk(ctx: &ChatContext, call_id: &str, delta: &str) -> Value {
    let state = ctx
        .tool_calls
        .get(call_id)
        .expect("tool call state present");
    json!({
        "id": ctx.completion_id.clone(),
        "object": "chat.completion.chunk",
        "created": 0,
        "model": ctx.model.clone(),
        "choices": [{
            "index": 0,
            "delta": {
                "tool_calls": [{
                    "index": state.index,
                    "function": { "arguments": delta },
                }],
            },
            "finish_reason": Value::Null,
        }],
    })
}

fn finish_chunk(ctx: &ChatContext, finish: CursorFinishReason) -> Value {
    json!({
        "id": ctx.completion_id.clone(),
        "object": "chat.completion.chunk",
        "created": 0,
        "model": ctx.model.clone(),
        "choices": [{
            "index": 0,
            "delta": {},
            "finish_reason": chat_finish_reason(finish),
        }],
        "usage": {
            "prompt_tokens": ctx.input_tokens,
            "completion_tokens": ctx.output_tokens,
            "total_tokens": ctx.input_tokens.saturating_add(ctx.output_tokens),
            "completion_tokens_details": { "reasoning_tokens": ctx.reasoning_tokens },
        },
    })
}

fn error_chunk(ctx: &ChatContext, code: &str, message: &str) -> Value {
    json!({
        "id": ctx.completion_id.clone(),
        "object": "chat.completion.chunk",
        "created": 0,
        "model": ctx.model.clone(),
        "error": { "type": code, "message": message },
        "choices": [],
    })
}

// ---------------------------------------------------------------------------
// Request parsing
// ---------------------------------------------------------------------------

#[allow(clippy::type_complexity)]
fn convert_chat_messages(
    messages: &[Value],
) -> AppResult<(
    Option<String>,
    Option<String>,
    Vec<CursorMessage>,
    Vec<CursorToolResult>,
)> {
    let mut system_parts: Vec<String> = Vec::new();
    let mut developer_parts: Vec<String> = Vec::new();
    let mut normalized: Vec<CursorMessage> = Vec::new();
    let mut tool_results: Vec<CursorToolResult> = Vec::new();

    let has_tool_results = messages.iter().any(|message| {
        message
            .as_object()
            .and_then(|object| object.get("role"))
            .and_then(Value::as_str)
            == Some("tool")
    });

    for message in messages {
        let object = message
            .as_object()
            .ok_or_else(|| AppError::BadRequest("message must be an object".into()))?;
        let role = required_string(object, "role")?;
        match role {
            "system" => {
                if let Some(text) = chat_content_text(object.get("content"))? {
                    system_parts.push(text);
                }
            }
            "developer" => {
                if let Some(text) = chat_content_text(object.get("content"))? {
                    developer_parts.push(text);
                }
            }
            "user" => {
                let blocks = chat_user_content_blocks(object.get("content"))?;
                normalized.push(CursorMessage::User { blocks });
            }
            "assistant" => {
                let blocks = match chat_content_text(object.get("content"))? {
                    Some(text) if !text.is_empty() => vec![CursorContentBlock::Text(text)],
                    _ => Vec::new(),
                };
                let mut tool_calls = Vec::new();
                // When the request carries tool results, prior assistant
                // `tool_calls` are echoes of pending exec calls. The run
                // engine already knows about them via the parked session
                // store, so we drop them rather than feeding duplicate
                // `tool_use` blocks back to Cursor.
                if !has_tool_results {
                    if let Some(raw_calls) = object.get("tool_calls").and_then(Value::as_array) {
                        for call in raw_calls {
                            let function = call
                                .get("function")
                                .and_then(Value::as_object)
                                .ok_or_else(|| {
                                    AppError::BadRequest("tool_call.function is required".into())
                                })?;
                            let id = call
                                .get("id")
                                .and_then(Value::as_str)
                                .ok_or_else(|| {
                                    AppError::BadRequest("tool_call.id is required".into())
                                })?
                                .to_string();
                            let name = required_string(function, "name")?.to_string();
                            let arguments_string = function
                                .get("arguments")
                                .and_then(Value::as_str)
                                .unwrap_or("{}")
                                .to_string();
                            let arguments_value: Value = serde_json::from_str(&arguments_string)
                                .unwrap_or(Value::String(arguments_string));
                            tool_calls.push(CursorToolCall {
                                id,
                                name,
                                arguments: arguments_value,
                            });
                        }
                    }
                }
                if !blocks.is_empty() || !tool_calls.is_empty() {
                    normalized.push(CursorMessage::Assistant { blocks, tool_calls });
                }
            }
            "tool" => {
                let call_id = object
                    .get("tool_call_id")
                    .and_then(Value::as_str)
                    .ok_or_else(|| AppError::BadRequest("tool_call_id is required".into()))?
                    .to_string();
                let (output, error) = chat_tool_content(object.get("content"))?;
                tool_results.push(CursorToolResult {
                    call_id,
                    output,
                    error,
                });
            }
            other => {
                return Err(AppError::BadRequest(format!(
                    "unsupported message role: {other}"
                )))
            }
        }
    }

    Ok((
        nonempty_join(system_parts, "\n\n"),
        nonempty_join(developer_parts, "\n\n"),
        normalized,
        tool_results,
    ))
}

/// Extract a chat `role: "tool"` message's content, parsing it as JSON when
/// possible (so structured tool outputs survive round-tripping into Cursor)
/// and falling back to a plain string when it is not valid JSON. Detects an
/// `error` key on parsed JSON outputs and surfaces it on
/// `CursorToolResult::error` while leaving the original output intact for the
/// downstream exec result frame.
fn chat_tool_content(content: Option<&Value>) -> AppResult<(Value, Option<String>)> {
    let raw = chat_content_text(content)?.unwrap_or_default();
    let parsed: Value = serde_json::from_str(&raw)
        .ok()
        .filter(|value| !matches!(value, Value::String(_)))
        .unwrap_or(Value::String(raw));
    let error = match &parsed {
        Value::Object(map) => map.get("error").and_then(|value| {
            if value.is_null() {
                None
            } else {
                Some(match value {
                    Value::String(text) => text.clone(),
                    other => other.to_string(),
                })
            }
        }),
        _ => None,
    };
    Ok((parsed, error))
}

fn chat_content_text(content: Option<&Value>) -> AppResult<Option<String>> {
    match content {
        Some(Value::String(text)) => Ok(Some(text.clone())),
        Some(Value::Array(blocks)) => {
            let mut texts = Vec::new();
            for block in blocks {
                let block_type = block.get("type").and_then(Value::as_str).unwrap_or("text");
                if block_type != "text" {
                    return Err(AppError::BadRequest(
                        "only text content is supported here".into(),
                    ));
                }
                if let Some(text) = block.get("text").and_then(Value::as_str) {
                    texts.push(text.to_string());
                }
            }
            Ok(nonempty_join(texts, ""))
        }
        Some(Value::Null) | None => Ok(None),
        Some(_) => Err(AppError::BadRequest(
            "message content must be a string or array".into(),
        )),
    }
}

fn chat_user_content_blocks(content: Option<&Value>) -> AppResult<Vec<CursorContentBlock>> {
    match content {
        Some(Value::String(text)) => Ok(vec![CursorContentBlock::Text(text.clone())]),
        Some(Value::Array(blocks)) => {
            let mut out = Vec::new();
            for block in blocks {
                let block_type = block.get("type").and_then(Value::as_str).unwrap_or("text");
                match block_type {
                    "text" | "input_text" => {
                        if let Some(text) = block.get("text").and_then(Value::as_str) {
                            out.push(CursorContentBlock::Text(text.to_string()));
                        }
                    }
                    "image_url" | "input_image" => {
                        return Err(AppError::BadRequest(
                            "image content is not supported for Cursor Composer".into(),
                        ));
                    }
                    other => {
                        return Err(AppError::BadRequest(format!(
                            "unsupported chat content block: {other}"
                        )))
                    }
                }
            }
            Ok(out)
        }
        Some(Value::Null) | None => Ok(Vec::new()),
        Some(_) => Err(AppError::BadRequest(
            "message content must be a string or array".into(),
        )),
    }
}

fn parse_tools(value: &Value) -> AppResult<Vec<CursorTool>> {
    let array = value
        .as_array()
        .ok_or_else(|| AppError::BadRequest("tools must be an array".into()))?;
    let mut out = Vec::new();
    for tool in array {
        let object = tool
            .as_object()
            .ok_or_else(|| AppError::BadRequest("tool must be an object".into()))?;
        let tool_type = object
            .get("type")
            .and_then(Value::as_str)
            .unwrap_or("function");
        if tool_type != "function" {
            return Err(AppError::BadRequest(format!(
                "tool type {tool_type} is not supported for chat completions"
            )));
        }
        let function = object
            .get("function")
            .and_then(Value::as_object)
            .ok_or_else(|| AppError::BadRequest("function tool is missing function".into()))?;
        let name = required_string(function, "name")?;
        validate_tool_name(name)?;
        let description = function
            .get("description")
            .and_then(Value::as_str)
            .map(str::to_string);
        let parameters_schema = function
            .get("parameters")
            .cloned()
            .unwrap_or_else(|| json!({ "type": "object", "properties": {} }));
        out.push(CursorTool {
            name: name.to_string(),
            description,
            parameters_schema,
            kind: CursorToolKind::Function,
        });
    }
    Ok(out)
}

fn validate_tool_choice(value: &Value) -> AppResult<()> {
    match value {
        Value::String(choice) if matches!(choice.as_str(), "auto" | "none" | "required") => Ok(()),
        Value::Object(object) => match object.get("type").and_then(Value::as_str) {
            Some("function") => {
                let function = object
                    .get("function")
                    .and_then(Value::as_object)
                    .ok_or_else(|| {
                        AppError::BadRequest("tool_choice.function is required".into())
                    })?;
                required_string(function, "name")?;
                Ok(())
            }
            Some(other) => Err(AppError::BadRequest(format!(
                "tool_choice type {other} is not supported"
            ))),
            None => Err(AppError::BadRequest("tool_choice.type is required".into())),
        },
        _ => Err(AppError::BadRequest("tool_choice is not supported".into())),
    }
}

fn required_string<'a>(object: &'a Map<String, Value>, key: &str) -> AppResult<&'a str> {
    object
        .get(key)
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| AppError::BadRequest(format!("{key} is required")))
}

fn validate_tool_name(name: &str) -> AppResult<()> {
    if name.is_empty() {
        return Err(AppError::BadRequest("tool name is required".into()));
    }
    if name.len() > MAX_TOOL_NAME_LEN {
        return Err(AppError::BadRequest(
            "tool name exceeds 64 characters".into(),
        ));
    }
    Ok(())
}

fn nonempty_join(values: Vec<String>, separator: &str) -> Option<String> {
    let values = values
        .into_iter()
        .filter(|value| !value.trim().is_empty())
        .collect::<Vec<_>>();
    if values.is_empty() {
        None
    } else {
        Some(values.join(separator))
    }
}
