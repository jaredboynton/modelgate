use bytes::Bytes;
use serde_json::{json, Map, Value};
use std::collections::{HashMap, HashSet};

use crate::{AppError, AppResult};

const MAX_TOOL_NAME_LEN: usize = 64;

#[derive(Clone, Debug, Default)]
pub struct GoogleToolContext {
    custom_tool_names: HashSet<String>,
}

impl GoogleToolContext {
    fn is_custom_tool(&self, name: &str) -> bool {
        self.custom_tool_names.contains(name)
    }
}

pub fn responses_to_google_generate_content(body: Value, upstream_model: &str) -> AppResult<Value> {
    let (body, _) = responses_to_google_generate_content_with_context(body, upstream_model)?;
    Ok(body)
}

pub fn responses_to_google_generate_content_with_context(
    body: Value,
    upstream_model: &str,
) -> AppResult<(Value, GoogleToolContext)> {
    if upstream_model.trim().is_empty() {
        return Err(AppError::BadRequest("missing Google upstream model".into()));
    }
    let object = body
        .as_object()
        .ok_or_else(|| AppError::BadRequest("Responses request must be a JSON object".into()))?;
    reject_unsupported_responses_request(object)?;

    let mut system_parts = Vec::new();
    if let Some(instructions) = object.get("instructions").and_then(Value::as_str) {
        if !instructions.is_empty() {
            system_parts.push(json!({ "text": instructions }));
        }
    }

    let input = object
        .get("input")
        .ok_or_else(|| AppError::BadRequest("missing input".into()))?;
    let contents = responses_input_to_google_contents(input, &mut system_parts)?;
    if contents.is_empty() {
        return Err(AppError::BadRequest(
            "input produced no Google contents".into(),
        ));
    }

    let mut out = Map::new();
    out.insert("contents".into(), Value::Array(contents));
    if !system_parts.is_empty() {
        out.insert(
            "systemInstruction".into(),
            json!({
                "parts": system_parts,
            }),
        );
    }

    let generation_config = google_generation_config(object, upstream_model)?;
    if !generation_config.is_empty() {
        out.insert("generationConfig".into(), Value::Object(generation_config));
    }
    let (tools, tool_names, tool_context) = google_tools(object)?;
    if let Some(tools) = tools {
        out.insert("tools".into(), tools);
    }
    if let Some(tool_config) = google_tool_config(object, &tool_names)? {
        out.insert("toolConfig".into(), tool_config);
    }
    Ok((Value::Object(out), tool_context))
}

pub fn google_generate_content_to_responses(
    body: Value,
    requested_model: &str,
) -> AppResult<Value> {
    google_generate_content_to_responses_with_context(
        body,
        requested_model,
        &GoogleToolContext::default(),
    )
}

pub fn google_generate_content_to_responses_with_context(
    body: Value,
    requested_model: &str,
    tool_context: &GoogleToolContext,
) -> AppResult<Value> {
    let candidates = body
        .get("candidates")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            AppError::Upstream("Google generateContent response missing candidates".into())
        })?;
    let candidate = candidates.first().ok_or_else(|| {
        AppError::Upstream("Google generateContent response contained no text candidates".into())
    })?;
    let parts = candidate
        .pointer("/content/parts")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            AppError::Upstream(
                "Google generateContent response contained no text candidates".into(),
            )
        })?;
    let output = google_parts_to_responses_output(parts, tool_context)?;
    if output.is_empty() {
        return Err(AppError::Upstream(
            "Google generateContent response contained no text candidates".into(),
        ));
    }

    let finish_reason = candidate
        .get("finishReason")
        .and_then(Value::as_str)
        .unwrap_or("STOP");
    let status = if finish_reason == "MAX_TOKENS" {
        "incomplete"
    } else {
        "completed"
    };

    let mut response = json!({
        "id": "resp_google_generate_content",
        "object": "response",
        "created_at": 0,
        "model": requested_model,
        "status": status,
        "output": output,
        "usage": google_usage(body.get("usageMetadata")),
    });
    if finish_reason == "MAX_TOKENS" {
        response["incomplete_details"] = json!({ "reason": "max_output_tokens" });
    }
    Ok(response)
}

pub fn is_google_responses_stream_request(body: &Value) -> bool {
    body.get("stream").and_then(Value::as_bool) == Some(true)
}

pub struct GoogleResponsesSseTranslator {
    requested_model: String,
    tool_context: GoogleToolContext,
    buffer: String,
    started: bool,
    item_started: bool,
    completed: bool,
    text: String,
    text_index: Option<usize>,
    output_items: Vec<Value>,
    finish_reason: Option<String>,
    usage: Option<Value>,
}

impl GoogleResponsesSseTranslator {
    pub fn new(requested_model: &str) -> Self {
        Self::with_tool_context(requested_model, GoogleToolContext::default())
    }

    pub fn with_tool_context(requested_model: &str, tool_context: GoogleToolContext) -> Self {
        Self {
            requested_model: requested_model.to_string(),
            tool_context,
            buffer: String::new(),
            started: false,
            item_started: false,
            completed: false,
            text: String::new(),
            text_index: None,
            output_items: Vec::new(),
            finish_reason: None,
            usage: None,
        }
    }

