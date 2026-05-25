use std::collections::HashMap;

use bytes::Bytes;
use serde_json::{json, Map, Value};

use crate::{AppError, AppResult};

const DEFAULT_ANTHROPIC_MESSAGE_ID: &str = "msg_responses_adapter";
const DEFAULT_RESPONSE_ID: &str = "resp_responses_adapter";
const MAX_TOOL_NAME_LEN: usize = 64;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum AnthropicThinkingMode {
    Adaptive { supports_xhigh: bool },
    Manual,
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq)]
enum ResponsesToolKind {
    #[default]
    Function,
    Custom,
}

#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub struct ToolContext {
    tools: HashMap<String, ResponsesToolKind>,
}

impl ToolContext {
    pub fn with_custom_tool(mut self, name: &str) -> Self {
        self.record(name, ResponsesToolKind::Custom);
        self
    }

    fn record(&mut self, name: &str, kind: ResponsesToolKind) {
        self.tools.insert(name.to_string(), kind);
    }

    fn kind_for(&self, name: &str) -> ResponsesToolKind {
        self.tools.get(name).copied().unwrap_or_default()
    }

    fn is_custom(&self, name: &str) -> bool {
        self.kind_for(name) == ResponsesToolKind::Custom
    }
}

pub fn anthropic_messages_to_responses(body: Value) -> AppResult<Value> {
    let body = rewrite_max_alias_messages(body)?;
    let object = body
        .as_object()
        .ok_or_else(|| AppError::BadRequest("request body must be a JSON object".into()))?;
    reject_unsupported_top_level(object)?;

    let model = required_string(object, "model")?;
    let mut out = Map::new();
    out.insert("model".into(), Value::String(model.to_string()));

    if let Some(instructions) = anthropic_system_to_instructions(object.get("system"))? {
        out.insert("instructions".into(), Value::String(instructions));
    }

    copy_field(object, &mut out, "temperature", "temperature");
    copy_field(object, &mut out, "top_p", "top_p");
    copy_field(object, &mut out, "stream", "stream");
    copy_field(object, &mut out, "stop_sequences", "stop");
    copy_field(object, &mut out, "max_tokens", "max_output_tokens");

    if let Some(user) = object
        .get("metadata")
        .and_then(|metadata| metadata.get("user_id"))
        .and_then(string_like)
    {
        out.insert("user".into(), Value::String(user));
    }

    if let Some(reasoning) = anthropic_thinking_to_reasoning(object.get("thinking"))? {
        out.insert("reasoning".into(), reasoning);
    }

    if let Some(tools) = object.get("tools") {
        out.insert("tools".into(), convert_anthropic_tools(tools)?);
    }
    if let Some(tool_choice) = object.get("tool_choice") {
        out.insert(
            "tool_choice".into(),
            convert_anthropic_tool_choice(tool_choice)?,
        );
    }

    let messages = object
        .get("messages")
        .and_then(Value::as_array)
        .ok_or_else(|| AppError::BadRequest("messages must be an array".into()))?;
    out.insert(
        "input".into(),
        Value::Array(convert_anthropic_messages(messages)?),
    );

    Ok(Value::Object(out))
}

pub fn chat_completions_to_responses(body: Value) -> AppResult<Value> {
    let body = rewrite_max_alias_chat(body)?;
    let object = body
        .as_object()
        .ok_or_else(|| AppError::BadRequest("request body must be a JSON object".into()))?;
    let model = required_string(object, "model")?;

    let mut out = Map::new();
    out.insert("model".into(), Value::String(model.to_string()));
    copy_field(object, &mut out, "temperature", "temperature");
    copy_field(object, &mut out, "top_p", "top_p");
    copy_field(object, &mut out, "stream", "stream");
    copy_field(object, &mut out, "user", "user");
    copy_field(object, &mut out, "max_tokens", "max_output_tokens");
    copy_field(
        object,
        &mut out,
        "max_completion_tokens",
        "max_output_tokens",
    );

    if let Some(effort) = object.get("reasoning_effort").and_then(Value::as_str) {
        out.insert(
            "reasoning".into(),
            json!({ "effort": effort, "summary": "auto" }),
        );
    }
    if !object.contains_key("service_tier") {
        out.insert("service_tier".into(), Value::String("priority".into()));
    } else {
        copy_field(object, &mut out, "service_tier", "service_tier");
    }
    if let Some(tools) = object.get("tools") {
        out.insert("tools".into(), convert_chat_tools(tools)?);
    }
    if let Some(tool_choice) = object.get("tool_choice") {
        out.insert("tool_choice".into(), convert_chat_tool_choice(tool_choice)?);
    }

    let messages = object
        .get("messages")
        .and_then(Value::as_array)
        .ok_or_else(|| AppError::BadRequest("messages must be an array".into()))?;
    let (instructions, input) = convert_chat_messages(messages)?;
    if let Some(instructions) = instructions {
        out.insert("instructions".into(), Value::String(instructions));
    }
    out.insert("input".into(), Value::Array(input));

    Ok(Value::Object(out))
}

pub fn responses_to_anthropic_messages(body: Value) -> AppResult<Value> {
    responses_to_anthropic_messages_with_context(body).map(|(body, _)| body)
}

pub fn responses_to_anthropic_messages_with_context(
    body: Value,
) -> AppResult<(Value, ToolContext)> {
    let body = rewrite_max_alias(body)?;
    let object = body
        .as_object()
        .ok_or_else(|| AppError::BadRequest("request body must be a JSON object".into()))?;
    reject_unsupported_responses_request(object)?;

    let model = required_string(object, "model")?;
    let mut out = Map::new();
    out.insert("model".into(), Value::String(model.to_string()));
    out.insert(
        "max_tokens".into(),
        object
            .get("max_output_tokens")
            .cloned()
            .unwrap_or_else(|| json!(4096)),
    );

    copy_field(object, &mut out, "temperature", "temperature");
    copy_field(object, &mut out, "top_p", "top_p");
    copy_field(object, &mut out, "stream", "stream");

    if let Some(stop) = object.get("stop") {
        out.insert("stop_sequences".into(), responses_stop_sequences(stop)?);
    }
    if let Some(user) = object.get("user").and_then(Value::as_str) {
        out.insert("metadata".into(), json!({ "user_id": user }));
    }
    let thinking_enabled = if let Some(reasoning) = object.get("reasoning") {
        apply_responses_reasoning_to_anthropic(reasoning, model, &mut out)?
    } else {
        false
    };
    if thinking_enabled {
        reject_sampling_with_thinking(object)?;
    }
    let mut tool_context = ToolContext::default();
    if let Some(tools) = object.get("tools") {
        let (tools, context) = convert_responses_tools_to_anthropic(tools)?;
        tool_context = context;
        out.insert("tools".into(), tools);
    }
    if let Some(tool_choice) = object.get("tool_choice") {
        out.insert(
            "tool_choice".into(),
            convert_responses_tool_choice_to_anthropic(tool_choice, thinking_enabled)?,
        );
    }

    let mut system = Vec::new();
    if let Some(instructions) = object.get("instructions").and_then(Value::as_str) {
        if !instructions.trim().is_empty() {
            system.push(instructions.to_string());
        }
    }

    let input = object
        .get("input")
        .ok_or_else(|| AppError::BadRequest("missing input".into()))?;
    let messages =
        convert_responses_input_to_anthropic_messages(input, &mut system, &mut tool_context)?;
    if messages.is_empty() {
        return Err(AppError::BadRequest("input produced no messages".into()));
    }
    if let Some(system) = nonempty_join(system, "\n\n") {
        out.insert("system".into(), Value::String(system));
    }
    out.insert("messages".into(), Value::Array(messages));

    Ok((Value::Object(out), tool_context))
}

pub fn anthropic_message_to_responses_json(message: Value) -> AppResult<Value> {
    anthropic_message_to_responses_json_with_context(message, &ToolContext::default())
}

pub fn anthropic_message_to_responses_json_with_context(
    message: Value,
    tool_context: &ToolContext,
) -> AppResult<Value> {
    let object = message
        .as_object()
        .ok_or_else(|| AppError::BadRequest("Anthropic message must be a JSON object".into()))?;
    let model = object
        .get("model")
        .and_then(Value::as_str)
        .unwrap_or("anthropic");
    let message_id = object
        .get("id")
        .and_then(Value::as_str)
        .unwrap_or(DEFAULT_ANTHROPIC_MESSAGE_ID);
    let response_id = responses_response_id_for_anthropic_message(message_id);
    let stop_reason = object.get("stop_reason").and_then(Value::as_str);

    let mut output = Vec::new();
    let mut message_content = Vec::new();
    for block in object
        .get("content")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
    {
        match block.get("type").and_then(Value::as_str) {
            Some("text") => {
                if let Some(text) = block.get("text").and_then(Value::as_str) {
                    message_content.push(json!({ "type": "output_text", "text": text }));
                }
            }
            Some("tool_use") => {
                flush_responses_output_message(&mut output, message_id, &mut message_content);
                let input = block.get("input").cloned().unwrap_or_else(|| json!({}));
                let name = block.get("name").and_then(Value::as_str).unwrap_or("tool");
                let call_id = block
                    .get("id")
                    .and_then(Value::as_str)
                    .unwrap_or("call_anthropic_adapter");
                if tool_context.is_custom(name) {
                    output.push(json!({
                        "id": format!("ctc_{}", output.len()),
                        "type": "custom_tool_call",
                        "status": "completed",
                        "call_id": call_id,
                        "name": name,
                        "input": anthropic_custom_tool_input(&input),
                    }));
                } else {
                    output.push(json!({
                        "id": format!("fc_{}", output.len()),
                        "type": "function_call",
                        "status": "completed",
                        "call_id": call_id,
                        "name": name,
                        "arguments": input.to_string(),
                    }));
                }
            }
            Some("thinking") | Some("redacted_thinking") => {}
            Some(other) => {
                return Err(AppError::BadRequest(format!(
                    "unsupported Anthropic content block: {other}"
                )))
            }
            None => {}
        }
    }
    flush_responses_output_message(&mut output, message_id, &mut message_content);

    let status = if stop_reason == Some("max_tokens") {
        "incomplete"
    } else {
        "completed"
    };
    let mut response = json!({
        "id": response_id,
        "object": "response",
        "created_at": 0,
        "model": model,
        "status": status,
        "output": output,
        "usage": responses_usage_for_anthropic_message(object.get("usage")),
    });
    if stop_reason == Some("max_tokens") {
        response["incomplete_details"] = json!({ "reason": "max_output_tokens" });
    }
    Ok(response)
}

