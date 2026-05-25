use std::time::{SystemTime, UNIX_EPOCH};

use bytes::Bytes;
use serde_json::{json, Map, Value};
use uuid::Uuid;

use crate::{AppError, AppResult};

#[derive(Debug, Clone)]
pub struct PriorWindsurfResponse {
    pub raw_response: Value,
    pub raw_input_items: Value,
}

pub fn build_chat_request(
    value: &Value,
    upstream_model: &str,
    prior: Option<&PriorWindsurfResponse>,
) -> AppResult<(Value, Value)> {
    let object = value
        .as_object()
        .ok_or_else(|| AppError::BadRequest("request body must be a JSON object".into()))?;

    enforce_supported_fields(object)?;

    let mut messages = Vec::new();
    if let Some(instructions) = object.get("instructions").and_then(Value::as_str) {
        if !instructions.trim().is_empty() {
            messages.push(json!({ "role": "system", "content": instructions }));
        }
    }
    if let Some(prior) = prior {
        messages.extend(input_to_chat_messages(&prior.raw_input_items)?);
        messages.extend(output_to_chat_messages(prior.raw_response.get("output"))?);
    }
    let raw_input_items = object.get("input").cloned().unwrap_or(Value::Null);
    messages.extend(input_to_chat_messages(&raw_input_items)?);
    if messages.is_empty() {
        return Err(AppError::BadRequest("missing input".into()));
    }

    let mut out = Map::new();
    out.insert("model".into(), Value::String(upstream_model.to_string()));
    out.insert("messages".into(), Value::Array(messages));
    if let Some(stream) = object.get("stream").and_then(Value::as_bool) {
        out.insert("stream".into(), Value::Bool(stream));
    }
    if let Some(tools) = object.get("tools") {
        out.insert("tools".into(), convert_tools(tools)?);
    }
    if let Some(tool_choice) = object.get("tool_choice") {
        out.insert("tool_choice".into(), tool_choice.clone());
    }
    Ok((Value::Object(out), raw_input_items))
}

pub fn tool_result_call_ids(value: &Value) -> AppResult<Vec<String>> {
    let mut out = Vec::new();
    collect_tool_result_call_ids(value.get("input"), &mut out)?;
    Ok(out)
}

pub fn response_function_call_ids(response: &Value) -> Vec<String> {
    response
        .get("output")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter(|item| item.get("type").and_then(Value::as_str) == Some("function_call"))
        .filter_map(|item| item.get("call_id").and_then(Value::as_str))
        .map(ToOwned::to_owned)
        .collect()
}

pub fn response_from_text_with_id(
    response_id: &str,
    model: &str,
    content: impl Into<String>,
) -> Value {
    let content = content.into();
    json!({
        "id": response_id,
        "object": "response",
        "created_at": created_timestamp(),
        "model": model,
        "status": "completed",
        "output": [{
            "type": "message",
            "id": format!("msg_{}", Uuid::new_v4().simple()),
            "status": "completed",
            "role": "assistant",
            "content": [{
                "type": "output_text",
                "text": content,
                "annotations": []
            }]
        }],
        "usage": zero_usage()
    })
}

pub fn response_from_text(model: &str, content: impl Into<String>) -> Value {
    response_from_text_with_id(&response_id(), model, content)
}

pub fn response_from_tool_calls(
    model: &str,
    calls: &[crate::adapter::windsurf_chat::ToolCallPlan],
) -> Value {
    json!({
        "id": response_id(),
        "object": "response",
        "created_at": created_timestamp(),
        "model": model,
        "status": "completed",
        "output": calls.iter().map(|call| {
            json!({
                "type": "function_call",
                "id": format!("fc_{}", Uuid::new_v4().simple()),
                "call_id": format!("call_{}", Uuid::new_v4().simple()),
                "name": call.name,
                "arguments": stringify_arguments(&call.arguments),
                "status": "completed"
            })
        }).collect::<Vec<_>>(),
        "usage": zero_usage()
    })
}