    pub fn push_bytes(&mut self, bytes: Bytes) -> AppResult<Bytes> {
        let text = std::str::from_utf8(&bytes)
            .map_err(|err| AppError::Upstream(format!("invalid Google SSE UTF-8: {err}")))?;
        self.buffer.push_str(text);

        let mut output = String::new();
        let mut consumed = 0usize;
        loop {
            let buf = &self.buffer[consumed..];
            let Some(rel_end) = find_sse_frame_end(buf) else {
                break;
            };
            let abs_end = consumed + rel_end;
            let frame = self.buffer[consumed..abs_end].to_string();
            consumed = abs_end;
            // advance over the standard SSE frame separator
            let bytes = self.buffer.as_bytes();
            if bytes.get(consumed) == Some(&b'\n') {
                consumed += 1;
            }
            if bytes.get(consumed) == Some(&b'\r') {
                consumed += 1;
            }
            if bytes.get(consumed) == Some(&b'\n') {
                consumed += 1;
            }
            output.push_str(&self.process_frame(&frame)?);
        }
        if consumed > 0 {
            // Single drain of the consumed prefix instead of repeated shifts inside the loop.
            let _ = self.buffer.drain(..consumed);
        }
        Ok(Bytes::from(output))
    }

    pub fn finish(&mut self) -> AppResult<Bytes> {
        let mut output = String::new();
        if !self.buffer.is_empty() {
            let frame = std::mem::take(&mut self.buffer);
            output.push_str(&self.process_frame(&frame)?);
        }
        if self.started && !self.completed {
            output.push_str(&self.emit_completed());
        }
        Ok(Bytes::from(output))
    }

    fn process_frame(&mut self, frame: &str) -> AppResult<String> {
        let data = collect_sse_data(frame);
        if data.is_empty() {
            return Ok(String::new());
        }
        if data == "[DONE]" {
            return Ok(self.emit_completed());
        }

        let value: Value = serde_json::from_str(&data)
            .map_err(|err| AppError::Upstream(format!("invalid Google SSE JSON: {err}")))?;
        let mut output = String::new();
        output.push_str(&self.ensure_started());

        if let Some(usage) = value.get("usageMetadata") {
            self.usage = Some(usage.clone());
        }

        for part in google_candidate_parts(&value) {
            if let Some(delta) = part.get("text").and_then(Value::as_str) {
                if !delta.is_empty() {
                    output.push_str(&self.emit_text_delta(delta));
                }
            }
            if part.get("functionCall").is_some() {
                output.push_str(&self.emit_function_call(part));
            }
        }

        if let Some(finish_reason) = google_finish_reason(&value) {
            self.finish_reason = Some(finish_reason.to_string());
            output.push_str(&self.emit_completed());
        }
        Ok(output)
    }

    fn ensure_started(&mut self) -> String {
        if self.started {
            return String::new();
        }
        self.started = true;
        sse_event(
            "response.created",
            json!({
                "type": "response.created",
                "response": self.response_envelope("in_progress"),
            }),
        )
    }

    fn ensure_text_item_started(&mut self) -> String {
        if self.item_started {
            return String::new();
        }
        self.item_started = true;
        let index = self.output_items.len();
        self.text_index = Some(index);
        self.output_items.push(self.output_item("in_progress"));
        sse_event(
            "response.output_item.added",
            json!({
                "type": "response.output_item.added",
                "output_index": index,
                "item": self.output_item("in_progress"),
            }),
        )
    }

    fn emit_text_delta(&mut self, delta: &str) -> String {
        let mut output = self.ensure_text_item_started();
        self.text.push_str(delta);
        output.push_str(&sse_event(
            "response.output_text.delta",
            json!({
                "type": "response.output_text.delta",
                "output_index": self.text_index.unwrap_or(0),
                "content_index": 0,
                "delta": delta,
            }),
        ));
        output
    }

    fn emit_function_call(&mut self, part: &Value) -> String {
        let index = self.output_items.len();
        let function_call = part.get("functionCall").unwrap_or(&Value::Null);
        let thought_signature = google_thought_signature_from_part(part);
        let name = function_call
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or("tool");
        let call_id = function_call
            .get("id")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| format!("call_google_{index}"));
        if self.tool_context.is_custom_tool(name) {
            return self.emit_custom_tool_call(
                index,
                name,
                &call_id,
                function_call,
                thought_signature,
            );
        }
        let arguments = compact_json(function_call.get("args").unwrap_or(&json!({})));
        let mut item = json!({
            "id": format!("fc_{index}"),
            "type": "function_call",
            "status": "in_progress",
            "call_id": call_id,
            "name": name,
            "arguments": "",
        });
        attach_google_thought_signature(&mut item, thought_signature);
        self.output_items.push(item.clone());