pub fn anthropic_sse_to_responses_sse_text(input: &str) -> AppResult<String> {
    let mut translator = AnthropicSseTranslator::default();
    for block in sse_blocks(input) {
        translator.process_block(&block)?;
    }
    Ok(translator.output)
}

pub fn anthropic_sse_to_responses_sse_text_with_model(
    input: &str,
    requested_model: &str,
) -> AppResult<String> {
    let mut translator = AnthropicSseTranslator::with_model(requested_model);
    for block in sse_blocks(input) {
        translator.process_block(&block)?;
    }
    Ok(translator.output)
}

pub fn anthropic_sse_to_responses_sse_text_with_model_and_context(
    input: &str,
    requested_model: &str,
    tool_context: ToolContext,
) -> AppResult<String> {
    let mut translator =
        AnthropicSseTranslator::with_model_and_context(requested_model, tool_context);
    for block in sse_blocks(input) {
        translator.process_block(&block)?;
    }
    Ok(translator.output)
}

#[derive(Default)]
pub struct AnthropicSseStreamTranslator {
    translator: AnthropicSseTranslator,
    pending: Vec<u8>,
}

impl AnthropicSseStreamTranslator {
    pub fn with_model(requested_model: &str) -> Self {
        Self {
            translator: AnthropicSseTranslator::with_model(requested_model),
            pending: Vec::new(),
        }
    }

    pub fn with_model_and_context(requested_model: &str, tool_context: ToolContext) -> Self {
        Self {
            translator: AnthropicSseTranslator::with_model_and_context(
                requested_model,
                tool_context,
            ),
            pending: Vec::new(),
        }
    }

    pub fn push_bytes(&mut self, bytes: Bytes) -> AppResult<Bytes> {
        self.pending.extend_from_slice(&bytes);
        self.process_complete_blocks()
    }

    pub fn finish(&mut self) -> AppResult<Bytes> {
        let mut output = self.process_complete_blocks()?;
        if !self.pending.is_empty() {
            let block = std::str::from_utf8(&self.pending).map_err(|error| {
                AppError::Upstream(format!("invalid Anthropic SSE UTF-8: {error}"))
            })?;
            self.translator.process_block(block)?;
            let final_output = std::mem::take(&mut self.translator.output);
            let _ = self.pending.drain(..);
            if !final_output.is_empty() {
                let mut combined = Vec::with_capacity(output.len() + final_output.len());
                combined.extend_from_slice(&output);
                combined.extend_from_slice(final_output.as_bytes());
                output = Bytes::from(combined);
            }
        }
        Ok(output)
    }

    fn process_complete_blocks(&mut self) -> AppResult<Bytes> {
        let mut output = String::new();
        while let Some((position, separator_len)) = find_sse_separator(&self.pending) {
            let end = position + separator_len;
            let block = std::str::from_utf8(&self.pending[..end]).map_err(|error| {
                AppError::Upstream(format!("invalid Anthropic SSE UTF-8: {error}"))
            })?;
            self.translator.process_block(block)?;
            output.push_str(&std::mem::take(&mut self.translator.output));
            let _ = self.pending.drain(..end);
        }
        Ok(Bytes::from(output))
    }
}

pub fn responses_json_to_anthropic_message(
    response: Value,
    requested_model: &str,
) -> AppResult<Value> {
    let response = response.get("response").unwrap_or(&response);
    if response.get("error").is_some() || response["status"] == "failed" {
        return Err(AppError::Upstream("Responses API returned failure".into()));
    }

    let output = response
        .get("output")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_else(Vec::new);
    let mut content = Vec::new();
    for item in &output {
        push_anthropic_content_for_response_item(item, &mut content);
    }

    let has_tool_use = content
        .iter()
        .any(|block| block.get("type").and_then(Value::as_str) == Some("tool_use"));
    let stop_reason = if has_tool_use {
        "tool_use"
    } else if response["status"] == "incomplete"
        || response
            .get("incomplete_details")
            .and_then(|details| details.get("reason"))
            .and_then(Value::as_str)
            == Some("max_output_tokens")
    {
        "max_tokens"
    } else {
        "end_turn"
    };

    Ok(json!({
        "id": anthropic_message_id(response),
        "type": "message",
        "role": "assistant",
        "model": requested_model,
        "content": content,
        "stop_reason": stop_reason,
        "stop_sequence": Value::Null,
        "usage": anthropic_usage(response),
    }))
}

pub fn responses_sse_to_anthropic_sse_text(
    input: &str,
    requested_model: &str,
) -> AppResult<String> {
    let mut translator = SseTranslator::new(requested_model);
    for block in sse_blocks(input) {
        translator.process_block(&block)?;
    }
    Ok(translator.output)
}

/// Codex's `ReasoningEffort` enum tops out at `xhigh`; the proxy supports a
/// stronger `max` tier on Anthropic models. Catalog entries surface that as a
/// dedicated `*-max` slug. Strip the suffix before resolving the upstream model
/// and force `reasoning.effort = "max"` so downstream conversion picks the
/// `max` budget regardless of the Codex-side reasoning_effort sent.
fn rewrite_max_alias_chat(body: Value) -> AppResult<Value> {
    let mut body = body;
    let Some(object) = body.as_object_mut() else {
        return Ok(body);
    };
    let Some(model) = object.get("model").and_then(Value::as_str) else {
        return Ok(body);
    };
    let Some(stripped) = model.strip_suffix("-max") else {
        return Ok(body);
    };
    let stripped = stripped.to_string();
    object.insert("model".into(), Value::String(stripped));
    object.insert("reasoning_effort".into(), Value::String("max".into()));
    Ok(body)
}

fn rewrite_max_alias_messages(body: Value) -> AppResult<Value> {
    let mut body = body;
    let Some(object) = body.as_object_mut() else {
        return Ok(body);
    };
    let Some(model) = object.get("model").and_then(Value::as_str) else {
        return Ok(body);
    };
    let Some(stripped) = model.strip_suffix("-max") else {
        return Ok(body);
    };
    let stripped = stripped.to_string();
    object.insert("model".into(), Value::String(stripped));
    let thinking = object
        .entry("thinking")
        .or_insert_with(|| json!({ "type": "enabled" }));
    if let Some(thinking_object) = thinking.as_object_mut() {
        thinking_object.insert("effort".into(), Value::String("max".into()));
    } else {
        *thinking = json!({ "type": "enabled", "effort": "max" });
    }
    Ok(body)
}

fn rewrite_max_alias(body: Value) -> AppResult<Value> {
    let mut body = body;
    let Some(object) = body.as_object_mut() else {
        return Ok(body);
    };
    let Some(model) = object.get("model").and_then(Value::as_str) else {
        return Ok(body);
    };
    let Some(stripped) = model.strip_suffix("-max") else {
        return Ok(body);
    };
    let stripped = stripped.to_string();
    object.insert("model".into(), Value::String(stripped));
    let reasoning = object
        .entry("reasoning")
        .or_insert_with(|| json!({ "summary": "auto" }));
    if let Some(reasoning_object) = reasoning.as_object_mut() {
        reasoning_object.insert("effort".into(), Value::String("max".into()));
    } else {
        *reasoning = json!({ "effort": "max", "summary": "auto" });
    }
    Ok(body)
}

fn reject_unsupported_responses_request(object: &Map<String, Value>) -> AppResult<()> {
    for key in ["previous_response_id", "conversation"] {
        if object.get(key).is_some_and(|value| !value.is_null()) {
            return Err(AppError::BadRequest(format!("{key} is not supported yet")));
        }
    }
    if object.get("store").and_then(Value::as_bool) == Some(true) {
        return Err(AppError::BadRequest(
            "store=true is not supported yet".into(),
        ));
    }
    if object
        .get("reasoning")
        .and_then(|reasoning| reasoning.get("encrypted_content"))
        .is_some()
    {
        return Err(AppError::BadRequest(
            "encrypted reasoning state is not supported".into(),
        ));
    }
    Ok(())
}

fn responses_stop_sequences(stop: &Value) -> AppResult<Value> {
    match stop {
        Value::String(_) => Ok(Value::Array(vec![stop.clone()])),
        Value::Array(values) => {
            if values.iter().all(Value::is_string) {
                Ok(stop.clone())
            } else {
                Err(AppError::BadRequest(
                    "stop must be a string or array of strings".into(),
                ))
            }
        }
        Value::Null => Ok(Value::Array(Vec::new())),
        _ => Err(AppError::BadRequest(
            "stop must be a string or array of strings".into(),
        )),
    }
}

