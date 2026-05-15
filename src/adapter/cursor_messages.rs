//! `/v1/messages` (Anthropic Messages) <-> Cursor agent translation.
//!
//! Converts public Anthropic Messages JSON to/from `crate::cursor_agent::*`
//! DTOs only. The route layer wires the upstream call between
//! `build_request` and `emit_event` / `collect_non_stream`.
//!
//! Image blocks are explicitly REJECTED with a `400` per ralplan Section 7
//! step 1: image content is unsupported until a Cursor multimodal path is
//! proven. Reasoning/thinking events surface as `thinking_delta` content
//! block deltas (Composer 2-family); thinking signature support is omitted
//! until Phase 0 records whether Cursor provides a per-block signature.

use std::collections::HashMap;

use serde_json::{json, Map, Value};
use uuid::Uuid;

use crate::adapter::cursor_events::anthropic_stop_reason;
use crate::cursor_agent::{
    CursorAgentEvent, CursorAgentRequest, CursorContentBlock, CursorFinishReason, CursorMessage,
    CursorTool, CursorToolCall, CursorToolKind, CursorToolResult,
};
use crate::{AppError, AppResult};

const MAX_TOOL_NAME_LEN: usize = 64;

/// One on-the-wire SSE frame for the Anthropic Messages stream.
#[derive(Debug, Clone)]
pub struct MessagesSseEvent {
    pub event: String,
    pub data: Value,
}

impl MessagesSseEvent {
    pub fn new(event: impl Into<String>, data: Value) -> Self {
        Self {
            event: event.into(),
            data,
        }
    }

    pub fn to_wire(&self) -> String {
        let mut out = String::with_capacity(self.event.len() + 64);
        out.push_str("event: ");
        out.push_str(&self.event);
        out.push('\n');
        out.push_str("data: ");
        out.push_str(&self.data.to_string());
        out.push_str("\n\n");
        out
    }
}

/// Per-request Anthropic Messages translation state.
#[derive(Debug, Clone)]
pub struct MessagesContext {
    pub model: String,
    pub message_id: String,
    pub started: bool,
    pub completed: bool,
    pub failed: bool,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub finish_reason: Option<CursorFinishReason>,
    next_index: u32,
    text_state: Option<TextBlockState>,
    thinking_state: Option<ThinkingBlockState>,
    tool_calls: HashMap<String, ToolCallBlockState>,
    tool_call_order: Vec<String>,
    completed_blocks: Vec<Value>,
}

#[derive(Debug, Clone)]
struct TextBlockState {
    index: u32,
    text: String,
}

#[derive(Debug, Clone)]
struct ThinkingBlockState {
    index: u32,
    thinking: String,
}

#[derive(Debug, Clone)]
struct ToolCallBlockState {
    index: u32,
    id: String,
    name: String,
    arguments_buffer: String,
}

impl MessagesContext {
    pub fn new(model: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            message_id: format!("msg_{}", Uuid::new_v4().simple()),
            started: false,
            completed: false,
            failed: false,
            input_tokens: 0,
            output_tokens: 0,
            finish_reason: None,
            next_index: 0,
            text_state: None,
            thinking_state: None,
            tool_calls: HashMap::new(),
            tool_call_order: Vec::new(),
            completed_blocks: Vec::new(),
        }
    }

    fn allocate_index(&mut self) -> u32 {
        let index = self.next_index;
        self.next_index = self.next_index.saturating_add(1);
        index
    }
}

/// Build a `CursorAgentRequest` from a public Anthropic Messages request body.
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

    let system_instructions = object
        .get("system")
        .and_then(anthropic_system_text)
        .transpose()?;

    let messages_value = object
        .get("messages")
        .and_then(Value::as_array)
        .ok_or_else(|| AppError::BadRequest("messages must be an array".into()))?;

    let (normalized_messages, tool_results) = convert_anthropic_messages(messages_value)?;

    if !tool_results.is_empty() {
        let has_prior_tool_use = normalized_messages.iter().any(|message| {
            matches!(
                message,
                CursorMessage::Assistant { tool_calls, .. } if !tool_calls.is_empty()
            )
        });
        if !has_prior_tool_use {
            return Err(AppError::BadRequest(
                "tool result requires previous_response_id".into(),
            ));
        }
    }

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
        developer_instructions: None,
        messages: normalized_messages,
        tools,
        tool_results,
        continuation_key: None,
        workspace: None,
        stream,
        request_id: Uuid::new_v4(),
    })
}