        let mut output = sse_event(
            "response.output_item.added",
            json!({
                "type": "response.output_item.added",
                "output_index": index,
                "item": item,
            }),
        );
        if !arguments.is_empty() {
            output.push_str(&sse_event(
                "response.function_call_arguments.delta",
                json!({
                    "type": "response.function_call_arguments.delta",
                    "output_index": index,
                    "delta": arguments,
                }),
            ));
        }
        item["status"] = Value::String("completed".into());
        item["arguments"] = Value::String(arguments);
        self.output_items[index] = item.clone();
        output.push_str(&sse_event(
            "response.output_item.done",
            json!({
                "type": "response.output_item.done",
                "output_index": index,
                "item": item,
            }),
        ));
        output
    }

    fn emit_custom_tool_call(
        &mut self,
        index: usize,
        name: &str,
        call_id: &str,
        function_call: &Value,
        thought_signature: Option<&str>,
    ) -> String {
        let input = google_custom_tool_input(function_call.get("args"));
        let mut item = json!({
            "id": format!("ctc_{index}"),
            "type": "custom_tool_call",
            "status": "in_progress",
            "call_id": call_id,
            "name": name,
            "input": "",
        });
        attach_google_thought_signature(&mut item, thought_signature);
        self.output_items.push(item.clone());

        let mut output = sse_event(
            "response.output_item.added",
            json!({
                "type": "response.output_item.added",
                "output_index": index,
                "item": item,
            }),
        );
        if !input.is_empty() {
            output.push_str(&sse_event(
                "response.custom_tool_call_input.delta",
                json!({
                    "type": "response.custom_tool_call_input.delta",
                    "output_index": index,
                    "item_id": format!("ctc_{index}"),
                    "call_id": call_id,
                    "delta": input,
                }),
            ));
        }
        item["status"] = Value::String("completed".into());
        item["input"] = Value::String(input);
        self.output_items[index] = item.clone();
        output.push_str(&sse_event(
            "response.output_item.done",
            json!({
                "type": "response.output_item.done",
                "output_index": index,
                "item": item,
            }),
        ));
        output
    }

    fn emit_completed(&mut self) -> String {
        if self.completed {
            return String::new();
        }
        self.completed = true;
        let mut output = String::new();
        output.push_str(&self.ensure_started());
        if self.item_started {
            let index = self.text_index.unwrap_or(0);
            let item = self.output_item("completed");
            self.output_items[index] = item.clone();
            output.push_str(&sse_event(
                "response.output_item.done",
                json!({
                    "type": "response.output_item.done",
                    "output_index": index,
                    "item": item,
                }),
            ));
        }

        let status = if self.finish_reason.as_deref() == Some("MAX_TOKENS") {
            "incomplete"
        } else {
            "completed"
        };
        let mut response = self.response_envelope(status);
        if status == "incomplete" {
            response["incomplete_details"] = json!({ "reason": "max_output_tokens" });
        }
        output.push_str(&sse_event(
            "response.completed",
            json!({
                "type": "response.completed",
                "response": response,
            }),
        ));
        output
    }

    fn response_envelope(&self, status: &str) -> Value {
        let output = self.output_items.clone();
        let mut response = json!({
            "id": "resp_google_generate_content",
            "object": "response",
            "created_at": 0,
            "model": self.requested_model,
            "status": status,
            "output": output,
            "usage": google_usage(self.usage.as_ref()),
        });
        if status == "incomplete" {
            response["incomplete_details"] = json!({ "reason": "max_output_tokens" });
        }
        response
    }

    fn output_item(&self, status: &str) -> Value {
        let content = if status == "in_progress" {
            Vec::new()
        } else {
            vec![json!({
                "type": "output_text",
                "text": self.text,
            })]
        };
        json!({
            "id": "msg_google_generate_content",
            "type": "message",
            "status": status,
            "role": "assistant",
            "content": content,
        })
    }
}

pub fn google_generate_content_sse_to_responses_sse_text(
    input: &str,
    requested_model: &str,
) -> AppResult<String> {
    let mut translator = GoogleResponsesSseTranslator::new(requested_model);
    let first = translator.push_bytes(Bytes::copy_from_slice(input.as_bytes()))?;
    let second = translator.finish()?;
    let mut output = Vec::with_capacity(first.len() + second.len());
    output.extend_from_slice(&first);
    output.extend_from_slice(&second);
    String::from_utf8(output)
        .map_err(|err| AppError::Upstream(format!("invalid Responses SSE UTF-8: {err}")))
}

fn reject_unsupported_responses_request(object: &Map<String, Value>) -> AppResult<()> {
    if object
        .get("previous_response_id")
        .is_some_and(|value| !value.is_null())
    {
        return Err(AppError::BadRequest(
            "previous_response_id is not supported for Google Responses adapter yet".into(),
        ));
    }
    if object
        .get("conversation")
        .is_some_and(|value| !value.is_null())
    {
        return Err(AppError::BadRequest(
            "conversation is not supported for Google Responses adapter yet".into(),
        ));
    }
    if object.get("store").and_then(Value::as_bool) == Some(true) {
        return Err(AppError::BadRequest(
            "store=true is not supported for Google Responses adapter yet".into(),
        ));
    }
    Ok(())
}