fn apply_responses_reasoning_to_anthropic(
    reasoning: &Value,
    model: &str,
    out: &mut Map<String, Value>,
) -> AppResult<bool> {
    if reasoning.is_null() {
        return Ok(false);
    }
    let object = reasoning
        .as_object()
        .ok_or_else(|| AppError::BadRequest("reasoning must be an object".into()))?;
    let effort = object.get("effort").and_then(Value::as_str);
    if effort == Some("none") {
        return Ok(false);
    }
    match anthropic_thinking_mode_for_model(model) {
        AnthropicThinkingMode::Adaptive { supports_xhigh } => {
            let Some(effort) = effort else {
                if object.get("budget_tokens").is_some() {
                    return Err(AppError::BadRequest(
                        "reasoning.budget_tokens is not supported for adaptive Anthropic models; use reasoning.effort".into(),
                    ));
                }
                return Ok(false);
            };
            let effort = adaptive_effort(effort, supports_xhigh)?;
            out.insert("thinking".into(), json!({ "type": "adaptive" }));
            out.insert("output_config".into(), json!({ "effort": effort }));
            Ok(true)
        }
        AnthropicThinkingMode::Manual => {
            let budget_tokens = manual_thinking_budget_tokens(object)?;
            let Some(budget_tokens) = budget_tokens else {
                return Ok(false);
            };
            out.insert(
                "thinking".into(),
                json!({
                    "type": "enabled",
                    "budget_tokens": budget_tokens,
                }),
            );
            Ok(true)
        }
    }
}

fn anthropic_thinking_mode_for_model(model: &str) -> AnthropicThinkingMode {
    let model = normalized_model_key(model);
    if model.contains("claude-opus-4-7") {
        AnthropicThinkingMode::Adaptive {
            supports_xhigh: true,
        }
    } else if model.contains("claude-opus-4-6") || model.contains("claude-sonnet-4-6") {
        AnthropicThinkingMode::Adaptive {
            supports_xhigh: false,
        }
    } else {
        AnthropicThinkingMode::Manual
    }
}

fn normalized_model_key(model: &str) -> String {
    model
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect()
}

fn adaptive_effort(effort: &str, supports_xhigh: bool) -> AppResult<&'static str> {
    match effort {
        "minimal" | "low" => Ok("low"),
        "medium" => Ok("medium"),
        "high" => Ok("high"),
        "xhigh" if supports_xhigh => Ok("xhigh"),
        "xhigh" => Ok("high"),
        "max" => Ok("max"),
        other => Err(AppError::BadRequest(format!(
            "unsupported reasoning effort: {other}"
        ))),
    }
}

fn manual_thinking_budget_tokens(object: &Map<String, Value>) -> AppResult<Option<u64>> {
    let budget_tokens = match object.get("effort").and_then(Value::as_str) {
        Some("none") => return Ok(None),
        Some("minimal") | Some("low") => 1_024,
        Some("medium") => 2_048,
        Some("high") => 4_096,
        Some("xhigh") => 8_192,
        Some("max") => 16_384,
        Some(other) => {
            return Err(AppError::BadRequest(format!(
                "unsupported reasoning effort: {other}"
            )))
        }
        None => match object.get("budget_tokens").and_then(Value::as_u64) {
            Some(tokens) => tokens.max(1_024),
            None => return Ok(None),
        },
    };
    Ok(Some(budget_tokens))
}

fn reject_sampling_with_thinking(object: &Map<String, Value>) -> AppResult<()> {
    if let Some(temperature) = object.get("temperature").and_then(Value::as_f64) {
        if (temperature - 1.0).abs() > f64::EPSILON {
            return Err(AppError::BadRequest(
                "temperature may only be 1 when Anthropic thinking is enabled".into(),
            ));
        }
    }
    if object.get("top_p").is_some_and(|value| !value.is_null()) {
        return Err(AppError::BadRequest(
            "top_p is not supported when Anthropic thinking is enabled".into(),
        ));
    }
    Ok(())
}

fn convert_responses_input_to_anthropic_messages(
    input: &Value,
    system: &mut Vec<String>,
    tool_context: &mut ToolContext,
) -> AppResult<Vec<Value>> {
    match input {
        Value::String(text) => Ok(vec![json!({ "role": "user", "content": text })]),
        Value::Array(items) => {
            let mut messages = Vec::new();
            let mut pending_tool_uses = Vec::new();
            let mut pending_tool_results = Vec::new();
            for item in items {
                let object = item
                    .as_object()
                    .ok_or_else(|| AppError::BadRequest("input item must be an object".into()))?;
                match object.get("type").and_then(Value::as_str) {
                    Some("message") | None if object.contains_key("role") => {
                        flush_responses_tool_uses(&mut messages, &mut pending_tool_uses);
                        flush_responses_tool_results(&mut messages, &mut pending_tool_results);
                        convert_responses_message_item(object, system, &mut messages)?
                    }
                    Some("function_call") => {
                        flush_responses_tool_results(&mut messages, &mut pending_tool_results);
                        pending_tool_uses.push(json!({
                            "type": "tool_use",
                            "id": required_responses_call_id(object)?,
                            "name": required_string(object, "name")?,
                            "input": parse_responses_function_arguments(object.get("arguments"))?,
                        }));
                    }
                    Some("custom_tool_call") => {
                        flush_responses_tool_results(&mut messages, &mut pending_tool_results);
                        let name = required_string(object, "name")?;
                        tool_context.record(name, ResponsesToolKind::Custom);
                        pending_tool_uses.push(json!({
                            "type": "tool_use",
                            "id": required_responses_call_id(object)?,
                            "name": name,
                            "input": responses_custom_tool_input_to_anthropic(object),
                        }));
                    }
                    Some("function_call_output") => {
                        flush_responses_tool_uses(&mut messages, &mut pending_tool_uses);
                        pending_tool_results.push(responses_tool_result_block(object, "output")?);
                    }
                    Some("custom_tool_call_output") => {
                        flush_responses_tool_uses(&mut messages, &mut pending_tool_uses);
                        pending_tool_results.push(responses_tool_result_block(object, "output")?);
                    }
                    Some("reasoning") => {}
                    Some("item_reference") => {
                        return Err(AppError::BadRequest(
                            "item_reference input is not supported".into(),
                        ))
                    }
                    Some(other) => {
                        return Err(AppError::BadRequest(format!(
                            "unsupported Responses input item: {other}"
                        )))
                    }
                    None => {
                        return Err(AppError::BadRequest(
                            "input item type or role is required".into(),
                        ))
                    }
                }
            }
            flush_responses_tool_uses(&mut messages, &mut pending_tool_uses);
            flush_responses_tool_results(&mut messages, &mut pending_tool_results);
            Ok(messages)
        }
        _ => Err(AppError::BadRequest(
            "input must be a string or array".into(),
        )),
    }
}

fn flush_responses_tool_uses(messages: &mut Vec<Value>, pending_tool_uses: &mut Vec<Value>) {
    if pending_tool_uses.is_empty() {
        return;
    }
    messages.push(json!({
        "role": "assistant",
        "content": std::mem::take(pending_tool_uses),
    }));
}

fn flush_responses_tool_results(messages: &mut Vec<Value>, pending_tool_results: &mut Vec<Value>) {
    if pending_tool_results.is_empty() {
        return;
    }
    messages.push(json!({
        "role": "user",
        "content": std::mem::take(pending_tool_results),
    }));
}

fn responses_tool_result_block(object: &Map<String, Value>, output_key: &str) -> AppResult<Value> {
    Ok(json!({
        "type": "tool_result",
        "tool_use_id": required_responses_call_id(object)?,
        "content": responses_output_text(object.get(output_key))?,
    }))
}

fn convert_responses_message_item(
    object: &Map<String, Value>,
    system: &mut Vec<String>,
    messages: &mut Vec<Value>,
) -> AppResult<()> {
    let role = required_string(object, "role")?;
    let content = object.get("content").unwrap_or(&Value::Null);
    match role {
        "system" | "developer" => {
            if let Some(text) = responses_content_text(content)? {
                system.push(text);
            }
        }
        "user" | "assistant" => {
            let blocks = responses_content_blocks_to_anthropic(content)?;
            let content = if matches!(content, Value::String(_)) && blocks.len() == 1 {
                blocks[0]
                    .get("text")
                    .and_then(Value::as_str)
                    .map(|text| Value::String(text.to_string()))
                    .unwrap_or_else(|| Value::Array(blocks))
            } else {
                Value::Array(blocks)
            };
            messages.push(json!({ "role": role, "content": content }));
        }
        other => {
            return Err(AppError::BadRequest(format!(
                "unsupported Responses message role: {other}"
            )))
        }
    }
    Ok(())
}

fn responses_content_blocks_to_anthropic(content: &Value) -> AppResult<Vec<Value>> {
    match content {
        Value::String(text) => Ok(vec![json!({ "type": "text", "text": text })]),
        Value::Array(blocks) => blocks
            .iter()
            .map(responses_content_block_to_anthropic)
            .collect(),
        Value::Null => Ok(Vec::new()),
        _ => Err(AppError::BadRequest(
            "message content must be string or array".into(),
        )),
    }
}

fn responses_content_block_to_anthropic(block: &Value) -> AppResult<Value> {
    let block_type = block
        .get("type")
        .and_then(Value::as_str)
        .ok_or_else(|| AppError::BadRequest("content block type is required".into()))?;
    match block_type {
        "input_text" | "output_text" | "text" => Ok(json!({
            "type": "text",
            "text": block.get("text").and_then(Value::as_str).unwrap_or(""),
        })),
        "input_image" => responses_image_block_to_anthropic(block),
        "input_file" => Err(AppError::BadRequest("input_file is not supported".into())),
        other => Err(AppError::BadRequest(format!(
            "unsupported Responses content block: {other}"
        ))),
    }
}

fn responses_image_block_to_anthropic(block: &Value) -> AppResult<Value> {
    let image_url = block
        .get("image_url")
        .and_then(Value::as_str)
        .or_else(|| {
            block
                .get("image_url")
                .and_then(|value| value.get("url"))
                .and_then(Value::as_str)
        })
        .ok_or_else(|| AppError::BadRequest("input_image image_url is required".into()))?;
    if let Some(rest) = image_url.strip_prefix("data:") {
        if let Some((media_type, data)) = rest.split_once(";base64,") {
            return Ok(json!({
                "type": "image",
                "source": {
                    "type": "base64",
                    "media_type": media_type,
                    "data": data,
                },
            }));
        }
    }
    Ok(json!({
        "type": "image",
        "source": {
            "type": "url",
            "url": image_url,
        },
    }))
}