/// Translate a `CursorAgentEvent` into one or more Anthropic Messages SSE
/// frames.
pub fn emit_event(event: &CursorAgentEvent, ctx: &mut MessagesContext) -> Vec<MessagesSseEvent> {
    let mut out = Vec::new();
    if !ctx.started {
        out.push(message_start(ctx));
        ctx.started = true;
    }

    match event {
        CursorAgentEvent::TextDelta { delta, .. } => emit_text_delta(ctx, delta, &mut out),
        CursorAgentEvent::ReasoningDelta { delta } => emit_thinking_delta(ctx, delta, &mut out),
        CursorAgentEvent::ToolCallStarted { call_id, name, .. } => {
            emit_tool_call_started(ctx, call_id, name, &mut out)
        }
        CursorAgentEvent::ToolCallArgumentsDelta { call_id, delta } => {
            emit_tool_call_arguments_delta(ctx, call_id, delta, &mut out)
        }
        CursorAgentEvent::ToolCallDone { call_id, arguments } => {
            emit_tool_call_done(ctx, call_id, arguments, &mut out)
        }
        CursorAgentEvent::UsageUpdate {
            input_tokens,
            output_tokens,
            ..
        } => {
            ctx.input_tokens = *input_tokens;
            ctx.output_tokens = *output_tokens;
        }
        CursorAgentEvent::Checkpoint { .. } => {}
        CursorAgentEvent::ProviderError { code, message, .. } => {
            ctx.failed = true;
            out.push(error_event(code, message));
        }
        CursorAgentEvent::Done { finish_reason, .. } => {
            close_open_text(ctx, &mut out);
            close_open_thinking(ctx, &mut out);
            close_open_tool_calls(ctx, &mut out);
            ctx.finish_reason = Some(*finish_reason);
            out.push(message_delta(ctx, *finish_reason));
            out.push(MessagesSseEvent::new(
                "message_stop",
                json!({ "type": "message_stop" }),
            ));
            ctx.completed = true;
        }
    }

    out
}

/// Collect a complete Anthropic Messages object from a finished event stream.
pub fn collect_non_stream(model: &str, events: Vec<CursorAgentEvent>) -> AppResult<Value> {
    let mut ctx = MessagesContext::new(model);
    for event in events {
        emit_event(&event, &mut ctx);
    }

    let stop_reason = ctx
        .finish_reason
        .map(anthropic_stop_reason)
        .unwrap_or("end_turn");

    Ok(json!({
        "id": ctx.message_id,
        "type": "message",
        "role": "assistant",
        "model": ctx.model.clone(),
        "content": ctx.completed_blocks,
        "stop_reason": stop_reason,
        "stop_sequence": Value::Null,
        "usage": {
            "input_tokens": ctx.input_tokens,
            "output_tokens": ctx.output_tokens,
        },
    }))
}

// ---------------------------------------------------------------------------
// SSE emission
// ---------------------------------------------------------------------------

fn message_start(ctx: &MessagesContext) -> MessagesSseEvent {
    MessagesSseEvent::new(
        "message_start",
        json!({
            "type": "message_start",
            "message": {
                "id": ctx.message_id.clone(),
                "type": "message",
                "role": "assistant",
                "model": ctx.model.clone(),
                "content": [],
                "stop_reason": Value::Null,
                "stop_sequence": Value::Null,
                "usage": { "input_tokens": 0, "output_tokens": 0 },
            },
        }),
    )
}

fn emit_text_delta(ctx: &mut MessagesContext, delta: &str, out: &mut Vec<MessagesSseEvent>) {
    if ctx.text_state.is_none() {
        let index = ctx.allocate_index();
        ctx.text_state = Some(TextBlockState {
            index,
            text: String::new(),
        });
        out.push(MessagesSseEvent::new(
            "content_block_start",
            json!({
                "type": "content_block_start",
                "index": index,
                "content_block": { "type": "text", "text": "" },
            }),
        ));
    }
    let state = ctx.text_state.as_mut().expect("text state opened");
    state.text.push_str(delta);
    out.push(MessagesSseEvent::new(
        "content_block_delta",
        json!({
            "type": "content_block_delta",
            "index": state.index,
            "delta": { "type": "text_delta", "text": delta },
        }),
    ));
}