fn responses_input_to_google_contents(
    input: &Value,
    system_parts: &mut Vec<Value>,
) -> AppResult<Vec<Value>> {
    match input {
        Value::String(text) => Ok(vec![google_content("user", vec![json!({ "text": text })])]),
        Value::Array(items) => {
            let mut contents = Vec::new();
            let mut function_names = HashMap::new();
            for item in items {
                let object = item
                    .as_object()
                    .ok_or_else(|| AppError::BadRequest("input item must be an object".into()))?;
                match object.get("type").and_then(Value::as_str) {
                    Some("message") | None if object.contains_key("role") => {
                        convert_message_item(object, &mut contents, system_parts)?
                    }
                    Some("function_call") => {
                        convert_function_call_item(object, &mut contents, &mut function_names)?
                    }
                    Some("function_call_output") => {
                        convert_function_call_output_item(object, &mut contents, &function_names)?
                    }
                    Some("custom_tool_call") => {
                        convert_custom_tool_call_item(object, &mut contents, &mut function_names)?
                    }
                    Some("custom_tool_call_output") => convert_custom_tool_call_output_item(
                        object,
                        &mut contents,
                        &function_names,
                    )?,
                    Some("reasoning") | Some("item_reference") => {
                        return Err(AppError::BadRequest(format!(
                            "{} is not supported for Google Responses adapter yet",
                            object.get("type").and_then(Value::as_str).unwrap_or("item")
                        )))
                    }
                    Some(other) => {
                        return Err(AppError::BadRequest(format!(
                            "unsupported Responses input item for Google adapter: {other}"
                        )))
                    }
                    None => {
                        return Err(AppError::BadRequest(
                            "input item missing type or role".into(),
                        ))
                    }
                }
            }
            Ok(contents)
        }
        _ => Err(AppError::BadRequest(
            "input must be a string or array".into(),
        )),
    }
}

fn convert_function_call_item(
    object: &Map<String, Value>,
    contents: &mut Vec<Value>,
    function_names: &mut HashMap<String, String>,
) -> AppResult<()> {
    let call_id = required_string(object, "call_id")?;
    let name = required_string(object, "name")?;
    let args = parse_json_object_string(object.get("arguments"), "function_call arguments")?;
    function_names.insert(call_id.to_string(), name.to_string());
    let mut part = json!({
        "functionCall": {
            "id": call_id,
            "name": name,
            "args": args,
        }
    });
    attach_google_part_thought_signature(&mut part, google_thought_signature_from_item(object));
    contents.push(google_content("model", vec![part]));
    Ok(())
}

fn convert_custom_tool_call_item(
    object: &Map<String, Value>,
    contents: &mut Vec<Value>,
    function_names: &mut HashMap<String, String>,
) -> AppResult<()> {
    let call_id = required_string(object, "call_id")?;
    let name = required_string(object, "name")?;
    let input = required_string(object, "input")?;
    function_names.insert(call_id.to_string(), name.to_string());
    let mut part = json!({
        "functionCall": {
            "id": call_id,
            "name": name,
            "args": { "input": input },
        }
    });
    attach_google_part_thought_signature(&mut part, google_thought_signature_from_item(object));
    contents.push(google_content("model", vec![part]));
    Ok(())
}

fn convert_function_call_output_item(
    object: &Map<String, Value>,
    contents: &mut Vec<Value>,
    function_names: &HashMap<String, String>,
) -> AppResult<()> {
    let call_id = required_string(object, "call_id")?;
    let name = object
        .get("name")
        .and_then(Value::as_str)
        .or_else(|| function_names.get(call_id).map(String::as_str))
        .unwrap_or("tool");
    let response = parse_function_call_output(object.get("output"))?;
    contents.push(google_content(
        "user",
        vec![json!({
            "functionResponse": {
                "id": call_id,
                "name": name,
                "response": response,
            }
        })],
    ));
    Ok(())
}

fn convert_custom_tool_call_output_item(
    object: &Map<String, Value>,
    contents: &mut Vec<Value>,
    function_names: &HashMap<String, String>,
) -> AppResult<()> {
    let call_id = required_string(object, "call_id")?;
    let name = object
        .get("name")
        .and_then(Value::as_str)
        .or_else(|| function_names.get(call_id).map(String::as_str))
        .unwrap_or("tool");
    let response = parse_function_call_output(object.get("output"))?;
    contents.push(google_content(
        "user",
        vec![json!({
            "functionResponse": {
                "id": call_id,
                "name": name,
                "response": response,
            }
        })],
    ));
    Ok(())
}

fn convert_message_item(
    object: &Map<String, Value>,
    contents: &mut Vec<Value>,
    system_parts: &mut Vec<Value>,
) -> AppResult<()> {
    let role = object
        .get("role")
        .and_then(Value::as_str)
        .ok_or_else(|| AppError::BadRequest("missing role".into()))?;
    let content = object.get("content").unwrap_or(&Value::Null);
    let text = responses_content_text(content)?;
    match role {
        "system" | "developer" => {
            if !text.is_empty() {
                system_parts.push(json!({ "text": text }));
            }
        }
        "user" => contents.push(google_content("user", vec![json!({ "text": text })])),
        "assistant" => contents.push(google_content("model", vec![json!({ "text": text })])),
        other => {
            return Err(AppError::BadRequest(format!(
                "unsupported Responses message role for Google adapter: {other}"
            )))
        }
    }
    Ok(())
}

fn responses_content_text(content: &Value) -> AppResult<String> {
    match content {
        Value::String(text) => Ok(text.to_string()),
        Value::Array(blocks) => blocks
            .iter()
            .map(responses_content_block_text)
            .collect::<AppResult<Vec<_>>>()
            .map(|parts| parts.join("")),
        Value::Null => Ok(String::new()),
        _ => Err(AppError::BadRequest(
            "message content must be a string or text blocks".into(),
        )),
    }
}