fn responses_content_text(content: &Value) -> AppResult<Option<String>> {
    match content {
        Value::String(text) => Ok(nonblank(text)),
        Value::Array(blocks) => {
            let mut texts = Vec::new();
            for block in blocks {
                match block.get("type").and_then(Value::as_str) {
                    Some("input_text" | "output_text" | "text") => {
                        if let Some(text) = block.get("text").and_then(Value::as_str) {
                            texts.push(text.to_string());
                        }
                    }
                    Some("input_file") => {
                        return Err(AppError::BadRequest("input_file is not supported".into()))
                    }
                    Some(other) => {
                        return Err(AppError::BadRequest(format!(
                            "unsupported system content block: {other}"
                        )))
                    }
                    None => {}
                }
            }
            Ok(nonempty_join(texts, ""))
        }
        Value::Null => Ok(None),
        _ => Err(AppError::BadRequest(
            "message content must be string or array".into(),
        )),
    }
}

fn required_responses_call_id(object: &Map<String, Value>) -> AppResult<&str> {
    object
        .get("call_id")
        .or_else(|| object.get("id"))
        .and_then(Value::as_str)
        .ok_or_else(|| AppError::BadRequest("call_id is required".into()))
}

fn parse_responses_function_arguments(arguments: Option<&Value>) -> AppResult<Value> {
    match arguments {
        Some(Value::String(arguments)) => serde_json::from_str(arguments)
            .map_err(|_| AppError::BadRequest("function arguments must be valid JSON".into())),
        Some(Value::Object(_)) => Ok(arguments.cloned().unwrap()),
        Some(Value::Null) | None => Ok(json!({})),
        Some(_) => Err(AppError::BadRequest(
            "function arguments must be a JSON string or object".into(),
        )),
    }
}

fn responses_custom_tool_input_to_anthropic(object: &Map<String, Value>) -> Value {
    match object.get("input") {
        Some(Value::String(input)) => json!({ "input": input }),
        Some(Value::Null) | None => json!({ "input": "" }),
        Some(input) => json!({ "input": input.to_string() }),
    }
}

fn anthropic_custom_tool_input(input: &Value) -> String {
    match input {
        Value::Object(object) => object
            .get("input")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| input.to_string()),
        Value::String(input) => input.clone(),
        Value::Null => String::new(),
        _ => input.to_string(),
    }
}

fn anthropic_custom_tool_input_from_json(input: &str) -> String {
    serde_json::from_str::<Value>(input)
        .map(|value| anthropic_custom_tool_input(&value))
        .unwrap_or_else(|_| input.to_string())
}

fn responses_output_text(output: Option<&Value>) -> AppResult<String> {
    match output {
        Some(Value::String(text)) => Ok(text.clone()),
        Some(Value::Array(blocks)) => {
            let mut texts = Vec::new();
            for block in blocks {
                if matches!(
                    block.get("type").and_then(Value::as_str),
                    Some("output_text") | Some("input_text") | Some("text")
                ) {
                    if let Some(text) = block.get("text").and_then(Value::as_str) {
                        texts.push(text.to_string());
                    }
                } else {
                    return Err(AppError::BadRequest(
                        "function_call_output supports text content only".into(),
                    ));
                }
            }
            Ok(texts.join(""))
        }
        Some(Value::Null) | None => Ok(String::new()),
        Some(_) => Err(AppError::BadRequest(
            "function_call_output output must be string or text blocks".into(),
        )),
    }
}

fn convert_responses_tools_to_anthropic(tools: &Value) -> AppResult<(Value, ToolContext)> {
    let tools = tools
        .as_array()
        .ok_or_else(|| AppError::BadRequest("tools must be an array".into()))?;
    let mut converted = Vec::new();
    let mut context = ToolContext::default();
    for tool in tools {
        let object = tool
            .as_object()
            .ok_or_else(|| AppError::BadRequest("tool must be an object".into()))?;
        let tool_type = object.get("type").and_then(Value::as_str).unwrap_or("");
        match tool_type {
            "function" | "" => {
                let function = object.get("function").and_then(Value::as_object);
                let source = function.unwrap_or(object);
                let name = required_string(source, "name")?;
                validate_tool_name(name)?;
                context.record(name, ResponsesToolKind::Function);
                let mut converted_tool = Map::new();
                converted_tool.insert("name".into(), Value::String(name.to_string()));
                if let Some(description) = source.get("description") {
                    converted_tool.insert("description".into(), description.clone());
                }
                converted_tool.insert(
                    "input_schema".into(),
                    source
                        .get("parameters")
                        .cloned()
                        .unwrap_or_else(|| json!({ "type": "object", "properties": {} })),
                );
                converted.push(Value::Object(converted_tool));
            }
            "freeform" | "custom" => {
                let name = required_string(object, "name")?;
                validate_tool_name(name)?;
                context.record(name, ResponsesToolKind::Custom);
                let description = object
                    .get("description")
                    .and_then(Value::as_str)
                    .map(ToOwned::to_owned)
                    .unwrap_or_else(|| {
                        format!("Codex {tool_type} tool '{name}' adapted to a function tool. Pass the body as the `input` argument.")
                    });
                converted.push(json!({
                    "name": name,
                    "description": description,
                    "input_schema": {
                        "type": "object",
                        "properties": {
                            "input": {
                                "type": "string",
                                "description": "Raw freeform tool body."
                            }
                        },
                        "required": ["input"]
                    }
                }));
            }
            "code_interpreter"
            | "computer_use_preview"
            | "file_search"
            | "image_generation"
            | "local_shell"
            | "namespace"
            | "tool_search"
            | "web_search"
            | "web_search_preview" => {}
            other => {
                return Err(AppError::BadRequest(format!(
                    "unsupported Responses tool type for Anthropic adapter: {other}"
                )));
            }
        }
    }
    Ok((Value::Array(converted), context))
}

fn convert_responses_tool_choice_to_anthropic(
    tool_choice: &Value,
    thinking_enabled: bool,
) -> AppResult<Value> {
    match tool_choice {
        Value::String(choice) => match choice.as_str() {
            "auto" | "none" => Ok(json!({ "type": choice })),
            "required" => {
                reject_forced_tool_choice_with_thinking(thinking_enabled)?;
                Ok(json!({ "type": "any" }))
            }
            other => Err(AppError::BadRequest(format!(
                "unsupported tool_choice: {other}"
            ))),
        },
        Value::Object(object) => match object.get("type").and_then(Value::as_str) {
            Some("function") => {
                reject_forced_tool_choice_with_thinking(thinking_enabled)?;
                let name = object
                    .get("name")
                    .and_then(Value::as_str)
                    .or_else(|| {
                        object
                            .get("function")
                            .and_then(|function| function.get("name"))
                            .and_then(Value::as_str)
                    })
                    .ok_or_else(|| AppError::BadRequest("tool_choice name is required".into()))?;
                Ok(json!({ "type": "tool", "name": name }))
            }
            Some("tool") => {
                reject_forced_tool_choice_with_thinking(thinking_enabled)?;
                Ok(tool_choice.clone())
            }
            Some("any") => {
                reject_forced_tool_choice_with_thinking(thinking_enabled)?;
                Ok(tool_choice.clone())
            }
            Some("auto") | Some("none") => Ok(tool_choice.clone()),
            Some(other) => Err(AppError::BadRequest(format!(
                "unsupported tool_choice type: {other}"
            ))),
            None => Err(AppError::BadRequest("tool_choice type is required".into())),
        },
        _ => Err(AppError::BadRequest("unsupported tool_choice".into())),
    }
}

fn reject_forced_tool_choice_with_thinking(thinking_enabled: bool) -> AppResult<()> {
    if thinking_enabled {
        return Err(AppError::BadRequest(
            "forced tool_choice is not supported when Anthropic thinking is enabled".into(),
        ));
    }
    Ok(())
}

fn flush_responses_output_message(
    output: &mut Vec<Value>,
    message_id: &str,
    content: &mut Vec<Value>,
) {
    if content.is_empty() {
        return;
    }
    output.push(json!({
        "id": format!("msg_{}", output.len()),
        "type": "message",
        "status": "completed",
        "role": "assistant",
        "content": std::mem::take(content),
    }));
    let _ = message_id;
}

fn responses_response_id_for_anthropic_message(message_id: &str) -> String {
    if message_id.starts_with("resp_") {
        message_id.to_string()
    } else {
        format!("resp_{message_id}")
    }
}

fn responses_usage_for_anthropic_message(usage: Option<&Value>) -> Value {
    let usage = usage.unwrap_or(&Value::Null);
    let input_tokens = usage
        .get("input_tokens")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let output_tokens = usage
        .get("output_tokens")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let cached_tokens = usage
        .get("cache_read_input_tokens")
        .or_else(|| usage.get("cache_creation_input_tokens"))
        .and_then(Value::as_u64)
        .unwrap_or(0);
    json!({
        "input_tokens": input_tokens,
        "output_tokens": output_tokens,
        "total_tokens": input_tokens + output_tokens,
        "input_tokens_details": {
            "cached_tokens": cached_tokens,
        },
        "output_tokens_details": {
            "reasoning_tokens": 0,
        },
    })
}

fn reject_unsupported_top_level(object: &Map<String, Value>) -> AppResult<()> {
    for key in ["output_config", "output_format"] {
        if object.contains_key(key) {
            return Err(AppError::BadRequest(format!("{key} is not supported yet")));
        }
    }
    if object.contains_key("context_management") {
        return Err(AppError::BadRequest(
            "context_management is not supported yet".into(),
        ));
    }
    Ok(())
}

fn required_string<'a>(object: &'a Map<String, Value>, key: &str) -> AppResult<&'a str> {
    object
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| AppError::BadRequest(format!("missing {key}")))
}

fn copy_field(input: &Map<String, Value>, output: &mut Map<String, Value>, from: &str, to: &str) {
    if let Some(value) = input.get(from) {
        output.insert(to.into(), value.clone());
    }
}

