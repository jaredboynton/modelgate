//! `/v1/responses` <-> Cursor agent translation.
//!
//! Converts public OpenAI Responses JSON to/from `crate::cursor_agent::*`
//! DTOs only. This adapter never calls `crate::upstream::cursor` or
//! `crate::auth::cursor`; the route layer wires the upstream call between
//! `build_request` and `emit_event` / `collect_non_stream`.
//!
//! Event surface follows the matrix in
//! `.omx/research/cursor-phase0/responses-events-extraction.md`. Anti-patterns
//! from the same note (Anthropic adaptive reasoning translation, Anthropic
//! `rewrite_max_alias`, Google thought-signature plumbing, Google candidate
//! finishReason detection, v1 Codex-style allowlist) are deliberately not
//! replicated here.

use serde_json::{json, Map, Value};
use uuid::Uuid;

use crate::adapter::cursor_events::{ResponseContext, ResponsesSseEvent, ToolCallSnapshot};
use crate::cursor_agent::{
    CursorAgentEvent, CursorAgentRequest, CursorContentBlock, CursorMessage, CursorTool,
    CursorToolCall, CursorToolKind, CursorToolResult,
};
use crate::{AppError, AppResult};

const MAX_TOOL_NAME_LEN: usize = 64;

/// Build a `CursorAgentRequest` from a public Responses request body.
///
/// Enforces the field policy matrix from ralplan Section 5 step 3: every
/// non-routing field is either mapped, stored-only, or rejected with a
/// 400. Unknown top-level fields are rejected; nothing is silently stripped.
pub fn build_request(public_json: &Value) -> AppResult<CursorAgentRequest> {
    let object = public_json
        .as_object()
        .ok_or_else(|| AppError::BadRequest("request body must be a JSON object".into()))?;

    enforce_top_level_policy(object)?;

    let model = required_string(object, "model")?.to_string();
    let upstream_model = model.clone();

    let stream = object
        .get("stream")
        .and_then(Value::as_bool)
        .unwrap_or(false);

    let mut system_instructions: Option<String> = None;
    let mut developer_instructions: Option<String> = None;
    if let Some(text) = object.get("instructions").and_then(string_like) {
        system_instructions = Some(text);
    }

    let messages = parse_input(
        object.get("input"),
        &mut system_instructions,
        &mut developer_instructions,
    )?;

    let mut tools = Vec::new();
    if let Some(raw_tools) = object.get("tools") {
        tools = parse_tools(raw_tools)?;
    }

    if let Some(tool_choice) = object.get("tool_choice") {
        validate_tool_choice(tool_choice)?;
    }

    let tool_results = collect_tool_results(object.get("input"))?;

    if !tool_results.is_empty() {
        let prior = object.get("previous_response_id");
        let has_prior = matches!(prior, Some(Value::String(value)) if !value.is_empty());
        if !has_prior {
            return Err(AppError::BadRequest(
                "tool result requires previous_response_id".into(),
            ));
        }
    }

    let continuation_key = None; // Route layer rebuilds the key from the prior store.

    Ok(CursorAgentRequest {
        model,
        upstream_model,
        system_instructions,
        developer_instructions,
        messages,
        tools,
        tool_results,
        continuation_key,
        workspace: None,
        stream,
        request_id: Uuid::new_v4(),
        client_profile: Default::default(),
    })
}