fn responses_content_block_text(block: &Value) -> AppResult<String> {
    let object = block
        .as_object()
        .ok_or_else(|| AppError::BadRequest("content block must be an object".into()))?;
    match object.get("type").and_then(Value::as_str) {
        Some("input_text") | Some("output_text") | Some("text") => object
            .get("text")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned)
            .ok_or_else(|| AppError::BadRequest("text content block missing text".into())),
        Some("input_image") | Some("input_file") | Some("file") | Some("image") => {
            Err(AppError::BadRequest(
                "non-text content blocks are not supported for Google Responses adapter yet".into(),
            ))
        }
        Some(other) => Err(AppError::BadRequest(format!(
            "unsupported content block for Google Responses adapter: {other}"
        ))),
        None => Err(AppError::BadRequest("content block missing type".into())),
    }
}

fn google_content(role: &str, parts: Vec<Value>) -> Value {
    json!({
        "role": role,
        "parts": parts,
    })
}

fn google_generation_config(
    object: &Map<String, Value>,
    upstream_model: &str,
) -> AppResult<Map<String, Value>> {
    let mut config = Map::new();
    copy_field(object, &mut config, "max_output_tokens", "maxOutputTokens");
    copy_field(object, &mut config, "temperature", "temperature");
    copy_field(object, &mut config, "top_p", "topP");
    if let Some(stop) = object.get("stop") {
        if let Some(stop_sequences) = stop_sequences(stop)? {
            config.insert("stopSequences".into(), stop_sequences);
        }
    }
    if let Some(thinking_config) = google_thinking_config(object.get("reasoning"), upstream_model)?
    {
        config.insert("thinkingConfig".into(), thinking_config);
    }
    Ok(config)
}

fn google_thinking_config(
    reasoning: Option<&Value>,
    upstream_model: &str,
) -> AppResult<Option<Value>> {
    let Some(reasoning) = reasoning else {
        return Ok(None);
    };
    if reasoning.is_null() {
        return Ok(None);
    }
    let reasoning = reasoning
        .as_object()
        .ok_or_else(|| AppError::BadRequest("reasoning must be an object".into()))?;
    if let Some(effort) = reasoning.get("effort").and_then(Value::as_str) {
        if is_gemini_3_model(upstream_model) {
            return Ok(Some(json!({
                "thinkingLevel": google_thinking_level_for_effort(effort, upstream_model)?
            })));
        }
        return Ok(Some(json!({
            "thinkingBudget": google_thinking_budget_for_effort(effort)?
        })));
    }
    if let Some(budget) = reasoning.get("budget_tokens") {
        let budget = budget.as_i64().ok_or_else(|| {
            AppError::BadRequest("reasoning.budget_tokens must be an integer".into())
        })?;
        return Ok(Some(json!({ "thinkingBudget": budget })));
    }
    Ok(None)
}

fn is_gemini_3_model(upstream_model: &str) -> bool {
    upstream_model.starts_with("gemini-3")
}

fn google_thinking_level_for_effort(effort: &str, upstream_model: &str) -> AppResult<&'static str> {
    Ok(match effort {
        "none" => {
            if google_model_supports_minimal_thinking(upstream_model) {
                "minimal"
            } else {
                "low"
            }
        }
        "minimal" => "minimal",
        "low" => "low",
        "medium" => "medium",
        "high" | "xhigh" | "max" => "high",
        other => {
            return Err(AppError::BadRequest(format!(
                "unsupported reasoning effort for Google adapter: {other}"
            )))
        }
    })
}

fn google_model_supports_minimal_thinking(upstream_model: &str) -> bool {
    upstream_model.contains("flash")
}

fn google_thinking_budget_for_effort(effort: &str) -> AppResult<i64> {
    Ok(match effort {
        "none" | "minimal" => 0,
        "low" => 1_024,
        "medium" => 4_096,
        "high" => 8_192,
        "xhigh" | "max" => 16_384,
        other => {
            return Err(AppError::BadRequest(format!(
                "unsupported reasoning effort for Google adapter: {other}"
            )))
        }
    })
}

fn google_tools(
    object: &Map<String, Value>,
) -> AppResult<(Option<Value>, Vec<String>, GoogleToolContext)> {
    let Some(tools) = object.get("tools") else {
        return Ok((None, Vec::new(), GoogleToolContext::default()));
    };
    if tools.is_null() {
        return Ok((None, Vec::new(), GoogleToolContext::default()));
    }
    let mut declarations = Vec::new();
    let mut function_names = Vec::new();
    let mut context = GoogleToolContext::default();
    for tool in tools
        .as_array()
        .ok_or_else(|| AppError::BadRequest("tools must be an array".into()))?
    {
        if let Some((declaration, is_custom)) = google_function_declaration(tool)? {
            if let Some(name) = declaration.get("name").and_then(Value::as_str) {
                function_names.push(name.to_string());
                if is_custom {
                    context.custom_tool_names.insert(name.to_string());
                }
            }
            declarations.push(declaration);
        }
    }
    if declarations.is_empty() {
        return Ok((None, Vec::new(), context));
    }
    Ok((
        Some(json!([{ "functionDeclarations": declarations }])),
        function_names,
        context,
    ))
}