fn close_open_text(ctx: &mut MessagesContext, out: &mut Vec<MessagesSseEvent>) {
    let Some(state) = ctx.text_state.take() else {
        return;
    };
    out.push(MessagesSseEvent::new(
        "content_block_stop",
        json!({
            "type": "content_block_stop",
            "index": state.index,
        }),
    ));
    ctx.completed_blocks
        .push(json!({ "type": "text", "text": state.text }));
}

fn emit_thinking_delta(ctx: &mut MessagesContext, delta: &str, out: &mut Vec<MessagesSseEvent>) {
    if ctx.thinking_state.is_none() {
        let index = ctx.allocate_index();
        ctx.thinking_state = Some(ThinkingBlockState {
            index,
            thinking: String::new(),
        });
        out.push(MessagesSseEvent::new(
            "content_block_start",
            json!({
                "type": "content_block_start",
                "index": index,
                "content_block": { "type": "thinking", "thinking": "" },
            }),
        ));
    }
    let state = ctx.thinking_state.as_mut().expect("thinking state opened");
    state.thinking.push_str(delta);
    out.push(MessagesSseEvent::new(
        "content_block_delta",
        json!({
            "type": "content_block_delta",
            "index": state.index,
            "delta": { "type": "thinking_delta", "thinking": delta },
        }),
    ));
}

fn close_open_thinking(ctx: &mut MessagesContext, out: &mut Vec<MessagesSseEvent>) {
    let Some(state) = ctx.thinking_state.take() else {
        return;
    };
    out.push(MessagesSseEvent::new(
        "content_block_stop",
        json!({
            "type": "content_block_stop",
            "index": state.index,
        }),
    ));
    ctx.completed_blocks.push(json!({
        "type": "thinking",
        "thinking": state.thinking,
    }));
}

fn emit_tool_call_started(
    ctx: &mut MessagesContext,
    call_id: &str,
    name: &str,
    out: &mut Vec<MessagesSseEvent>,
) {
    if ctx.tool_calls.contains_key(call_id) {
        return;
    }
    let index = ctx.allocate_index();
    ctx.tool_call_order.push(call_id.to_string());
    ctx.tool_calls.insert(
        call_id.to_string(),
        ToolCallBlockState {
            index,
            id: call_id.to_string(),
            name: name.to_string(),
            arguments_buffer: String::new(),
        },
    );
    out.push(MessagesSseEvent::new(
        "content_block_start",
        json!({
            "type": "content_block_start",
            "index": index,
            "content_block": {
                "type": "tool_use",
                "id": call_id,
                "name": name,
                "input": {},
            },
        }),
    ));
}

fn emit_tool_call_arguments_delta(
    ctx: &mut MessagesContext,
    call_id: &str,
    delta: &str,
    out: &mut Vec<MessagesSseEvent>,
) {
    if !ctx.tool_calls.contains_key(call_id) {
        // Lazy open so out-of-order fragments still get a content block.
        emit_tool_call_started(ctx, call_id, call_id, out);
    }
    if let Some(state) = ctx.tool_calls.get_mut(call_id) {
        state.arguments_buffer.push_str(delta);
        out.push(MessagesSseEvent::new(
            "content_block_delta",
            json!({
                "type": "content_block_delta",
                "index": state.index,
                "delta": { "type": "input_json_delta", "partial_json": delta },
            }),
        ));
    }
}

fn emit_tool_call_done(
    ctx: &mut MessagesContext,
    call_id: &str,
    arguments: &Value,
    out: &mut Vec<MessagesSseEvent>,
) {
    let final_string = match arguments {
        Value::String(text) => text.clone(),
        other => other.to_string(),
    };
    if let Some(state) = ctx.tool_calls.get_mut(call_id) {
        state.arguments_buffer = final_string;
    }
    close_single_tool_call(ctx, call_id, out);
}

fn close_single_tool_call(
    ctx: &mut MessagesContext,
    call_id: &str,
    out: &mut Vec<MessagesSseEvent>,
) {
    let Some(state) = ctx.tool_calls.remove(call_id) else {
        return;
    };
    out.push(MessagesSseEvent::new(
        "content_block_stop",
        json!({
            "type": "content_block_stop",
            "index": state.index,
        }),
    ));
    let input_value: Value = serde_json::from_str(&state.arguments_buffer)
        .unwrap_or(Value::String(state.arguments_buffer));
    ctx.completed_blocks.push(json!({
        "type": "tool_use",
        "id": state.id,
        "name": state.name,
        "input": input_value,
    }));
}