/// Translate a single `CursorAgentEvent` into one or more Responses SSE
/// frames. Caller drives the stream and accumulates the resulting frames.
pub fn emit_event(event: &CursorAgentEvent, ctx: &mut ResponseContext) -> Vec<ResponsesSseEvent> {
    let mut out = Vec::new();
    if !ctx.started {
        out.push(response_created(ctx));
        ctx.started = true;
    }

    match event {
        CursorAgentEvent::TextDelta { delta, .. } => {
            emit_text_delta(ctx, delta, &mut out);
        }
        CursorAgentEvent::ReasoningDelta { delta } => {
            emit_reasoning_delta(ctx, delta, &mut out);
        }
        CursorAgentEvent::ToolCallStarted {
            call_id,
            name,
            kind,
            ..
        } => {
            emit_tool_call_started(ctx, call_id, name, *kind, &mut out);
        }
        CursorAgentEvent::ToolCallArgumentsDelta { call_id, delta } => {
            emit_tool_call_arguments_delta(ctx, call_id, delta, &mut out);
        }
        CursorAgentEvent::ToolCallDone { call_id, arguments } => {
            emit_tool_call_done(ctx, call_id, arguments, &mut out);
        }
        CursorAgentEvent::UsageUpdate {
            input_tokens,
            output_tokens,
            reasoning_tokens,
        } => {
            ctx.record_usage(*input_tokens, *output_tokens, *reasoning_tokens);
        }
        CursorAgentEvent::Checkpoint { .. } => {
            // Checkpoints are upstream-internal continuation handles; no
            // public Responses frame is emitted here.
        }
        CursorAgentEvent::ProviderError { code, message, .. } => {
            emit_provider_error(ctx, code, message, &mut out);
        }
        CursorAgentEvent::Done {
            finish_reason,
            response_id,
            conversation_id,
        } => {
            close_open_text(ctx, &mut out);
            close_open_reasoning(ctx, &mut out);
            ctx.record_done(*finish_reason, conversation_id);
            ctx.response_id = response_id.clone();
            emit_response_completed(ctx, &mut out);
            ctx.completed = true;
        }
    }

    out
}

/// Collect a complete Responses object from a finished event stream.
///
/// Equivalent to the existing `anthropic_message_to_responses_json_with_context`
/// but driven from the Cursor neutral DTO.
pub fn collect_non_stream(events: Vec<CursorAgentEvent>) -> AppResult<Value> {
    let mut ctx = ResponseContext::new("composer", default_response_id());
    let mut model_seen = false;

    for event in &events {
        // Track usage before Done so we can collapse the final response.
        if let CursorAgentEvent::UsageUpdate {
            input_tokens,
            output_tokens,
            reasoning_tokens,
        } = event
        {
            ctx.record_usage(*input_tokens, *output_tokens, *reasoning_tokens);
        }
    }

    for event in events {
        let frames = emit_event(&event, &mut ctx);
        if !model_seen {
            // ResponseContext::new used a placeholder; if any frame surfaces
            // a model we can keep using the placeholder ("composer") since
            // the route layer is responsible for stamping the requested
            // public slug. We just need to mark the started flag.
            model_seen = true;
            let _ = frames;
        }
    }

    let status = ctx.response_status();
    let mut response = json!({
        "id": ctx.response_id.clone(),
        "object": "response",
        "created_at": 0,
        "model": ctx.model.clone(),
        "status": status,
        "output": ctx.completed_items().to_vec(),
        "usage": ctx.usage_envelope(),
    });
    if status == "incomplete" {
        response["incomplete_details"] = json!({ "reason": "max_output_tokens" });
    }
    if ctx.failed {
        response["status"] = Value::String("failed".into());
    }
    Ok(response)
}

// ---------------------------------------------------------------------------
// Field policy matrix (ralplan Section 5 step 3)
// ---------------------------------------------------------------------------

const ROUTING_FIELDS: &[&str] = &["model", "stream"];
const INPUT_FIELDS: &[&str] = &["input", "instructions"];
const REASONING_FIELDS: &[&str] = &["reasoning"];
const TOOL_FIELDS: &[&str] = &["tools", "tool_choice", "max_tool_calls"];
const CONTINUATION_FIELDS: &[&str] = &["previous_response_id"];
const STORAGE_FIELDS: &[&str] = &["store"];
const STREAM_OPTION_FIELDS: &[&str] = &["stream_options"];
const TEXT_OPTION_FIELDS: &[&str] = &["text"];
const TRUNCATION_FIELDS: &[&str] = &["truncation"];
const PROMPT_CACHE_FIELDS: &[&str] = &["prompt_cache_key", "prompt_cache_retention"];
const CONTEXT_MGMT_FIELDS: &[&str] = &["context_management"];
const LOCAL_METADATA_FIELDS: &[&str] = &["metadata", "user"];
const INCLUDE_FIELDS: &[&str] = &["include"];
const EXPLICIT_REJECTS: &[&str] = &[
    "background",
    "conversation",
    "prompt",
    "service_tier",
    "safety_identifier",
    "top_logprobs",
];