fn google_function_declaration(tool: &Value) -> AppResult<Option<(Value, bool)>> {
    let object = tool
        .as_object()
        .ok_or_else(|| AppError::BadRequest("tool must be an object".into()))?;
    let tool_type = object.get("type").and_then(Value::as_str).unwrap_or("");
    match tool_type {
        "function" | "" => {
            let source = object
                .get("function")
                .and_then(Value::as_object)
                .unwrap_or(object);
            Ok(Some((google_function_tool_declaration(source)?, false)))
        }
        "freeform" | "custom" => Ok(Some((
            google_custom_tool_declaration(object, tool_type)?,
            true,
        ))),
        "code_interpreter"
        | "computer_use_preview"
        | "file_search"
        | "image_generation"
        | "local_shell"
        | "namespace"
        | "tool_search"
        | "web_search"
        | "web_search_preview" => Ok(None),
        other => Err(AppError::BadRequest(format!(
            "unsupported Responses tool type for Google adapter: {other}"
        ))),
    }
}

fn google_function_tool_declaration(source: &Map<String, Value>) -> AppResult<Value> {
    let name = required_string(source, "name")?;
    validate_tool_name(name)?;
    let mut declaration = Map::new();
    declaration.insert("name".into(), Value::String(name.to_string()));
    if let Some(description) = source.get("description").and_then(Value::as_str) {
        declaration.insert("description".into(), Value::String(description.to_string()));
    }
    if let Some(parameters) = source.get("parameters") {
        declaration.insert("parameters".into(), google_sanitize_schema(parameters)?);
    }
    Ok(Value::Object(declaration))
}

fn google_custom_tool_declaration(
    object: &Map<String, Value>,
    tool_type: &str,
) -> AppResult<Value> {
    let name = required_string(object, "name")?;
    validate_tool_name(name)?;
    let description = object
        .get("description")
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| {
            format!("Codex {tool_type} tool '{name}' adapted to a function tool. Pass the body as the `input` argument.")
        });

    Ok(json!({
        "name": name,
        "description": description,
        "parameters": {
            "type": "object",
            "properties": {
                "input": {
                    "type": "string",
                    "description": "Raw freeform tool body."
                }
            },
            "required": ["input"]
        }
    }))
}

fn google_sanitize_schema(schema: &Value) -> AppResult<Value> {
    let object = schema
        .as_object()
        .ok_or_else(|| AppError::BadRequest("tool parameter schema must be an object".into()))?;

    if let Some(union_schema) = google_sanitize_union_schema(object)? {
        return Ok(union_schema);
    }

    let mut out = Map::new();
    let (schema_type, nullable_from_type) = google_schema_type(object.get("type"))?;
    if let Some(schema_type) = schema_type {
        out.insert("type".into(), Value::String(schema_type));
    }
    if let Some(description) = object.get("description").and_then(Value::as_str) {
        out.insert("description".into(), Value::String(description.to_string()));
    }
    if let Some(format) = object.get("format").and_then(Value::as_str) {
        out.insert("format".into(), Value::String(format.to_string()));
    }
    if let Some(nullable) = object.get("nullable").and_then(Value::as_bool) {
        out.insert(
            "nullable".into(),
            Value::Bool(nullable || nullable_from_type),
        );
    } else if nullable_from_type {
        out.insert("nullable".into(), Value::Bool(true));
    }
    if let Some(enum_values) = object.get("enum").and_then(Value::as_array) {
        out.insert("enum".into(), Value::Array(enum_values.clone()));
    }
    if let Some(required) = object.get("required").and_then(Value::as_array) {
        let required = required
            .iter()
            .filter_map(Value::as_str)
            .map(|name| Value::String(name.to_string()))
            .collect::<Vec<_>>();
        if !required.is_empty() {
            out.insert("required".into(), Value::Array(required));
        }
    }
    if let Some(properties) = object.get("properties").and_then(Value::as_object) {
        let mut converted = Map::new();
        for (name, property) in properties {
            converted.insert(name.clone(), google_sanitize_schema(property)?);
        }
        out.insert("properties".into(), Value::Object(converted));
        out.entry("type")
            .or_insert_with(|| Value::String("object".into()));
    }
    if let Some(items) = object.get("items") {
        out.insert("items".into(), google_sanitize_schema(items)?);
        out.entry("type")
            .or_insert_with(|| Value::String("array".into()));
    }
    Ok(Value::Object(out))
}

fn google_sanitize_union_schema(object: &Map<String, Value>) -> AppResult<Option<Value>> {
    for key in ["anyOf", "oneOf"] {
        let Some(options) = object.get(key) else {
            continue;
        };
        let options = options
            .as_array()
            .ok_or_else(|| AppError::BadRequest(format!("{key} must be an array")))?;
        let mut nullable = false;
        let mut selected = None;
        for option in options {
            if google_schema_option_is_null(option) {
                nullable = true;
            } else if selected.is_none() {
                selected = Some(option);
            }
        }
        if let Some(selected) = selected {
            let mut sanitized = google_sanitize_schema(selected)?;
            if nullable {
                if let Some(sanitized_object) = sanitized.as_object_mut() {
                    sanitized_object.insert("nullable".into(), Value::Bool(true));
                }
            }
            return Ok(Some(sanitized));
        }
        if nullable {
            return Ok(Some(json!({ "nullable": true })));
        }
    }
    Ok(None)
}

fn google_schema_option_is_null(option: &Value) -> bool {
    option.get("type").and_then(Value::as_str) == Some("null")
}