fn close_open_tool_calls(ctx: &mut MessagesContext, out: &mut Vec<MessagesSseEvent>) {
    let pending: Vec<String> = ctx.tool_call_order.clone();
    for call_id in pending {
        close_single_tool_call(ctx, &call_id, out);
    }
    ctx.tool_call_order.clear();
}

fn message_delta(ctx: &MessagesContext, finish: CursorFinishReason) -> MessagesSseEvent {
    MessagesSseEvent::new(
        "message_delta",
        json!({
            "type": "message_delta",
            "delta": {
                "stop_reason": anthropic_stop_reason(finish),
                "stop_sequence": Value::Null,
            },
            "usage": {
                "input_tokens": ctx.input_tokens,
                "output_tokens": ctx.output_tokens,
            },
        }),
    )
}

fn error_event(code: &str, message: &str) -> MessagesSseEvent {
    MessagesSseEvent::new(
        "error",
        json!({
            "type": "error",
            "error": { "type": code, "message": message },
        }),
    )
}

// ---------------------------------------------------------------------------
// Request parsing
// ---------------------------------------------------------------------------

fn anthropic_system_text(value: &Value) -> Option<AppResult<String>> {
    match value {
        Value::String(text) => Some(Ok(text.clone())),
        Value::Array(blocks) => {
            let mut out = Vec::new();
            for block in blocks {
                let block_type = block.get("type").and_then(Value::as_str).unwrap_or("text");
                if block_type != "text" {
                    return Some(Err(AppError::BadRequest(format!(
                        "system content block {block_type} is not supported"
                    ))));
                }
                if let Some(text) = block.get("text").and_then(Value::as_str) {
                    out.push(text.to_string());
                }
            }
            if out.is_empty() {
                None
            } else {
                Some(Ok(out.join("")))
            }
        }
        Value::Null => None,
        _ => Some(Err(AppError::BadRequest(
            "system must be a string or array of text blocks".into(),
        ))),
    }
}

fn convert_anthropic_messages(
    messages: &[Value],
) -> AppResult<(Vec<CursorMessage>, Vec<CursorToolResult>)> {
    let mut normalized = Vec::new();
    let mut tool_results = Vec::new();
    for message in messages {
        let object = message
            .as_object()
            .ok_or_else(|| AppError::BadRequest("message must be an object".into()))?;
        let role = required_string(object, "role")?;
        match role {
            "user" => {
                let (blocks, mut user_tool_results) =
                    convert_anthropic_user_content(object.get("content"))?;
                if !blocks.is_empty() {
                    normalized.push(CursorMessage::User { blocks });
                }
                tool_results.append(&mut user_tool_results);
            }
            "assistant" => {
                let (blocks, tool_calls) =
                    convert_anthropic_assistant_content(object.get("content"))?;
                if !blocks.is_empty() || !tool_calls.is_empty() {
                    normalized.push(CursorMessage::Assistant { blocks, tool_calls });
                }
            }
            other => {
                return Err(AppError::BadRequest(format!(
                    "unsupported message role: {other}"
                )))
            }
        }
    }
    Ok((normalized, tool_results))
}

fn convert_anthropic_user_content(
    content: Option<&Value>,
) -> AppResult<(Vec<CursorContentBlock>, Vec<CursorToolResult>)> {
    let raw_blocks = anthropic_content_blocks(content)?;
    let mut blocks = Vec::new();
    let mut tool_results = Vec::new();
    for block in raw_blocks {
        let block_type = block.get("type").and_then(Value::as_str).unwrap_or("text");
        match block_type {
            "text" => {
                if let Some(text) = block.get("text").and_then(Value::as_str) {
                    blocks.push(CursorContentBlock::Text(text.to_string()));
                }
            }
            "image" => {
                return Err(AppError::BadRequest(
                    "image content is not supported for Cursor Composer".into(),
                ));
            }
            "tool_result" => {
                let call_id = block
                    .get("tool_use_id")
                    .and_then(Value::as_str)
                    .ok_or_else(|| AppError::BadRequest("tool_use_id is required".into()))?
                    .to_string();
                let output_text = anthropic_tool_result_output(&block)?;
                let output_value: Value = serde_json::from_str(&output_text)
                    .ok()
                    .filter(|value| !matches!(value, Value::String(_)))
                    .unwrap_or_else(|| Value::String(output_text.clone()));
                let is_error = block
                    .get("is_error")
                    .and_then(Value::as_bool)
                    .unwrap_or(false);
                tool_results.push(CursorToolResult {
                    call_id,
                    output: output_value,
                    error: if is_error { Some(output_text) } else { None },
                });
            }
            other => {
                return Err(AppError::BadRequest(format!(
                    "unsupported user content block: {other}"
                )))
            }
        }
    }
    Ok((blocks, tool_results))
}