pub fn text_stream_start(response_id: &str, model: &str) -> Bytes {
    let response = json!({
        "id": response_id,
        "object": "response",
        "created_at": created_timestamp(),
        "model": model,
        "status": "in_progress",
        "output": [],
        "usage": null
    });
    let item_id = message_id(response_id);
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&sse_event(
        "response.created",
        &json!({
            "type": "response.created",
            "response": response
        }),
    ));
    bytes.extend_from_slice(&sse_event(
        "response.output_item.added",
        &json!({
            "type": "response.output_item.added",
            "output_index": 0,
            "item": {
                "type": "message",
                "id": item_id,
                "status": "in_progress",
                "role": "assistant",
                "content": []
            }
        }),
    ));
    bytes.extend_from_slice(&sse_event(
        "response.content_part.added",
        &json!({
            "type": "response.content_part.added",
            "item_id": item_id,
            "output_index": 0,
            "content_index": 0,
            "part": { "type": "output_text", "text": "", "annotations": [] }
        }),
    ));
    Bytes::from(bytes)
}

pub fn text_delta_frame(delta: &str) -> Bytes {
    sse_event(
        "response.output_text.delta",
        &json!({
            "type": "response.output_text.delta",
            "output_index": 0,
            "content_index": 0,
            "delta": delta
        }),
    )
}

pub fn text_stream_finish(response: &Value) -> Bytes {
    let response_id = response
        .get("id")
        .and_then(Value::as_str)
        .unwrap_or("resp_windsurf");
    let item_id = message_id(response_id);
    let text = response
        .get("output")
        .and_then(Value::as_array)
        .and_then(|items| items.first())
        .and_then(|item| item.get("content"))
        .and_then(Value::as_array)
        .and_then(|parts| parts.first())
        .and_then(|part| part.get("text"))
        .and_then(Value::as_str)
        .unwrap_or("");
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&sse_event(
        "response.output_text.done",
        &json!({
            "type": "response.output_text.done",
            "output_index": 0,
            "content_index": 0,
            "text": text
        }),
    ));
    bytes.extend_from_slice(&sse_event(
        "response.content_part.done",
        &json!({
            "type": "response.content_part.done",
            "item_id": item_id,
            "output_index": 0,
            "content_index": 0,
            "part": { "type": "output_text", "text": text, "annotations": [] }
        }),
    ));
    bytes.extend_from_slice(&sse_event(
        "response.output_item.done",
        &json!({
            "type": "response.output_item.done",
            "output_index": 0,
            "item": response["output"][0]
        }),
    ));
    bytes.extend_from_slice(&sse_event(
        "response.completed",
        &json!({
            "type": "response.completed",
            "response": response
        }),
    ));
    bytes.extend_from_slice(b"data: [DONE]\n\n");
    Bytes::from(bytes)
}

pub fn static_response_sse(response: &Value) -> Bytes {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&sse_event(
        "response.created",
        &json!({
            "type": "response.created",
            "response": {
                "id": response["id"],
                "object": "response",
                "created_at": response["created_at"],
                "model": response["model"],
                "status": "in_progress",
                "output": [],
                "usage": null
            }
        }),
    ));
    bytes.extend_from_slice(&sse_event(
        "response.completed",
        &json!({
            "type": "response.completed",
            "response": response
        }),
    ));
    bytes.extend_from_slice(b"data: [DONE]\n\n");
    Bytes::from(bytes)
}

pub fn error_stream_frame(error: &AppError) -> Bytes {
    let mut bytes = sse_event(
        "error",
        &json!({
            "type": "error",
            "error": {
                "message": error.to_string(),
                "type": error.error_type(),
                "code": error.code()
            }
        }),
    )
    .to_vec();
    bytes.extend_from_slice(b"data: [DONE]\n\n");
    Bytes::from(bytes)
}

pub fn response_id() -> String {
    format!("resp_{}", Uuid::new_v4().simple())
}