fn google_schema_type(value: Option<&Value>) -> AppResult<(Option<String>, bool)> {
    match value {
        Some(Value::String(schema_type)) if schema_type == "null" => Ok((None, true)),
        Some(Value::String(schema_type)) => Ok((Some(schema_type.clone()), false)),
        Some(Value::Array(types)) => {
            let mut nullable = false;
            let mut selected = None;
            for schema_type in types {
                let schema_type = schema_type.as_str().ok_or_else(|| {
                    AppError::BadRequest("tool parameter type entries must be strings".into())
                })?;
                if schema_type == "null" {
                    nullable = true;
                } else if selected.is_none() {
                    selected = Some(schema_type.to_string());
                }
            }
            Ok((selected, nullable))
        }
        Some(_) => Err(AppError::BadRequest(
            "tool parameter type must be a string or string array".into(),
        )),
        None => Ok((None, false)),
    }
}

fn google_tool_config(
    object: &Map<String, Value>,
    function_names: &[String],
) -> AppResult<Option<Value>> {
    let Some(tool_choice) = object.get("tool_choice") else {
        return Ok(None);
    };
    if tool_choice.is_null() {
        return Ok(None);
    }
    let mut config = Map::new();
    match tool_choice {
        Value::String(choice) => match choice.as_str() {
            "auto" => {
                if function_names.is_empty() {
                    return Ok(None);
                }
                config.insert("mode".into(), Value::String("AUTO".into()));
            }
            "required" => {
                if function_names.is_empty() {
                    return Err(AppError::BadRequest(
                        "tool_choice required needs at least one supported function tool for Google adapter".into(),
                    ));
                }
                config.insert("mode".into(), Value::String("ANY".into()));
            }
            "none" => {
                if function_names.is_empty() {
                    return Ok(None);
                }
                config.insert("mode".into(), Value::String("NONE".into()));
            }
            other => {
                return Err(AppError::BadRequest(format!(
                    "unsupported tool_choice: {other}"
                )))
            }
        },
        Value::Object(choice) => match choice.get("type").and_then(Value::as_str) {
            Some("auto") => {
                if function_names.is_empty() {
                    return Ok(None);
                }
                config.insert("mode".into(), Value::String("AUTO".into()));
            }
            Some("required") => {
                if function_names.is_empty() {
                    return Err(AppError::BadRequest(
                        "tool_choice required needs at least one supported function tool for Google adapter".into(),
                    ));
                }
                config.insert("mode".into(), Value::String("ANY".into()));
            }
            Some("none") => {
                if function_names.is_empty() {
                    return Ok(None);
                }
                config.insert("mode".into(), Value::String("NONE".into()));
            }
            Some("function") => {
                let name = choice
                    .get("name")
                    .and_then(Value::as_str)
                    .or_else(|| {
                        choice
                            .get("function")
                            .and_then(Value::as_object)
                            .and_then(|function| function.get("name"))
                            .and_then(Value::as_str)
                    })
                    .ok_or_else(|| AppError::BadRequest("tool_choice name is required".into()))?;
                if !function_names
                    .iter()
                    .any(|function_name| function_name == name)
                {
                    return Err(AppError::BadRequest(format!(
                        "tool_choice function is not a supported Google function tool: {name}"
                    )));
                }
                config.insert("mode".into(), Value::String("ANY".into()));
                config.insert("allowedFunctionNames".into(), json!([name]));
            }
            Some(other) => {
                return Err(AppError::BadRequest(format!(
                    "unsupported tool_choice type: {other}"
                )))
            }
            None => return Err(AppError::BadRequest("tool_choice type is required".into())),
        },
        _ => return Err(AppError::BadRequest("unsupported tool_choice".into())),
    }
    Ok(Some(
        json!({ "functionCallingConfig": Value::Object(config) }),
    ))
}

fn stop_sequences(stop: &Value) -> AppResult<Option<Value>> {
    match stop {
        Value::String(text) => Ok(Some(json!([text]))),
        Value::Array(values) if values.iter().all(Value::is_string) => Ok(Some(stop.clone())),
        Value::Null => Ok(None),
        _ => Err(AppError::BadRequest(
            "stop must be a string or array of strings".into(),
        )),
    }
}

fn copy_field(input: &Map<String, Value>, output: &mut Map<String, Value>, from: &str, to: &str) {
    if let Some(value) = input.get(from) {
        output.insert(to.into(), value.clone());
    }
}

fn google_thought_signature_from_part(part: &Value) -> Option<&str> {
    part.get("thoughtSignature")
        .or_else(|| part.get("thought_signature"))
        .and_then(Value::as_str)
}

fn google_thought_signature_from_item(object: &Map<String, Value>) -> Option<&str> {
    object
        .get("google_thought_signature")
        .or_else(|| object.get("thoughtSignature"))
        .or_else(|| object.get("thought_signature"))
        .and_then(Value::as_str)
}

fn attach_google_thought_signature(item: &mut Value, thought_signature: Option<&str>) {
    if let Some(thought_signature) = thought_signature {
        item["google_thought_signature"] = Value::String(thought_signature.to_string());
    }
}

fn attach_google_part_thought_signature(part: &mut Value, thought_signature: Option<&str>) {
    if let Some(thought_signature) = thought_signature {
        part["thoughtSignature"] = Value::String(thought_signature.to_string());
    }
}