fn convert_anthropic_assistant_content(
    content: Option<&Value>,
) -> AppResult<(Vec<CursorContentBlock>, Vec<CursorToolCall>)> {
    let raw_blocks = anthropic_content_blocks(content)?;
    let mut blocks = Vec::new();
    let mut tool_calls = Vec::new();
    for block in raw_blocks {
        let block_type = block.get("type").and_then(Value::as_str).unwrap_or("text");
        match block_type {
            "text" => {
                if let Some(text) = block.get("text").and_then(Value::as_str) {
                    blocks.push(CursorContentBlock::Text(text.to_string()));
                }
            }
            "thinking" => {}
            "tool_use" => {
                let id = block
                    .get("id")
                    .and_then(Value::as_str)
                    .ok_or_else(|| AppError::BadRequest("tool_use.id is required".into()))?
                    .to_string();
                let name = block
                    .get("name")
                    .and_then(Value::as_str)
                    .ok_or_else(|| AppError::BadRequest("tool_use.name is required".into()))?
                    .to_string();
                let arguments = block.get("input").cloned().unwrap_or_else(|| json!({}));
                tool_calls.push(CursorToolCall {
                    id,
                    name,
                    arguments,
                });
            }
            other => {
                return Err(AppError::BadRequest(format!(
                    "unsupported assistant content block: {other}"
                )))
            }
        }
    }
    Ok((blocks, tool_calls))
}

fn anthropic_content_blocks(content: Option<&Value>) -> AppResult<Vec<Value>> {
    match content {
        Some(Value::String(text)) => Ok(vec![json!({ "type": "text", "text": text })]),
        Some(Value::Array(blocks)) => Ok(blocks.clone()),
        Some(Value::Null) | None => Ok(Vec::new()),
        Some(_) => Err(AppError::BadRequest(
            "message content must be a string or array".into(),
        )),
    }
}

fn anthropic_tool_result_output(block: &Value) -> AppResult<String> {
    match block.get("content") {
        Some(Value::String(text)) => Ok(text.clone()),
        Some(Value::Array(blocks)) => {
            let mut out = Vec::new();
            for block in blocks {
                let block_type = block.get("type").and_then(Value::as_str).unwrap_or("text");
                if block_type != "text" {
                    return Err(AppError::BadRequest(
                        "tool_result content must be text only".into(),
                    ));
                }
                if let Some(text) = block.get("text").and_then(Value::as_str) {
                    out.push(text.to_string());
                }
            }
            Ok(out.join(""))
        }
        Some(Value::Null) | None => Ok(String::new()),
        Some(_) => Err(AppError::BadRequest(
            "tool_result content must be a string or array".into(),
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
        if let Some(tool_type) = object.get("type").and_then(Value::as_str) {
            if tool_type != "custom" && tool_type != "function" {
                return Err(AppError::BadRequest(format!(
                    "tool type {tool_type} is not supported"
                )));
            }
        }
        let name = required_string(object, "name")?;
        validate_tool_name(name)?;
        let description = object
            .get("description")
            .and_then(Value::as_str)
            .map(str::to_string);
        let parameters_schema = object
            .get("input_schema")
            .or_else(|| object.get("parameters"))
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
    let object = value
        .as_object()
        .ok_or_else(|| AppError::BadRequest("tool_choice must be an object".into()))?;
    match object.get("type").and_then(Value::as_str) {
        Some("auto") | Some("any") | Some("none") => Ok(()),
        Some("tool") => {
            required_string(object, "name")?;
            Ok(())
        }
        Some(other) => Err(AppError::BadRequest(format!(
            "tool_choice type {other} is not supported"
        ))),
        None => Err(AppError::BadRequest("tool_choice.type is required".into())),
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