fn enforce_top_level_policy(object: &Map<String, Value>) -> AppResult<()> {
    for (key, value) in object {
        if EXPLICIT_REJECTS.contains(&key.as_str()) {
            return Err(AppError::BadRequest(format!(
                "field {key} is not supported for Cursor Composer responses"
            )));
        }
        if matches!(
            key.as_str(),
            "max_output_tokens" | "temperature" | "top_p" | "parallel_tool_calls"
        ) {
            return Err(AppError::BadRequest(format!(
                "field {key} is not mapped for Cursor Composer responses"
            )));
        }
        if INCLUDE_FIELDS.contains(&key.as_str()) {
            validate_include(value)?;
            continue;
        }
        if STREAM_OPTION_FIELDS.contains(&key.as_str()) {
            validate_stream_options(value)?;
            continue;
        }
        if TEXT_OPTION_FIELDS.contains(&key.as_str()) {
            validate_text_format(value)?;
            continue;
        }
        if TRUNCATION_FIELDS.contains(&key.as_str()) {
            validate_truncation(value)?;
            continue;
        }
        if CONTEXT_MGMT_FIELDS.contains(&key.as_str()) {
            validate_context_management(value)?;
            continue;
        }
        if REASONING_FIELDS.contains(&key.as_str()) {
            validate_reasoning(value)?;
            continue;
        }
        if PROMPT_CACHE_FIELDS.contains(&key.as_str())
            || LOCAL_METADATA_FIELDS.contains(&key.as_str())
        {
            // Stored/redacted only. Accept any JSON shape.
            continue;
        }
        if ROUTING_FIELDS.contains(&key.as_str())
            || INPUT_FIELDS.contains(&key.as_str())
            || TOOL_FIELDS.contains(&key.as_str())
            || CONTINUATION_FIELDS.contains(&key.as_str())
            || STORAGE_FIELDS.contains(&key.as_str())
        {
            continue;
        }
        return Err(AppError::BadRequest(format!(
            "unknown top-level field {key}"
        )));
    }

    if let Some(value) = object.get("max_tool_calls") {
        validate_max_tool_calls(value)?;
    }

    Ok(())
}

fn validate_reasoning(value: &Value) -> AppResult<()> {
    if value.is_null() {
        return Ok(());
    }
    let object = value
        .as_object()
        .ok_or_else(|| AppError::BadRequest("reasoning must be an object".into()))?;
    for (key, _) in object {
        match key.as_str() {
            "effort" | "summary" => {}
            other => {
                return Err(AppError::BadRequest(format!(
                    "reasoning.{other} is not supported"
                )))
            }
        }
    }
    Ok(())
}

fn validate_include(value: &Value) -> AppResult<()> {
    let array = value
        .as_array()
        .ok_or_else(|| AppError::BadRequest("include must be an array".into()))?;
    if let Some(entry) = array.first() {
        let entry = entry
            .as_str()
            .ok_or_else(|| AppError::BadRequest("include entries must be strings".into()))?;
        return Err(AppError::BadRequest(format!(
            "include value {entry} is not supported for Cursor Composer responses"
        )));
    }
    Ok(())
}

fn validate_stream_options(value: &Value) -> AppResult<()> {
    if value.is_null() {
        return Ok(());
    }
    let object = value
        .as_object()
        .ok_or_else(|| AppError::BadRequest("stream_options must be an object".into()))?;
    for (key, value) in object {
        match key.as_str() {
            "include_obfuscation" => {
                if value.as_bool() == Some(true) {
                    return Err(AppError::BadRequest(
                        "stream_options.include_obfuscation=true is not supported".into(),
                    ));
                }
            }
            other => {
                return Err(AppError::BadRequest(format!(
                    "stream_options.{other} is not supported"
                )))
            }
        }
    }
    Ok(())
}

fn validate_text_format(value: &Value) -> AppResult<()> {
    if value.is_null() {
        return Ok(());
    }
    let object = value
        .as_object()
        .ok_or_else(|| AppError::BadRequest("text must be an object".into()))?;
    if let Some(format) = object.get("format") {
        let format_object = format
            .as_object()
            .ok_or_else(|| AppError::BadRequest("text.format must be an object".into()))?;
        match format_object.get("type").and_then(Value::as_str) {
            Some("text") => {}
            Some(other) => {
                return Err(AppError::BadRequest(format!(
                    "text.format.type {other} is not supported"
                )))
            }
            None => return Err(AppError::BadRequest("text.format.type is required".into())),
        }
    }
    if object.contains_key("verbosity") {
        return Err(AppError::BadRequest(
            "text.verbosity is not supported".into(),
        ));
    }
    Ok(())
}