fn input_to_chat_messages(value: &Value) -> AppResult<Vec<Value>> {
    match value {
        Value::Null => Ok(Vec::new()),
        Value::String(text) => Ok(vec![json!({ "role": "user", "content": text })]),
        Value::Array(items) => {
            let mut out = Vec::new();
            for item in items {
                match item {
                    Value::String(text) => out.push(json!({ "role": "user", "content": text })),
                    Value::Object(object) => match object.get("type").and_then(Value::as_str) {
                        Some("message") => {
                            let role = object.get("role").and_then(Value::as_str).unwrap_or("user");
                            out.push(json!({
                                "role": role,
                                "content": response_text_content(object.get("content"))?
                            }));
                        }
                        Some("input_text") => {
                            out.push(json!({
                                "role": "user",
                                "content": object.get("text").and_then(Value::as_str).unwrap_or("")
                            }));
                        }
                        Some("function_call") => {
                            out.push(chat_message_from_function_call(item)?);
                        }
                        Some("function_call_output" | "custom_tool_call_output") => {
                            let call_id = object
                                .get("call_id")
                                .and_then(Value::as_str)
                                .ok_or_else(|| {
                                    AppError::BadRequest(
                                        "function_call_output missing call_id".into(),
                                    )
                                })?;
                            out.push(json!({
                                "role": "tool",
                                "tool_call_id": call_id,
                                "content": function_output_text(object.get("output"))?
                            }));
                        }
                        Some(other) => {
                            return Err(AppError::BadRequest(format!(
                                "unsupported Windsurf Responses input item type {other}"
                            )));
                        }
                        None => {
                            if let Some(text) = object.get("text").and_then(Value::as_str) {
                                out.push(json!({ "role": "user", "content": text }));
                            } else {
                                return Err(AppError::BadRequest(
                                    "unsupported Windsurf Responses input item".into(),
                                ));
                            }
                        }
                    },
                    _ => {
                        return Err(AppError::BadRequest(
                            "unsupported Windsurf Responses input item".into(),
                        ));
                    }
                }
            }
            Ok(out)
        }
        _ => Err(AppError::BadRequest(
            "Windsurf Responses input must be a string or array".into(),
        )),
    }
}

fn output_to_chat_messages(value: Option<&Value>) -> AppResult<Vec<Value>> {
    let mut out = Vec::new();
    for item in value.and_then(Value::as_array).into_iter().flatten() {
        if item.get("type").and_then(Value::as_str) == Some("function_call") {
            out.push(chat_message_from_function_call(item)?);
        } else if item.get("type").and_then(Value::as_str) == Some("message") {
            let content = response_text_content(item.get("content"))?;
            if !content.is_empty() {
                out.push(json!({ "role": "assistant", "content": content }));
            }
        }
    }
    Ok(out)
}

fn chat_message_from_function_call(item: &Value) -> AppResult<Value> {
    let name = item
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| AppError::BadRequest("function_call missing name".into()))?;
    let call_id = item
        .get("call_id")
        .or_else(|| item.get("id"))
        .and_then(Value::as_str)
        .ok_or_else(|| AppError::BadRequest("function_call missing call_id".into()))?;
    let arguments = item
        .get("arguments")
        .and_then(Value::as_str)
        .unwrap_or("{}");
    Ok(json!({
        "role": "assistant",
        "content": null,
        "tool_calls": [{
            "id": call_id,
            "type": "function",
            "function": { "name": name, "arguments": arguments }
        }]
    }))
}

fn collect_tool_result_call_ids(value: Option<&Value>, out: &mut Vec<String>) -> AppResult<()> {
    if let Some(Value::Array(items)) = value {
        for item in items {
            if item.get("type").and_then(Value::as_str) == Some("function_call_output")
                || item.get("type").and_then(Value::as_str) == Some("custom_tool_call_output")
            {
                let call_id = item.get("call_id").and_then(Value::as_str).ok_or_else(|| {
                    AppError::BadRequest("function_call_output missing call_id".into())
                })?;
                out.push(call_id.to_string());
            }
        }
    }
    Ok(())
}