fn string_like(value: &Value) -> Option<String> {
    match value {
        Value::String(value) => Some(value.clone()),
        Value::Number(value) => Some(value.to_string()),
        _ => None,
    }
}

fn anthropic_system_to_instructions(system: Option<&Value>) -> AppResult<Option<String>> {
    let Some(system) = system else {
        return Ok(None);
    };
    match system {
        Value::String(text) => Ok(nonblank(text)),
        Value::Array(blocks) => {
            let texts = blocks
                .iter()
                .filter_map(|block| {
                    if block.get("type").and_then(Value::as_str) == Some("text") {
                        block.get("text").and_then(Value::as_str)
                    } else {
                        None
                    }
                })
                .filter(|text| !text.trim().is_empty())
                .map(ToOwned::to_owned)
                .collect::<Vec<_>>();
            Ok(if texts.is_empty() {
                None
            } else {
                Some(texts.join("\n\n"))
            })
        }
        _ => Err(AppError::BadRequest(
            "system must be a string or text blocks".into(),
        )),
    }
}

fn nonblank(text: &str) -> Option<String> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn anthropic_thinking_to_reasoning(thinking: Option<&Value>) -> AppResult<Option<Value>> {
    let Some(thinking) = thinking else {
        return Ok(None);
    };
    let object = thinking
        .as_object()
        .ok_or_else(|| AppError::BadRequest("thinking must be an object".into()))?;
    if object.get("type").and_then(Value::as_str) == Some("disabled") {
        return Ok(None);
    }
    let effort = object
        .get("effort")
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
        .unwrap_or_else(
            || match object.get("budget_tokens").and_then(Value::as_u64) {
                Some(tokens) if tokens >= 12_000 => "high".into(),
                Some(tokens) if tokens <= 2_000 => "low".into(),
                _ => "medium".into(),
            },
        );
    Ok(Some(json!({ "effort": effort, "summary": "auto" })))
}

fn convert_anthropic_messages(messages: &[Value]) -> AppResult<Vec<Value>> {
    let mut input = Vec::new();
    for message in messages {
        let object = message
            .as_object()
            .ok_or_else(|| AppError::BadRequest("message must be an object".into()))?;
        let role = required_string(object, "role")?;
        match role {
            "user" => convert_anthropic_user_message(object.get("content"), &mut input)?,
            "assistant" => convert_anthropic_assistant_message(object.get("content"), &mut input)?,
            _ => {
                return Err(AppError::BadRequest(format!(
                    "unsupported message role: {role}"
                )))
            }
        }
    }
    Ok(input)
}

fn convert_anthropic_user_message(
    content: Option<&Value>,
    input: &mut Vec<Value>,
) -> AppResult<()> {
    let mut message_content = Vec::new();
    for block in anthropic_content_blocks(content)? {
        let block_type = block.get("type").and_then(Value::as_str).unwrap_or("text");
        match block_type {
            "text" => {
                if let Some(text) = block.get("text").and_then(Value::as_str) {
                    message_content.push(json!({ "type": "input_text", "text": text }));
                }
            }
            "image" => {
                message_content.push(json!({
                    "type": "input_image",
                    "image_url": anthropic_image_url(&block)?,
                }));
            }
            "tool_result" => {
                flush_message(input, "user", &mut message_content);
                input.push(json!({
                    "type": "function_call_output",
                    "call_id": required_block_string(&block, "tool_use_id")?,
                    "output": anthropic_tool_result_output(&block)?,
                }));
            }
            _ => {
                return Err(AppError::BadRequest(format!(
                    "unsupported user content block: {block_type}"
                )))
            }
        }
    }
    flush_message(input, "user", &mut message_content);
    Ok(())
}

fn convert_anthropic_assistant_message(
    content: Option<&Value>,
    input: &mut Vec<Value>,
) -> AppResult<()> {
    let mut message_content = Vec::new();
    for block in anthropic_content_blocks(content)? {
        let block_type = block.get("type").and_then(Value::as_str).unwrap_or("text");
        match block_type {
            "text" => {
                if let Some(text) = block.get("text").and_then(Value::as_str) {
                    message_content.push(json!({ "type": "output_text", "text": text }));
                }
            }
            "tool_use" => {
                flush_message(input, "assistant", &mut message_content);
                let arguments = block.get("input").cloned().unwrap_or_else(|| json!({}));
                input.push(json!({
                    "type": "function_call",
                    "call_id": required_block_string(&block, "id")?,
                    "name": required_block_string(&block, "name")?,
                    "arguments": arguments.to_string(),
                }));
            }
            "thinking" => {}
            _ => {
                return Err(AppError::BadRequest(format!(
                    "unsupported assistant content block: {block_type}"
                )))
            }
        }
    }
    flush_message(input, "assistant", &mut message_content);
    Ok(())
}

fn anthropic_content_blocks(content: Option<&Value>) -> AppResult<Vec<Value>> {
    match content {
        Some(Value::String(text)) => Ok(vec![json!({ "type": "text", "text": text })]),
        Some(Value::Array(blocks)) => Ok(blocks.clone()),
        Some(_) => Err(AppError::BadRequest(
            "message content must be string or array".into(),
        )),
        None => Ok(Vec::new()),
    }
}

fn anthropic_image_url(block: &Value) -> AppResult<String> {
    let source = block
        .get("source")
        .and_then(Value::as_object)
        .ok_or_else(|| AppError::BadRequest("image source must be an object".into()))?;
    match source.get("type").and_then(Value::as_str) {
        Some("base64") => {
            let media_type = source
                .get("media_type")
                .and_then(Value::as_str)
                .ok_or_else(|| AppError::BadRequest("image media_type is required".into()))?;
            let data = source
                .get("data")
                .and_then(Value::as_str)
                .ok_or_else(|| AppError::BadRequest("image data is required".into()))?;
            Ok(format!("data:{media_type};base64,{data}"))
        }
        Some("url") => source
            .get("url")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned)
            .ok_or_else(|| AppError::BadRequest("image url is required".into())),
        Some(other) => Err(AppError::BadRequest(format!(
            "unsupported image source type: {other}"
        ))),
        None => Err(AppError::BadRequest("image source type is required".into())),
    }
}

fn anthropic_tool_result_output(block: &Value) -> AppResult<String> {
    match block.get("content") {
        Some(Value::String(text)) => Ok(text.clone()),
        Some(Value::Array(blocks)) => {
            let mut texts = Vec::new();
            for block in blocks {
                if block.get("type").and_then(Value::as_str) != Some("text") {
                    return Err(AppError::BadRequest(
                        "tool_result supports text content only".into(),
                    ));
                }
                if let Some(text) = block.get("text").and_then(Value::as_str) {
                    texts.push(text.to_string());
                }
            }
            Ok(texts.join(""))
        }
        Some(_) => Err(AppError::BadRequest(
            "tool_result content must be string or text blocks".into(),
        )),
        None => Ok(String::new()),
    }
}

fn required_block_string<'a>(block: &'a Value, key: &str) -> AppResult<&'a str> {
    block
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| AppError::BadRequest(format!("missing {key}")))
}

fn flush_message(input: &mut Vec<Value>, role: &str, content: &mut Vec<Value>) {
    if content.is_empty() {
        return;
    }
    input.push(json!({
        "type": "message",
        "role": role,
        "content": std::mem::take(content),
    }));
}

fn convert_anthropic_tools(tools: &Value) -> AppResult<Value> {
    let tools = tools
        .as_array()
        .ok_or_else(|| AppError::BadRequest("tools must be an array".into()))?;
    let mut converted = Vec::new();
    for tool in tools {
        let object = tool
            .as_object()
            .ok_or_else(|| AppError::BadRequest("tool must be an object".into()))?;
        if let Some(tool_type) = object.get("type").and_then(Value::as_str) {
            if tool_type != "custom" && tool_type != "function" {
                return Err(AppError::BadRequest(format!(
                    "unsupported Anthropic hosted tool: {tool_type}"
                )));
            }
        }
        let name = required_string(object, "name")?;
        validate_tool_name(name)?;
        let mut converted_tool = Map::new();
        converted_tool.insert("type".into(), Value::String("function".into()));
        converted_tool.insert("name".into(), Value::String(name.to_string()));
        if let Some(description) = object.get("description") {
            converted_tool.insert("description".into(), description.clone());
        }
        converted_tool.insert(
            "parameters".into(),
            object
                .get("input_schema")
                .cloned()
                .unwrap_or_else(|| json!({ "type": "object", "properties": {} })),
        );
        converted.push(Value::Object(converted_tool));
    }
    Ok(Value::Array(converted))
}

fn convert_chat_tools(tools: &Value) -> AppResult<Value> {
    let tools = tools
        .as_array()
        .ok_or_else(|| AppError::BadRequest("tools must be an array".into()))?;
    let mut converted = Vec::new();
    for tool in tools {
        let object = tool
            .as_object()
            .ok_or_else(|| AppError::BadRequest("tool must be an object".into()))?;
        if object.get("type").and_then(Value::as_str) != Some("function") {
            return Err(AppError::BadRequest(
                "only function tools are supported".into(),
            ));
        }
        let function = object
            .get("function")
            .and_then(Value::as_object)
            .ok_or_else(|| AppError::BadRequest("function tool is missing function".into()))?;
        let name = required_string(function, "name")?;
        validate_tool_name(name)?;
        let mut converted_tool = Map::new();
        converted_tool.insert("type".into(), Value::String("function".into()));
        converted_tool.insert("name".into(), Value::String(name.to_string()));
        if let Some(description) = function.get("description") {
            converted_tool.insert("description".into(), description.clone());
        }
        converted_tool.insert(
            "parameters".into(),
            function
                .get("parameters")
                .cloned()
                .unwrap_or_else(|| json!({ "type": "object", "properties": {} })),
        );
        converted.push(Value::Object(converted_tool));
    }
    Ok(Value::Array(converted))
}