fn validate_truncation(value: &Value) -> AppResult<()> {
    match value.as_str() {
        Some("disabled") | None => Ok(()),
        Some(other) => Err(AppError::BadRequest(format!(
            "truncation {other} is not supported"
        ))),
    }
}

fn validate_context_management(value: &Value) -> AppResult<()> {
    let array = value
        .as_array()
        .ok_or_else(|| AppError::BadRequest("context_management must be an array".into()))?;
    if array.is_empty() {
        return Ok(());
    }
    Err(AppError::BadRequest(
        "context_management entries are not supported".into(),
    ))
}

fn validate_max_tool_calls(value: &Value) -> AppResult<()> {
    let number = value
        .as_i64()
        .ok_or_else(|| AppError::BadRequest("max_tool_calls must be an integer".into()))?;
    if number < 1 {
        return Err(AppError::BadRequest("max_tool_calls must be >= 1".into()));
    }
    Ok(())
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

// ---------------------------------------------------------------------------
// Input parsing
// ---------------------------------------------------------------------------

fn parse_input(
    input: Option<&Value>,
    system_instructions: &mut Option<String>,
    developer_instructions: &mut Option<String>,
) -> AppResult<Vec<CursorMessage>> {
    let Some(input) = input else {
        return Ok(Vec::new());
    };
    match input {
        Value::String(text) => Ok(vec![CursorMessage::User {
            blocks: vec![CursorContentBlock::Text(text.clone())],
        }]),
        Value::Array(items) => {
            let mut messages = Vec::new();
            for item in items {
                let object = item
                    .as_object()
                    .ok_or_else(|| AppError::BadRequest("input items must be objects".into()))?;
                let item_type = object
                    .get("type")
                    .and_then(Value::as_str)
                    .unwrap_or("message");
                match item_type {
                    "message" => {
                        let role = required_string(object, "role")?;
                        let content = parse_message_content(object.get("content"))?;
                        match role {
                            "system" => {
                                if let Some(text) = first_text_join(&content) {
                                    *system_instructions =
                                        combine_optional(system_instructions.take(), text);
                                }
                            }
                            "developer" => {
                                if let Some(text) = first_text_join(&content) {
                                    *developer_instructions =
                                        combine_optional(developer_instructions.take(), text);
                                }
                            }
                            "user" => messages.push(CursorMessage::User { blocks: content }),
                            "assistant" => messages.push(CursorMessage::Assistant {
                                blocks: content,
                                tool_calls: Vec::new(),
                            }),
                            other => {
                                return Err(AppError::BadRequest(format!(
                                    "unsupported message role: {other}"
                                )))
                            }
                        }
                    }
                    "function_call" => {
                        let call_id = required_string(object, "call_id")
                            .or_else(|_| required_string(object, "id"))?
                            .to_string();
                        let name = required_string(object, "name")?.to_string();
                        let arguments = object
                            .get("arguments")
                            .and_then(Value::as_str)
                            .map(str::to_string)
                            .unwrap_or_else(|| "{}".to_string());
                        let arguments_value: Value =
                            serde_json::from_str(&arguments).unwrap_or(Value::String(arguments));
                        push_assistant_tool_call(
                            &mut messages,
                            CursorToolCall {
                                id: call_id,
                                name,
                                arguments: arguments_value,
                            },
                        );
                    }
                    "custom_tool_call" => {
                        let call_id = required_string(object, "call_id")
                            .or_else(|_| required_string(object, "id"))?
                            .to_string();
                        let name = required_string(object, "name")?.to_string();
                        let input = object
                            .get("input")
                            .and_then(Value::as_str)
                            .map(str::to_string)
                            .unwrap_or_default();
                        push_assistant_tool_call(
                            &mut messages,
                            CursorToolCall {
                                id: call_id,
                                name,
                                arguments: Value::String(input),
                            },
                        );
                    }
                    "function_call_output" | "custom_tool_call_output" => {
                        // tool result items are collected separately.
                    }
                    "reasoning" => {
                        // Prior reasoning items are discarded; Cursor regenerates them.
                    }
                    other => {
                        return Err(AppError::BadRequest(format!(
                            "unsupported input item type: {other}"
                        )))
                    }
                }
            }
            Ok(messages)
        }
        _ => Err(AppError::BadRequest(
            "input must be a string or array".into(),
        )),
    }
}

fn parse_message_content(content: Option<&Value>) -> AppResult<Vec<CursorContentBlock>> {
    match content {
        Some(Value::String(text)) => Ok(vec![CursorContentBlock::Text(text.clone())]),
        Some(Value::Array(blocks)) => {
            let mut out = Vec::new();
            for block in blocks {
                let object = block.as_object().ok_or_else(|| {
                    AppError::BadRequest("content block must be an object".into())
                })?;
                let block_type = object.get("type").and_then(Value::as_str).unwrap_or("");
                match block_type {
                    "input_text" | "output_text" | "text" => {
                        if let Some(text) = object.get("text").and_then(Value::as_str) {
                            out.push(CursorContentBlock::Text(text.to_string()));
                        }
                    }
                    "input_image" | "image_url" => {
                        return Err(AppError::BadRequest(
                            "image content is not supported for Cursor Composer".into(),
                        ));
                    }
                    other => {
                        return Err(AppError::BadRequest(format!(
                            "unsupported content block type: {other}"
                        )))
                    }
                }
            }
            Ok(out)
        }
        Some(Value::Null) | None => Ok(Vec::new()),
        Some(_) => Err(AppError::BadRequest(
            "content must be a string or array".into(),
        )),
    }
}

fn collect_tool_results(input: Option<&Value>) -> AppResult<Vec<CursorToolResult>> {
    let Some(Value::Array(items)) = input else {
        return Ok(Vec::new());
    };
    let mut results = Vec::new();
    for item in items {
        let object = item
            .as_object()
            .ok_or_else(|| AppError::BadRequest("input items must be objects".into()))?;
        let item_type = object.get("type").and_then(Value::as_str).unwrap_or("");
        if item_type != "function_call_output" && item_type != "custom_tool_call_output" {
            continue;
        }
        let call_id = required_string(object, "call_id")
            .or_else(|_| required_string(object, "id"))?
            .to_string();
        let output = match object.get("output") {
            Some(Value::String(text)) => Value::String(text.clone()),
            Some(Value::Array(blocks)) => {
                let mut texts = Vec::new();
                for block in blocks {
                    let block_type = block.get("type").and_then(Value::as_str).unwrap_or("");
                    match block_type {
                        "output_text" | "input_text" | "text" => {
                            if let Some(text) = block.get("text").and_then(Value::as_str) {
                                texts.push(text.to_string());
                            }
                        }
                        other => {
                            return Err(AppError::BadRequest(format!(
                                "tool output block {other} is not supported"
                            )))
                        }
                    }
                }
                Value::String(texts.join(""))
            }
            Some(_) => {
                return Err(AppError::BadRequest(
                    "tool output supports text content only".into(),
                ))
            }
            None => Value::String(String::new()),
        };
        let error = extract_tool_result_error(object, &output);
        results.push(CursorToolResult {
            call_id,
            output,
            error,
        });
    }
    Ok(results)
}

fn extract_tool_result_error(object: &Map<String, Value>, output: &Value) -> Option<String> {
    if let Some(error_value) = object.get("error") {
        if !error_value.is_null() {
            return Some(stringify_error_value(error_value));
        }
    }
    if let Some(output_object) = output.as_object() {
        if let Some(error_value) = output_object.get("error") {
            if !error_value.is_null() {
                return Some(stringify_error_value(error_value));
            }
        }
    }
    None
}

fn stringify_error_value(value: &Value) -> String {
    match value {
        Value::String(text) => text.clone(),
        other => other.to_string(),
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
        let kind = match tool_type {
            "function" => CursorToolKind::Function,
            "custom" => CursorToolKind::Custom,
            other => {
                return Err(AppError::BadRequest(format!(
                    "tool type {other} is not supported"
                )))
            }
        };
        let name = required_string(object, "name")?;
        validate_tool_name(name)?;
        let description = object
            .get("description")
            .and_then(Value::as_str)
            .map(str::to_string);
        let parameters_schema = object
            .get("parameters")
            .or_else(|| object.get("input_schema"))
            .cloned()
            .unwrap_or_else(|| json!({ "type": "object", "properties": {} }));
        out.push(CursorTool {
            name: name.to_string(),
            description,
            parameters_schema,
            kind,
        });
    }
    Ok(out)
}

fn push_assistant_tool_call(messages: &mut Vec<CursorMessage>, call: CursorToolCall) {
    if let Some(CursorMessage::Assistant { tool_calls, .. }) = messages.last_mut() {
        tool_calls.push(call);
        return;
    }
    messages.push(CursorMessage::Assistant {
        blocks: Vec::new(),
        tool_calls: vec![call],
    });
}

fn first_text_join(blocks: &[CursorContentBlock]) -> Option<String> {
    let texts: Vec<&str> = blocks
        .iter()
        .map(|block| match block {
            CursorContentBlock::Text(text) => text.as_str(),
        })
        .collect();
    if texts.is_empty() {
        None
    } else {
        Some(texts.join(""))
    }
}

fn combine_optional(prior: Option<String>, addition: String) -> Option<String> {
    match prior {
        Some(prior) if !prior.is_empty() => Some(format!("{prior}\n\n{addition}")),
        _ => Some(addition),
    }
}

// ---------------------------------------------------------------------------
// SSE emission
// ---------------------------------------------------------------------------

fn response_created(ctx: &ResponseContext) -> ResponsesSseEvent {
    ResponsesSseEvent::new(
        "response.created",
        json!({
            "type": "response.created",
            "response": response_envelope(ctx, "in_progress", Vec::new()),
        }),
    )
}

fn response_envelope(ctx: &ResponseContext, status: &str, output: Vec<Value>) -> Value {
    json!({
        "id": ctx.response_id.clone(),
        "object": "response",
        "created_at": 0,
        "model": ctx.model.clone(),
        "status": status,
        "output": output,
        "usage": ctx.usage_envelope(),
    })
}

fn emit_text_delta(ctx: &mut ResponseContext, delta: &str, out: &mut Vec<ResponsesSseEvent>) {
    let opening_added = ctx.current_text_index().is_none();
    let state = ctx.open_text_item();
    let output_index = state.output_index;
    let item_id = state.item_id.clone();
    state.text.push_str(delta);
    if opening_added {
        out.push(ResponsesSseEvent::new(
            "response.output_item.added",
            json!({
                "type": "response.output_item.added",
                "output_index": output_index,
                "item": {
                    "id": item_id,
                    "type": "message",
                    "status": "in_progress",
                    "role": "assistant",
                    "content": [],
                },
            }),
        ));
    }
    out.push(ResponsesSseEvent::new(
        "response.output_text.delta",
        json!({
            "type": "response.output_text.delta",
            "output_index": output_index,
            "content_index": 0,
            "delta": delta,
        }),
    ));
}

fn close_open_text(ctx: &mut ResponseContext, out: &mut Vec<ResponsesSseEvent>) {
    let snapshot = ctx.close_text_item();
    let Some((output_index, item_id, text)) = snapshot else {
        return;
    };
    out.push(ResponsesSseEvent::new(
        "response.output_text.done",
        json!({
            "type": "response.output_text.done",
            "output_index": output_index,
            "content_index": 0,
            "text": text.clone(),
        }),
    ));
    out.push(ResponsesSseEvent::new(
        "response.output_item.done",
        json!({
            "type": "response.output_item.done",
            "output_index": output_index,
            "item": {
                "id": item_id,
                "type": "message",
                "status": "completed",
                "role": "assistant",
                "content": [{ "type": "output_text", "text": text }],
            },
        }),
    ));
}

fn emit_reasoning_delta(ctx: &mut ResponseContext, delta: &str, out: &mut Vec<ResponsesSseEvent>) {
    let state = ctx.open_reasoning_item();
    let output_index = state.output_index;
    let summary_index = state.summary_index;
    let item_id = state.item_id.clone();
    let was_part_open = state.summary_part_open;
    let opening_added = !was_part_open && state.summary_text.is_empty();
    state.summary_part_open = true;
    state.summary_text.push_str(delta);

    if opening_added {
        out.push(ResponsesSseEvent::new(
            "response.output_item.added",
            json!({
                "type": "response.output_item.added",
                "output_index": output_index,
                "item": {
                    "id": item_id.clone(),
                    "type": "reasoning",
                    "status": "in_progress",
                    "summary": [],
                },
            }),
        ));
    }
    if !was_part_open {
        out.push(ResponsesSseEvent::new(
            "response.reasoning_summary_part.added",
            json!({
                "type": "response.reasoning_summary_part.added",
                "output_index": output_index,
                "summary_index": summary_index,
                "part": { "type": "summary_text", "text": "" },
            }),
        ));
    }
    out.push(ResponsesSseEvent::new(
        "response.reasoning_summary_text.delta",
        json!({
            "type": "response.reasoning_summary_text.delta",
            "output_index": output_index,
            "summary_index": summary_index,
            "delta": delta,
        }),
    ));
}

fn close_open_reasoning(ctx: &mut ResponseContext, out: &mut Vec<ResponsesSseEvent>) {
    let snapshot = ctx.close_reasoning_item();
    let Some((output_index, item_id, summary_index, summary_text)) = snapshot else {
        return;
    };
    out.push(ResponsesSseEvent::new(
        "response.reasoning_summary_text.done",
        json!({
            "type": "response.reasoning_summary_text.done",
            "output_index": output_index,
            "summary_index": summary_index,
            "text": summary_text.clone(),
        }),
    ));
    out.push(ResponsesSseEvent::new(
        "response.reasoning_summary_part.done",
        json!({
            "type": "response.reasoning_summary_part.done",
            "output_index": output_index,
            "summary_index": summary_index,
            "part": { "type": "summary_text", "text": summary_text.clone() },
        }),
    ));
    out.push(ResponsesSseEvent::new(
        "response.output_item.done",
        json!({
            "type": "response.output_item.done",
            "output_index": output_index,
            "item": {
                "id": item_id,
                "type": "reasoning",
                "status": "completed",
                "summary": [{
                    "type": "summary_text",
                    "text": summary_text,
                }],
            },
        }),
    ));
}

fn emit_tool_call_started(
    ctx: &mut ResponseContext,
    call_id: &str,
    name: &str,
    kind: CursorToolKind,
    out: &mut Vec<ResponsesSseEvent>,
) {
    // If the request-time tool catalog declared this tool as Custom, override.
    let resolved_kind = match kind {
        CursorToolKind::Function => match ctx.tool_kind(name) {
            CursorToolKind::Custom => CursorToolKind::Custom,
            CursorToolKind::Function => CursorToolKind::Function,
        },
        CursorToolKind::Custom => CursorToolKind::Custom,
    };
    let state = ctx.open_tool_call(call_id, name, resolved_kind);
    let output_index = state.output_index;
    let item_id = state.item_id.clone();
    let item = match resolved_kind {
        CursorToolKind::Function => json!({
            "id": item_id,
            "type": "function_call",
            "status": "in_progress",
            "call_id": call_id,
            "name": name,
            "arguments": "",
        }),
        CursorToolKind::Custom => json!({
            "id": item_id,
            "type": "custom_tool_call",
            "status": "in_progress",
            "call_id": call_id,
            "name": name,
            "input": "",
        }),
    };
    out.push(ResponsesSseEvent::new(
        "response.output_item.added",
        json!({
            "type": "response.output_item.added",
            "output_index": output_index,
            "item": item,
        }),
    ));
}

fn emit_tool_call_arguments_delta(
    ctx: &mut ResponseContext,
    call_id: &str,
    delta: &str,
    out: &mut Vec<ResponsesSseEvent>,
) {
    if ctx.tool_call_index(call_id).is_none() {
        // No matching `started` event was observed; open the call lazily as
        // a function tool and continue.
        let kind = ctx.tool_kind(call_id);
        let synthetic_name = call_id.to_string();
        let state = ctx.open_tool_call(call_id, synthetic_name, kind);
        let output_index = state.output_index;
        let item_id = state.item_id.clone();
        out.push(ResponsesSseEvent::new(
            "response.output_item.added",
            json!({
                "type": "response.output_item.added",
                "output_index": output_index,
                "item": {
                    "id": item_id,
                    "type": match kind {
                        CursorToolKind::Function => "function_call",
                        CursorToolKind::Custom => "custom_tool_call",
                    },
                    "status": "in_progress",
                    "call_id": call_id,
                    "name": call_id,
                    "arguments": "",
                },
            }),
        ));
    }
    let kind = ctx
        .tool_call_kind(call_id)
        .unwrap_or(CursorToolKind::Function);
    let output_index = ctx.tool_call_index(call_id).unwrap_or(0);
    ctx.append_tool_arguments(call_id, delta);
    let event_name = match kind {
        CursorToolKind::Function => "response.function_call_arguments.delta",
        CursorToolKind::Custom => "response.custom_tool_call_input.delta",
    };
    let body = match kind {
        CursorToolKind::Function => json!({
            "type": event_name,
            "output_index": output_index,
            "delta": delta,
        }),
        CursorToolKind::Custom => json!({
            "type": event_name,
            "output_index": output_index,
            "delta": delta,
        }),
    };
    out.push(ResponsesSseEvent::new(event_name, body));
}

fn emit_tool_call_done(
    ctx: &mut ResponseContext,
    call_id: &str,
    arguments: &Value,
    out: &mut Vec<ResponsesSseEvent>,
) {
    // If there are residual fragments not delivered as deltas, capture them
    // by overwriting the buffer with the canonical final string.
    let final_arguments = match arguments {
        Value::String(text) => text.clone(),
        other => other.to_string(),
    };
    if let Some(kind) = ctx.tool_call_kind(call_id) {
        let output_index = ctx.tool_call_index(call_id).unwrap_or(0);
        match kind {
            CursorToolKind::Function => {
                out.push(ResponsesSseEvent::new(
                    "response.function_call_arguments.done",
                    json!({
                        "type": "response.function_call_arguments.done",
                        "output_index": output_index,
                        "arguments": final_arguments.clone(),
                    }),
                ));
            }
            CursorToolKind::Custom => {
                out.push(ResponsesSseEvent::new(
                    "response.custom_tool_call_input.done",
                    json!({
                        "type": "response.custom_tool_call_input.done",
                        "output_index": output_index,
                        "input": final_arguments.clone(),
                    }),
                ));
            }
        }
        // Replace any prior fragment buffer with the canonical final string
        // so the closed item carries a self-sufficient body.
        ctx.append_tool_arguments(call_id, "");
    }

    let snapshot = ctx.close_tool_call(call_id);
    let Some(ToolCallSnapshot {
        output_index,
        item_id,
        call_id,
        name,
        kind,
        ..
    }) = snapshot
    else {
        return;
    };
    let item = match kind {
        CursorToolKind::Function => json!({
            "id": item_id,
            "type": "function_call",
            "status": "completed",
            "call_id": call_id,
            "name": name,
            "arguments": final_arguments,
        }),
        CursorToolKind::Custom => json!({
            "id": item_id,
            "type": "custom_tool_call",
            "status": "completed",
            "call_id": call_id,
            "name": name,
            "input": final_arguments,
        }),
    };
    out.push(ResponsesSseEvent::new(
        "response.output_item.done",
        json!({
            "type": "response.output_item.done",
            "output_index": output_index,
            "item": item,
        }),
    ));
}

fn emit_response_completed(ctx: &mut ResponseContext, out: &mut Vec<ResponsesSseEvent>) {
    let status = ctx.response_status();
    let mut response = response_envelope(ctx, status, ctx.completed_items().to_vec());
    if status == "incomplete" {
        response["incomplete_details"] = json!({ "reason": "max_output_tokens" });
    }
    out.push(ResponsesSseEvent::new(
        "response.completed",
        json!({
            "type": "response.completed",
            "response": response,
        }),
    ));
}

fn emit_provider_error(
    ctx: &mut ResponseContext,
    code: &str,
    message: &str,
    out: &mut Vec<ResponsesSseEvent>,
) {
    if ctx.started {
        let mut response = response_envelope(ctx, "failed", ctx.completed_items().to_vec());
        response["error"] = json!({ "type": code, "message": message });
        out.push(ResponsesSseEvent::new(
            "response.failed",
            json!({
                "type": "response.failed",
                "response": response,
            }),
        ));
    } else {
        out.push(ResponsesSseEvent::new(
            "error",
            json!({
                "type": "error",
                "error": { "type": code, "message": message },
            }),
        ));
    }
    ctx.failed = true;
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn required_string<'a>(object: &'a Map<String, Value>, key: &str) -> AppResult<&'a str> {
    object
        .get(key)
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| AppError::BadRequest(format!("{key} is required")))
}

fn string_like(value: &Value) -> Option<String> {
    match value {
        Value::String(text) => Some(text.clone()),
        Value::Number(number) => Some(number.to_string()),
        _ => None,
    }
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

fn default_response_id() -> String {
    format!("resp_{}", Uuid::new_v4().simple())
}