fn google_parts_to_responses_output(
    parts: &[Value],
    tool_context: &GoogleToolContext,
) -> AppResult<Vec<Value>> {
    let mut output = Vec::new();
    let text = parts
        .iter()
        .filter_map(|part| part.get("text").and_then(Value::as_str))
        .collect::<Vec<_>>()
        .join("");
    if !text.is_empty() {
        output.push(json!({
            "id": "msg_google_generate_content",
            "type": "message",
            "status": "completed",
            "role": "assistant",
            "content": [{
                "type": "output_text",
                "text": text,
            }]
        }));
    }
    for part in parts
        .iter()
        .filter(|part| part.get("functionCall").is_some())
    {
        let function_call = part.get("functionCall").unwrap_or(&Value::Null);
        let thought_signature = google_thought_signature_from_part(part);
        let index = output.len();
        let name = function_call
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or("tool");
        let call_id = function_call
            .get("id")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| format!("call_google_{index}"));
        if tool_context.is_custom_tool(name) {
            let mut item = json!({
                "id": format!("ctc_{index}"),
                "type": "custom_tool_call",
                "status": "completed",
                "call_id": call_id,
                "name": name,
                "input": google_custom_tool_input(function_call.get("args")),
            });
            attach_google_thought_signature(&mut item, thought_signature);
            output.push(item);
            continue;
        }
        let mut item = json!({
            "id": format!("fc_{index}"),
            "type": "function_call",
            "status": "completed",
            "call_id": call_id,
            "name": name,
            "arguments": compact_json(function_call.get("args").unwrap_or(&json!({}))),
        });
        attach_google_thought_signature(&mut item, thought_signature);
        output.push(item);
    }
    Ok(output)
}

fn google_usage(usage: Option<&Value>) -> Value {
    let usage = usage.unwrap_or(&Value::Null);
    let input_tokens = usage
        .get("promptTokenCount")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let output_tokens = usage
        .get("candidatesTokenCount")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let total_tokens = usage
        .get("totalTokenCount")
        .and_then(Value::as_u64)
        .unwrap_or(input_tokens + output_tokens);
    json!({
        "input_tokens": input_tokens,
        "output_tokens": output_tokens,
        "total_tokens": total_tokens,
        "input_tokens_details": {
            "cached_tokens": 0,
        },
        "output_tokens_details": {
            "reasoning_tokens": 0,
        },
    })
}

fn google_candidate_parts(body: &Value) -> Vec<&Value> {
    body.get("candidates")
        .and_then(Value::as_array)
        .and_then(|candidates| candidates.first())
        .and_then(|candidate| candidate.pointer("/content/parts"))
        .and_then(Value::as_array)
        .map(|parts| parts.iter().collect())
        .unwrap_or_default()
}

fn required_string<'a>(object: &'a Map<String, Value>, key: &str) -> AppResult<&'a str> {
    object
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| AppError::BadRequest(format!("{key} is required")))
}

fn validate_tool_name(name: &str) -> AppResult<()> {
    if name.len() > MAX_TOOL_NAME_LEN {
        return Err(AppError::BadRequest(
            "tool name exceeds 64 characters".into(),
        ));
    }
    Ok(())
}

fn parse_json_object_string(value: Option<&Value>, label: &str) -> AppResult<Value> {
    match value {
        Some(Value::String(text)) if !text.trim().is_empty() => serde_json::from_str(text)
            .map_err(|err| AppError::BadRequest(format!("{label} must be valid JSON: {err}"))),
        Some(Value::Object(_)) => Ok(value.cloned().unwrap_or_else(|| json!({}))),
        Some(Value::Null) | None => Ok(json!({})),
        _ => Err(AppError::BadRequest(format!(
            "{label} must be a JSON object string"
        ))),
    }
}

fn parse_function_call_output(value: Option<&Value>) -> AppResult<Value> {
    match value {
        Some(Value::String(text)) => serde_json::from_str(text)
            .or_else(|_| Ok::<Value, serde_json::Error>(json!({ "output": text })))
            .map_err(AppError::Json),
        Some(Value::Object(_)) => Ok(value.cloned().unwrap_or_else(|| json!({}))),
        Some(Value::Null) | None => Ok(json!({})),
        Some(other) => Ok(json!({ "output": other })),
    }
}

fn google_custom_tool_input(args: Option<&Value>) -> String {
    match args {
        Some(Value::Object(object)) => object
            .get("input")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| compact_json(args.unwrap_or(&json!({})))),
        Some(Value::String(text)) => text.clone(),
        Some(value) => compact_json(value),
        None => String::new(),
    }
}

fn google_finish_reason(body: &Value) -> Option<&str> {
    body.get("candidates")
        .and_then(Value::as_array)
        .and_then(|candidates| candidates.first())
        .and_then(|candidate| candidate.get("finishReason"))
        .and_then(Value::as_str)
}

fn sse_event(event: &str, data: Value) -> String {
    format!("event: {event}\ndata: {}\n\n", compact_json(&data))
}

fn compact_json(value: &Value) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| json!({}).to_string())
}

fn collect_sse_data(frame: &str) -> String {
    let mut data_lines = Vec::new();
    for line in frame.lines() {
        let line = line.trim_end_matches('\r');
        if let Some(data) = line.strip_prefix("data:") {
            data_lines.push(data.trim_start());
        }
    }
    data_lines.join("\n")
}

fn find_sse_frame_end(buffer: &str) -> Option<usize> {
    buffer.find("\n\n").or_else(|| buffer.find("\r\n\r\n"))
}