fn validate_tool_name(name: &str) -> AppResult<()> {
    if name.len() > MAX_TOOL_NAME_LEN {
        return Err(AppError::BadRequest(
            "tool name exceeds 64 characters".into(),
        ));
    }
    Ok(())
}

fn convert_anthropic_tool_choice(tool_choice: &Value) -> AppResult<Value> {
    let object = tool_choice
        .as_object()
        .ok_or_else(|| AppError::BadRequest("tool_choice must be an object".into()))?;
    match object.get("type").and_then(Value::as_str) {
        Some("auto") => Ok(Value::String("auto".into())),
        Some("any") => Ok(Value::String("required".into())),
        Some("none") => Ok(Value::String("none".into())),
        Some("tool") => Ok(json!({
            "type": "function",
            "name": required_string(object, "name")?,
        })),
        Some(other) => Err(AppError::BadRequest(format!(
            "unsupported tool_choice type: {other}"
        ))),
        None => Err(AppError::BadRequest("tool_choice type is required".into())),
    }
}

fn convert_chat_tool_choice(tool_choice: &Value) -> AppResult<Value> {
    match tool_choice {
        Value::String(choice) if matches!(choice.as_str(), "auto" | "none" | "required") => {
            Ok(Value::String(choice.clone()))
        }
        Value::Object(object) if object.get("type").and_then(Value::as_str) == Some("function") => {
            let function = object
                .get("function")
                .and_then(Value::as_object)
                .ok_or_else(|| AppError::BadRequest("tool_choice function is required".into()))?;
            Ok(json!({
                "type": "function",
                "name": required_string(function, "name")?,
            }))
        }
        _ => Err(AppError::BadRequest("unsupported tool_choice".into())),
    }
}

fn convert_chat_messages(messages: &[Value]) -> AppResult<(Option<String>, Vec<Value>)> {
    let mut instructions = Vec::new();
    let mut input = Vec::new();
    for message in messages {
        let object = message
            .as_object()
            .ok_or_else(|| AppError::BadRequest("message must be an object".into()))?;
        let role = required_string(object, "role")?;
        match role {
            "system" | "developer" => {
                if let Some(text) = chat_content_text(object.get("content"))? {
                    instructions.push(text);
                }
            }
            "user" => {
                let content = chat_user_content_blocks(object.get("content"))?;
                if !content.is_empty() {
                    input.push(json!({ "type": "message", "role": "user", "content": content }));
                }
            }
            "assistant" => {
                if let Some(text) = chat_content_text(object.get("content"))? {
                    input.push(json!({
                        "type": "message",
                        "role": "assistant",
                        "content": [{ "type": "output_text", "text": text }],
                    }));
                }
                if let Some(tool_calls) = object.get("tool_calls").and_then(Value::as_array) {
                    for tool_call in tool_calls {
                        let function = tool_call
                            .get("function")
                            .and_then(Value::as_object)
                            .ok_or_else(|| {
                                AppError::BadRequest("tool_call function is required".into())
                            })?;
                        input.push(json!({
                            "type": "function_call",
                            "call_id": tool_call.get("id").and_then(Value::as_str).unwrap_or(""),
                            "name": required_string(function, "name")?,
                            "arguments": function
                                .get("arguments")
                                .and_then(Value::as_str)
                                .unwrap_or("{}"),
                        }));
                    }
                }
            }
            "tool" => {
                input.push(json!({
                    "type": "function_call_output",
                    "call_id": object
                        .get("tool_call_id")
                        .and_then(Value::as_str)
                        .ok_or_else(|| AppError::BadRequest("tool_call_id is required".into()))?,
                    "output": chat_content_text(object.get("content"))?.unwrap_or_default(),
                }));
            }
            _ => {
                return Err(AppError::BadRequest(format!(
                    "unsupported message role: {role}"
                )))
            }
        }
    }
    Ok((nonempty_join(instructions, "\n\n"), input))
}

fn chat_user_content_blocks(content: Option<&Value>) -> AppResult<Vec<Value>> {
    match content {
        Some(Value::String(text)) => Ok(vec![json!({ "type": "input_text", "text": text })]),
        Some(Value::Array(blocks)) => {
            let mut converted = Vec::new();
            for block in blocks {
                let block_type = block.get("type").and_then(Value::as_str);
                match block_type {
                    Some("text") => {
                        if let Some(text) = block.get("text").and_then(Value::as_str) {
                            converted.push(json!({ "type": "input_text", "text": text }));
                        }
                    }
                    Some("image_url") => {
                        let image_url = block
                            .get("image_url")
                            .and_then(|value| value.get("url"))
                            .and_then(Value::as_str)
                            .ok_or_else(|| {
                                AppError::BadRequest("image_url.url is required".into())
                            })?;
                        converted.push(json!({ "type": "input_image", "image_url": image_url }));
                    }
                    Some(other) => {
                        return Err(AppError::BadRequest(format!(
                            "unsupported chat content block: {other}"
                        )))
                    }
                    None => {
                        return Err(AppError::BadRequest(
                            "content block type is required".into(),
                        ))
                    }
                }
            }
            Ok(converted)
        }
        Some(Value::Null) | None => Ok(Vec::new()),
        Some(_) => Err(AppError::BadRequest(
            "message content must be string or array".into(),
        )),
    }
}