fn convert_tools(value: &Value) -> AppResult<Value> {
    let tools = value
        .as_array()
        .ok_or_else(|| AppError::BadRequest("tools must be an array".into()))?;
    Ok(Value::Array(
        tools
            .iter()
            .map(|tool| {
                if tool.get("function").is_some() {
                    return tool.clone();
                }
                json!({
                    "type": "function",
                    "function": {
                        "name": tool.get("name").cloned().unwrap_or(Value::String("unknown".into())),
                        "description": tool.get("description").cloned().unwrap_or(Value::Null),
                        "parameters": tool.get("parameters").cloned().unwrap_or(json!({}))
                    }
                })
            })
            .collect(),
    ))
}

fn response_text_content(value: Option<&Value>) -> AppResult<String> {
    match value {
        Some(Value::String(text)) => Ok(text.clone()),
        Some(Value::Array(parts)) => Ok(parts
            .iter()
            .filter_map(|part| match part.get("type").and_then(Value::as_str) {
                Some("input_text" | "output_text" | "text") => {
                    part.get("text").and_then(Value::as_str)
                }
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("\n")),
        Some(Value::Null) | None => Ok(String::new()),
        _ => Err(AppError::BadRequest(
            "Windsurf Responses message content must be text".into(),
        )),
    }
}

fn function_output_text(value: Option<&Value>) -> AppResult<String> {
    match value {
        Some(Value::String(text)) => Ok(text.clone()),
        Some(other) => Ok(other.to_string()),
        None => Err(AppError::BadRequest(
            "function_call_output missing output".into(),
        )),
    }
}

fn enforce_supported_fields(object: &Map<String, Value>) -> AppResult<()> {
    for key in object.keys() {
        if !matches!(
            key.as_str(),
            "model"
                | "input"
                | "instructions"
                | "stream"
                | "tools"
                | "tool_choice"
                | "max_tool_calls"
                | "parallel_tool_calls"
                | "previous_response_id"
                | "store"
                | "metadata"
                | "user"
        ) {
            return Err(AppError::BadRequest(format!(
                "field {key} is not supported for Windsurf responses"
            )));
        }
    }
    Ok(())
}

fn created_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default()
}

fn zero_usage() -> Value {
    json!({
        "input_tokens": 0,
        "input_tokens_details": null,
        "output_tokens": 0,
        "output_tokens_details": null,
        "total_tokens": 0
    })
}

fn stringify_arguments(value: &Value) -> String {
    match value {
        Value::String(text) => text.clone(),
        other => other.to_string(),
    }
}

fn message_id(response_id: &str) -> String {
    format!("msg_{}", response_id.trim_start_matches("resp_"))
}

fn sse_event(event: &str, data: &Value) -> Bytes {
    Bytes::from(format!("event: {event}\ndata: {data}\n\n"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn responses_input_and_prior_output_reconstruct_chat_tool_loop() {
        let prior = PriorWindsurfResponse {
            raw_response: json!({
                "id": "resp_prior",
                "output": [{
                    "type": "function_call",
                    "call_id": "call_lookup",
                    "name": "lookup",
                    "arguments": "{\"q\":\"x\"}"
                }]
            }),
            raw_input_items: json!("find x"),
        };
        let (chat, raw) = build_chat_request(
            &json!({
                "model": "swe-1.6",
                "previous_response_id": "resp_prior",
                "input": [{ "type": "function_call_output", "call_id": "call_lookup", "output": "found" }]
            }),
            "swe-1-6",
            Some(&prior),
        )
        .unwrap();

        assert_eq!(raw[0]["call_id"], "call_lookup");
        let messages = chat["messages"].as_array().unwrap();
        assert_eq!(messages[0]["role"], "user");
        assert_eq!(messages[1]["tool_calls"][0]["id"], "call_lookup");
        assert_eq!(messages[2]["role"], "tool");
        assert!(
            chat.get("tools").is_none(),
            "tool-result continuation must not re-expose prior tools unless the request supplies tools"
        );
    }

    #[test]
    fn responses_tool_result_ids_are_extracted_for_policy_checks() {
        let ids = tool_result_call_ids(&json!({
            "input": [
                { "type": "function_call_output", "call_id": "call_1", "output": "ok" },
                { "type": "message", "role": "user", "content": [{ "type": "input_text", "text": "next" }] }
            ]
        }))
        .unwrap();

        assert_eq!(ids, vec!["call_1"]);
    }
}