fn chat_content_text(content: Option<&Value>) -> AppResult<Option<String>> {
    match content {
        Some(Value::String(text)) => Ok(Some(text.clone())),
        Some(Value::Array(blocks)) => {
            let mut texts = Vec::new();
            for block in blocks {
                if block.get("type").and_then(Value::as_str) != Some("text") {
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
            "message content must be string or array".into(),
        )),
    }
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

fn push_anthropic_content_for_response_item(item: &Value, content: &mut Vec<Value>) {
    match item.get("type").and_then(Value::as_str) {
        Some("message") => {
            if let Some(blocks) = item.get("content").and_then(Value::as_array) {
                for block in blocks {
                    if matches!(
                        block.get("type").and_then(Value::as_str),
                        Some("output_text") | Some("text")
                    ) {
                        if let Some(text) = block.get("text").and_then(Value::as_str) {
                            content.push(json!({ "type": "text", "text": text }));
                        }
                    }
                }
            }
        }
        Some("function_call") => {
            let arguments = item
                .get("arguments")
                .and_then(Value::as_str)
                .and_then(|value| serde_json::from_str(value).ok())
                .unwrap_or_else(|| {
                    tracing::warn!("Responses function_call arguments were not valid JSON");
                    json!({})
                });
            content.push(json!({
                "type": "tool_use",
                "id": item
                    .get("call_id")
                    .or_else(|| item.get("id"))
                    .and_then(Value::as_str)
                    .unwrap_or("call_responses_adapter"),
                "name": item.get("name").and_then(Value::as_str).unwrap_or("tool"),
                "input": arguments,
            }));
        }
        Some("custom_tool_call") => {
            content.push(json!({
                "type": "tool_use",
                "id": item
                    .get("call_id")
                    .or_else(|| item.get("id"))
                    .and_then(Value::as_str)
                    .unwrap_or("call_responses_adapter"),
                "name": item.get("name").and_then(Value::as_str).unwrap_or("tool"),
                "input": {
                    "input": item.get("input").and_then(Value::as_str).unwrap_or(""),
                },
            }));
        }
        _ => {}
    }
}

fn anthropic_message_id(response: &Value) -> String {
    response
        .get("id")
        .and_then(Value::as_str)
        .map(|id| {
            if id.starts_with("msg_") {
                id.to_string()
            } else {
                format!("msg_{id}")
            }
        })
        .unwrap_or_else(|| DEFAULT_ANTHROPIC_MESSAGE_ID.into())
}

fn anthropic_usage(response: &Value) -> Value {
    let usage = response.get("usage").unwrap_or(&Value::Null);
    json!({
        "input_tokens": usage
            .get("input_tokens")
            .and_then(Value::as_u64)
            .unwrap_or(0),
        "output_tokens": usage
            .get("output_tokens")
            .and_then(Value::as_u64)
            .unwrap_or(0),
        "cache_read_input_tokens": usage
            .get("input_tokens_details")
            .and_then(|details| details.get("cached_tokens"))
            .and_then(Value::as_u64)
            .unwrap_or(0),
    })
}

#[derive(Default)]
struct AnthropicSseTranslator {
    output: String,
    response_id: String,
    message_id: String,
    model: String,
    input_tokens: u64,
    output_tokens: u64,
    stop_reason: Option<String>,
    blocks: HashMap<usize, AnthropicSseBlock>,
    model_override: Option<String>,
    tool_context: ToolContext,
}

#[derive(Clone, Debug)]
struct AnthropicSseBlock {
    item: Value,
    text: String,
    arguments: String,
}

impl AnthropicSseTranslator {
    fn with_model(requested_model: &str) -> Self {
        Self {
            model_override: Some(requested_model.to_string()),
            ..Default::default()
        }
    }

    fn with_model_and_context(requested_model: &str, tool_context: ToolContext) -> Self {
        Self {
            model_override: Some(requested_model.to_string()),
            tool_context,
            ..Default::default()
        }
    }

    fn process_block(&mut self, block: &str) -> AppResult<()> {
        let event = event_name(block);
        let data = event_data_json(block);
        let event = event.as_deref().or_else(|| {
            data.as_ref()
                .and_then(|data| data.get("type"))
                .and_then(Value::as_str)
        });
        match event {
            Some("message_start") => self.emit_response_created(data.as_ref()),
            Some("content_block_start") => self.emit_output_item_added(data.as_ref())?,
            Some("content_block_delta") => self.emit_delta(data.as_ref()),
            Some("content_block_stop") => self.emit_output_item_done(data.as_ref()),
            Some("message_delta") => self.capture_message_delta(data.as_ref()),
            Some("message_stop") => self.emit_response_completed(),
            Some("error") => self.emit_responses_error(data.as_ref()),
            _ => {}
        }
        Ok(())
    }

    fn emit_response_created(&mut self, data: Option<&Value>) {
        let message = data
            .and_then(|data| data.get("message"))
            .unwrap_or(&Value::Null);
        self.message_id = message
            .get("id")
            .and_then(Value::as_str)
            .unwrap_or(DEFAULT_ANTHROPIC_MESSAGE_ID)
            .to_string();
        self.response_id = responses_response_id_for_anthropic_message(&self.message_id);
        self.model = self.model_override.clone().unwrap_or_else(|| {
            message
                .get("model")
                .and_then(Value::as_str)
                .unwrap_or("anthropic")
                .to_string()
        });
        if self.model.is_empty() {
            self.model = message
                .get("model")
                .and_then(Value::as_str)
                .unwrap_or("anthropic")
                .to_string();
        }
        if let Some(usage) = message.get("usage") {
            self.input_tokens = usage
                .get("input_tokens")
                .and_then(Value::as_u64)
                .unwrap_or(self.input_tokens);
            self.output_tokens = usage
                .get("output_tokens")
                .and_then(Value::as_u64)
                .unwrap_or(self.output_tokens);
        }
        self.push_sse(
            "response.created",
            json!({
                "type": "response.created",
                "response": self.response_envelope("in_progress", Vec::new()),
            }),
        );
    }

    fn emit_output_item_added(&mut self, data: Option<&Value>) -> AppResult<()> {
        self.ensure_started();
        let Some(data) = data else {
            return Ok(());
        };
        let index = data.get("index").and_then(Value::as_u64).unwrap_or(0) as usize;
        let block = data.get("content_block").unwrap_or(&Value::Null);
        let item = match block.get("type").and_then(Value::as_str) {
            Some("text") => json!({
                "id": format!("msg_{index}"),
                "type": "message",
                "status": "in_progress",
                "role": "assistant",
                "content": [],
            }),
            Some("tool_use") => {
                let name = block.get("name").and_then(Value::as_str).unwrap_or("tool");
                let call_id = block
                    .get("id")
                    .and_then(Value::as_str)
                    .unwrap_or("call_anthropic_adapter");
                if self.tool_context.is_custom(name) {
                    json!({
                        "id": format!("ctc_{index}"),
                        "type": "custom_tool_call",
                        "status": "in_progress",
                        "call_id": call_id,
                        "name": name,
                        "input": "",
                    })
                } else {
                    json!({
                        "id": format!("fc_{index}"),
                        "type": "function_call",
                        "status": "in_progress",
                        "call_id": call_id,
                        "name": name,
                        "arguments": "",
                    })
                }
            }
            Some("thinking" | "redacted_thinking") => return Ok(()),
            Some(other) => {
                return Err(AppError::BadRequest(format!(
                    "unsupported Anthropic SSE content block: {other}"
                )))
            }
            None => return Ok(()),
        };
        self.blocks.insert(
            index,
            AnthropicSseBlock {
                item: item.clone(),
                text: String::new(),
                arguments: String::new(),
            },
        );
        self.push_sse(
            "response.output_item.added",
            json!({
                "type": "response.output_item.added",
                "output_index": index,
                "item": item,
            }),
        );
        Ok(())
    }

    fn emit_delta(&mut self, data: Option<&Value>) {
        self.ensure_started();
        let Some(data) = data else {
            return;
        };
        let index = data.get("index").and_then(Value::as_u64).unwrap_or(0) as usize;
        let delta = data.get("delta").unwrap_or(&Value::Null);
        match delta.get("type").and_then(Value::as_str) {
            Some("text_delta") => {
                let text = delta.get("text").and_then(Value::as_str).unwrap_or("");
                if let Some(block) = self.blocks.get_mut(&index) {
                    block.text.push_str(text);
                }
                self.push_sse(
                    "response.output_text.delta",
                    json!({
                        "type": "response.output_text.delta",
                        "output_index": index,
                        "content_index": 0,
                        "delta": text,
                    }),
                );
            }
            Some("input_json_delta") => {
                let partial_json = delta
                    .get("partial_json")
                    .and_then(Value::as_str)
                    .unwrap_or("");
                let is_custom = self
                    .blocks
                    .get(&index)
                    .and_then(|block| block.item.get("type").and_then(Value::as_str))
                    == Some("custom_tool_call");
                if let Some(block) = self.blocks.get_mut(&index) {
                    block.arguments.push_str(partial_json);
                }
                if is_custom {
                    self.push_sse(
                        "response.custom_tool_call_input.delta",
                        json!({
                            "type": "response.custom_tool_call_input.delta",
                            "output_index": index,
                            "delta": partial_json,
                        }),
                    );
                } else {
                    self.push_sse(
                        "response.function_call_arguments.delta",
                        json!({
                            "type": "response.function_call_arguments.delta",
                            "output_index": index,
                            "delta": partial_json,
                        }),
                    );
                }
            }
            _ => {}
        }
    }

    fn emit_output_item_done(&mut self, data: Option<&Value>) {
        self.ensure_started();
        let index = data
            .and_then(|data| data.get("index"))
            .and_then(Value::as_u64)
            .unwrap_or(0) as usize;
        let Some(mut block) = self.blocks.remove(&index) else {
            return;
        };
        if block.item.get("type").and_then(Value::as_str) == Some("message") {
            block.item["status"] = Value::String("completed".into());
            block.item["content"] =
                json!([{ "type": "output_text", "text": std::mem::take(&mut block.text) }]);
        } else if block.item.get("type").and_then(Value::as_str) == Some("function_call") {
            block.item["status"] = Value::String("completed".into());
            block.item["arguments"] = Value::String(std::mem::take(&mut block.arguments));
        } else if block.item.get("type").and_then(Value::as_str) == Some("custom_tool_call") {
            block.item["status"] = Value::String("completed".into());
            block.item["input"] = Value::String(anthropic_custom_tool_input_from_json(
                &std::mem::take(&mut block.arguments),
            ));
        }
        self.blocks.insert(index, block.clone());
        self.push_sse(
            "response.output_item.done",
            json!({
                "type": "response.output_item.done",
                "output_index": index,
                "item": block.item,
            }),
        );
    }

    fn capture_message_delta(&mut self, data: Option<&Value>) {
        let Some(data) = data else {
            return;
        };
        if let Some(stop_reason) = data
            .get("delta")
            .and_then(|delta| delta.get("stop_reason"))
            .and_then(Value::as_str)
        {
            self.stop_reason = Some(stop_reason.to_string());
        }
        if let Some(usage) = data.get("usage") {
            self.output_tokens = usage
                .get("output_tokens")
                .and_then(Value::as_u64)
                .unwrap_or(self.output_tokens);
        }
    }

    fn emit_response_completed(&mut self) {
        self.ensure_started();
        let status = if self.stop_reason.as_deref() == Some("max_tokens") {
            "incomplete"
        } else {
            "completed"
        };
        let mut response = self.response_envelope(status, self.output_items());
        if status == "incomplete" {
            response["incomplete_details"] = json!({ "reason": "max_output_tokens" });
        }
        self.push_sse(
            "response.completed",
            json!({
                "type": "response.completed",
                "response": response,
            }),
        );
    }

    fn emit_responses_error(&mut self, data: Option<&Value>) {
        let message = data
            .and_then(|data| {
                data.get("error")
                    .and_then(|error| error.get("message"))
                    .or_else(|| data.get("message"))
            })
            .and_then(Value::as_str)
            .unwrap_or("Anthropic Messages stream returned failure");
        self.push_sse(
            "error",
            json!({
                "type": "error",
                "error": { "type": "api_error", "message": message },
            }),
        );
    }

    fn ensure_started(&mut self) {
        if self.response_id.is_empty() {
            self.message_id = DEFAULT_ANTHROPIC_MESSAGE_ID.to_string();
            self.response_id = DEFAULT_RESPONSE_ID.to_string();
            self.model = self
                .model_override
                .clone()
                .unwrap_or_else(|| "anthropic".to_string());
            self.push_sse(
                "response.created",
                json!({
                    "type": "response.created",
                    "response": self.response_envelope("in_progress", Vec::new()),
                }),
            );
        }
    }

    fn output_items(&self) -> Vec<Value> {
        let mut entries = self
            .blocks
            .iter()
            .map(|(index, block)| (*index, block.item.clone()))
            .collect::<Vec<_>>();
        entries.sort_by_key(|(index, _)| *index);
        entries.into_iter().map(|(_, item)| item).collect()
    }

    fn response_envelope(&self, status: &str, output: Vec<Value>) -> Value {
        json!({
            "id": self.response_id,
            "object": "response",
            "created_at": 0,
            "model": self.model,
            "status": status,
            "output": output,
            "usage": {
                "input_tokens": self.input_tokens,
                "output_tokens": self.output_tokens,
                "total_tokens": self.input_tokens + self.output_tokens,
                "input_tokens_details": { "cached_tokens": 0 },
                "output_tokens_details": { "reasoning_tokens": 0 },
            },
        })
    }

    fn push_sse(&mut self, event: &str, data: Value) {
        self.output.push_str("event: ");
        self.output.push_str(event);
        self.output.push('\n');
        self.output.push_str("data: ");
        self.output.push_str(&data.to_string());
        self.output.push_str("\n\n");
    }
}

struct SseTranslator<'a> {
    requested_model: &'a str,
    output: String,
    started: bool,
    next_index: usize,
    item_indexes: HashMap<String, usize>,
    output_items: Vec<Value>,
}

impl<'a> SseTranslator<'a> {
    fn new(requested_model: &'a str) -> Self {
        Self {
            requested_model,
            output: String::new(),
            started: false,
            next_index: 0,
            item_indexes: HashMap::new(),
            output_items: Vec::new(),
        }
    }

    fn process_block(&mut self, block: &str) -> AppResult<()> {
        let Some(event) = event_name(block) else {
            return Ok(());
        };
        let data = event_data_json(block);
        match event.as_str() {
            "response.created" => self.emit_message_start(data.as_ref()),
            "response.output_item.added" => self.emit_content_block_start(data.as_ref())?,
            "response.output_text.delta" => self.emit_text_delta(data.as_ref())?,
            "response.function_call_arguments.delta" => self.emit_arguments_delta(data.as_ref())?,
            "response.custom_tool_call_input.delta" => {
                self.emit_custom_tool_input_delta(data.as_ref())?
            }
            "response.output_item.done" => self.emit_content_block_stop(data.as_ref()),
            "response.completed" => self.emit_message_done(data.as_ref())?,
            "response.failed" | "error" => self.emit_error(data.as_ref()),
            _ => {}
        }
        Ok(())
    }

    fn ensure_started(&mut self, response: Option<&Value>) {
        if !self.started {
            self.emit_message_start(response);
        }
    }

    fn emit_message_start(&mut self, response: Option<&Value>) {
        if self.started {
            return;
        }
        let response = response
            .and_then(|data| data.get("response"))
            .or(response)
            .unwrap_or(&Value::Null);
        let message = json!({
            "id": anthropic_message_id(response),
            "type": "message",
            "role": "assistant",
            "model": self.requested_model,
            "content": [],
            "stop_reason": Value::Null,
            "stop_sequence": Value::Null,
            "usage": { "input_tokens": 0, "output_tokens": 0 },
        });
        self.push_sse(
            "message_start",
            json!({
                "type": "message_start",
                "message": message,
            }),
        );
        self.started = true;
    }

    fn emit_content_block_start(&mut self, data: Option<&Value>) -> AppResult<()> {
        self.ensure_started(data);
        let Some(item) = data.and_then(output_item_from_event) else {
            return Ok(());
        };
        let item_type = item.get("type").and_then(Value::as_str);
        let item_key = item_key(item);
        let index = self.next_index;
        self.next_index += 1;
        if let Some(key) = item_key {
            self.item_indexes.insert(key, index);
        }
        match item_type {
            Some("message") => self.push_sse(
                "content_block_start",
                json!({
                    "type": "content_block_start",
                    "index": index,
                    "content_block": { "type": "text", "text": "" },
                }),
            ),
            Some("function_call") | Some("custom_tool_call") => self.push_sse(
                "content_block_start",
                json!({
                    "type": "content_block_start",
                    "index": index,
                    "content_block": {
                        "type": "tool_use",
                        "id": item
                            .get("call_id")
                            .or_else(|| item.get("id"))
                            .and_then(Value::as_str)
                            .unwrap_or("call_responses_adapter"),
                        "name": item.get("name").and_then(Value::as_str).unwrap_or("tool"),
                        "input": {},
                    },
                }),
            ),
            _ => {}
        }
        Ok(())
    }

    fn emit_text_delta(&mut self, data: Option<&Value>) -> AppResult<()> {
        self.ensure_started(data);
        let Some(data) = data else {
            return Ok(());
        };
        let index = self.index_for_delta(data, "text");
        let text = data
            .get("delta")
            .and_then(Value::as_str)
            .or_else(|| data.get("text").and_then(Value::as_str))
            .unwrap_or("");
        self.push_sse(
            "content_block_delta",
            json!({
                "type": "content_block_delta",
                "index": index,
                "delta": { "type": "text_delta", "text": text },
            }),
        );
        Ok(())
    }

    fn emit_arguments_delta(&mut self, data: Option<&Value>) -> AppResult<()> {
        self.ensure_started(data);
        let Some(data) = data else {
            return Ok(());
        };
        let index = self.index_for_delta(data, "function_call");
        let partial_json = data
            .get("delta")
            .and_then(Value::as_str)
            .or_else(|| data.get("arguments").and_then(Value::as_str))
            .unwrap_or("");
        self.push_sse(
            "content_block_delta",
            json!({
                "type": "content_block_delta",
                "index": index,
                "delta": { "type": "input_json_delta", "partial_json": partial_json },
            }),
        );
        Ok(())
    }

    fn emit_custom_tool_input_delta(&mut self, data: Option<&Value>) -> AppResult<()> {
        self.ensure_started(data);
        let Some(data) = data else {
            return Ok(());
        };
        let index = self.index_for_delta(data, "function_call");
        let partial_json = data
            .get("delta")
            .and_then(Value::as_str)
            .or_else(|| data.get("input").and_then(Value::as_str))
            .unwrap_or("");
        self.push_sse(
            "content_block_delta",
            json!({
                "type": "content_block_delta",
                "index": index,
                "delta": { "type": "input_json_delta", "partial_json": partial_json },
            }),
        );
        Ok(())
    }

    fn emit_content_block_stop(&mut self, data: Option<&Value>) {
        self.ensure_started(data);
        let Some(item) = data.and_then(output_item_from_event) else {
            return;
        };
        self.output_items.push(item.clone());
        if let Some(key) = item_key(item) {
            if let Some(index) = self.item_indexes.get(&key).copied() {
                self.push_sse(
                    "content_block_stop",
                    json!({
                        "type": "content_block_stop",
                        "index": index,
                    }),
                );
            }
        }
    }

    fn emit_message_done(&mut self, data: Option<&Value>) -> AppResult<()> {
        self.ensure_started(data);
        let mut response = data
            .and_then(|data| data.get("response"))
            .cloned()
            .or_else(|| data.cloned())
            .unwrap_or_else(|| json!({ "id": DEFAULT_RESPONSE_ID, "status": "completed" }));
        let output_missing = response
            .get("output")
            .and_then(Value::as_array)
            .map(Vec::is_empty)
            .unwrap_or(true);
        if output_missing && !self.output_items.is_empty() {
            if let Some(object) = response.as_object_mut() {
                object.insert("output".into(), Value::Array(self.output_items.clone()));
            }
        }
        let message = responses_json_to_anthropic_message(response, self.requested_model)?;
        self.push_sse(
            "message_delta",
            json!({
                "type": "message_delta",
                "delta": {
                    "stop_reason": message["stop_reason"].clone(),
                    "stop_sequence": Value::Null,
                },
                "usage": message["usage"].clone(),
            }),
        );
        self.push_sse("message_stop", json!({ "type": "message_stop" }));
        Ok(())
    }

    fn emit_error(&mut self, data: Option<&Value>) {
        let message = data
            .and_then(|data| {
                data.get("error")
                    .and_then(|error| error.get("message"))
                    .or_else(|| data.get("message"))
            })
            .and_then(Value::as_str)
            .unwrap_or("Responses API returned failure");
        self.push_sse(
            "error",
            json!({
                "type": "error",
                "error": { "type": "api_error", "message": message },
            }),
        );
    }

    fn index_for_delta(&mut self, data: &Value, kind: &str) -> usize {
        if let Some(key) = delta_key(data) {
            if let Some(index) = self.item_indexes.get(&key).copied() {
                return index;
            }
            let index = self.next_index;
            self.next_index += 1;
            self.item_indexes.insert(key, index);
            let content_block = if kind == "function_call" {
                json!({ "type": "tool_use", "id": "call_responses_adapter", "name": "tool", "input": {} })
            } else {
                json!({ "type": "text", "text": "" })
            };
            self.push_sse(
                "content_block_start",
                json!({
                    "type": "content_block_start",
                    "index": index,
                    "content_block": content_block,
                }),
            );
            return index;
        }
        0
    }

    fn push_sse(&mut self, event: &str, data: Value) {
        self.output.push_str("event: ");
        self.output.push_str(event);
        self.output.push('\n');
        self.output.push_str("data: ");
        self.output.push_str(&data.to_string());
        self.output.push_str("\n\n");
    }
}

fn sse_blocks(input: &str) -> Vec<String> {
    let mut blocks = Vec::new();
    let mut current = String::new();
    for line in input.lines() {
        current.push_str(line);
        current.push('\n');
        if line.is_empty() {
            blocks.push(std::mem::take(&mut current));
        }
    }
    if !current.is_empty() {
        blocks.push(current);
    }
    blocks
}

fn find_sse_separator(bytes: &[u8]) -> Option<(usize, usize)> {
    let mut index = 0;
    while index < bytes.len() {
        if bytes.get(index..index + 2) == Some(b"\n\n") {
            return Some((index, 2));
        }
        if bytes.get(index..index + 4) == Some(b"\r\n\r\n") {
            return Some((index, 4));
        }
        index += 1;
    }
    None
}

fn event_name(block: &str) -> Option<String> {
    block.lines().find_map(|line| {
        line.strip_prefix("event:")
            .map(str::trim_start)
            .map(ToOwned::to_owned)
    })
}

fn event_data_json(block: &str) -> Option<Value> {
    let data = block
        .lines()
        .filter_map(|line| line.strip_prefix("data:").map(str::trim_start))
        .collect::<Vec<_>>()
        .join("\n");
    if data.is_empty() {
        None
    } else {
        serde_json::from_str(&data).ok()
    }
}

fn output_item_from_event(data: &Value) -> Option<&Value> {
    data.get("item").or_else(|| {
        if data.get("type").is_some() || data.get("id").is_some() {
            Some(data)
        } else {
            None
        }
    })
}

fn item_key(item: &Value) -> Option<String> {
    item.get("id")
        .and_then(Value::as_str)
        .or_else(|| item.get("item_id").and_then(Value::as_str))
        .or_else(|| item.get("output_index").and_then(Value::as_i64).map(|_| ""))
        .map(|value| {
            if value.is_empty() {
                item.get("output_index").unwrap().to_string()
            } else {
                value.to_string()
            }
        })
}

fn delta_key(data: &Value) -> Option<String> {
    data.get("item_id")
        .and_then(Value::as_str)
        .or_else(|| data.get("id").and_then(Value::as_str))
        .map(ToOwned::to_owned)
        .or_else(|| data.get("output_index").map(Value::to_string))
}
