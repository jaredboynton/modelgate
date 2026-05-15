//! Shared helpers used by the public-shape Cursor adapters
//! (`cursor_responses.rs`, `cursor_chat.rs`, `cursor_messages.rs`).
//!
//! This module owns the small bookkeeping pieces every Cursor adapter needs:
//! deterministic public-shape IDs, output-index allocation, tool-kind
//! tracking, finish-reason mapping, and usage envelope construction. It does
//! not call into `crate::route`, `crate::upstream`, `crate::auth`,
//! `crate::router`, or `crate::state`. It depends only on
//! `crate::cursor_agent::*` DTOs and standard JSON.
//!
//! All three public adapters share these helpers so naming, sequencing, and
//! error envelopes stay consistent across `/v1/responses`,
//! `/v1/chat/completions`, and `/v1/messages`.
//!
//! NOTE: anti-patterns from `responses-events-extraction.md` are deliberately
//! avoided here. We do not borrow Anthropic's reasoning translation,
//! Anthropic's `rewrite_max_alias` family, Google's thought-signature
//! plumbing, Google's `candidates[0]` finishReason detection, or v1's
//! Codex-style allowlist.

use std::collections::HashMap;

use serde_json::{json, Value};

use crate::cursor_agent::{
    CursorAgentRequest, CursorContinuationKey, CursorFinishReason, CursorMessage, CursorRoute,
    CursorTool, CursorToolKind,
};
use crate::model_alias::{Provider, TargetFormat};

/// One on-the-wire SSE frame (`event:` line + `data:` JSON).
///
/// Adapters return a `Vec<ResponsesSseEvent>` per `CursorAgentEvent` so the
/// route layer can serialize them to the SSE wire shape with consistent
/// formatting.
#[derive(Debug, Clone)]
pub struct ResponsesSseEvent {
    pub event: String,
    pub data: Value,
}

impl ResponsesSseEvent {
    pub fn new(event: impl Into<String>, data: Value) -> Self {
        Self {
            event: event.into(),
            data,
        }
    }

    /// Serialize the frame using the on-the-wire shape used by every
    /// existing Responses adapter:
    /// `event: <event-name>\ndata: <single-line compact JSON>\n\n`.
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

/// Stable bookkeeping the Cursor public adapters share.
///
/// One `ResponseContext` is created per request. It tracks which output items
/// have been opened on the Responses side (so deltas land on the right
/// `output_index`) and which tools were declared as Function vs Custom (so
/// tool-call frames are routed to the correct event family).
#[derive(Debug, Clone)]
pub struct ResponseContext {
    pub model: String,
    pub response_id: String,
    pub conversation_id: Option<String>,
    pub started: bool,
    pub completed: bool,
    pub failed: bool,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub reasoning_tokens: u64,
    pub finish_reason: Option<CursorFinishReason>,
    /// Tool definitions declared on the inbound request, keyed by name. Used
    /// to decide whether to emit `function_call_arguments.*` or
    /// `custom_tool_call_input.*` for a given tool call.
    tool_kinds: HashMap<String, CursorToolKind>,
    /// Open tool-call buffers, keyed by Cursor exec id (`call_id`).
    tool_calls: HashMap<String, ToolCallState>,
    /// Open text item, if any. Cursor adapters only ever maintain a single
    /// text channel, matching the existing first-party adapters.
    text_state: Option<TextItemState>,
    /// Open reasoning item, if any. Tracks the running summary text so the
    /// matching `output_item.done` can carry the full string.
    reasoning_state: Option<ReasoningItemState>,
    /// Monotonic next index used when opening a new output item.
    next_index: u32,
    /// Ordered list of output items that have been closed via
    /// `output_item.done`. The non-stream collapse uses this verbatim.
    completed_items: Vec<Value>,
}

#[derive(Debug, Clone)]
pub struct TextItemState {
    pub output_index: u32,
    pub item_id: String,
    pub text: String,
}

#[derive(Debug, Clone)]
pub struct ReasoningItemState {
    pub output_index: u32,
    pub item_id: String,
    pub summary_index: u32,
    pub summary_text: String,
    pub summary_part_open: bool,
}

#[derive(Debug, Clone)]
pub struct ToolCallState {
    pub output_index: u32,
    pub item_id: String,
    pub call_id: String,
    pub name: String,
    pub kind: CursorToolKind,
    pub arguments_buffer: String,
}

impl ResponseContext {
    pub fn new(model: impl Into<String>, response_id: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            response_id: response_id.into(),
            conversation_id: None,
            started: false,
            completed: false,
            failed: false,
            input_tokens: 0,
            output_tokens: 0,
            reasoning_tokens: 0,
            finish_reason: None,
            tool_kinds: HashMap::new(),
            tool_calls: HashMap::new(),
            text_state: None,
            reasoning_state: None,
            next_index: 0,
            completed_items: Vec::new(),
        }
    }

    pub fn record_tool_kind(&mut self, name: impl Into<String>, kind: CursorToolKind) {
        self.tool_kinds.insert(name.into(), kind);
    }

    pub fn tool_kind(&self, name: &str) -> CursorToolKind {
        self.tool_kinds
            .get(name)
            .copied()
            .unwrap_or(CursorToolKind::Function)
    }

    pub fn allocate_index(&mut self) -> u32 {
        let index = self.next_index;
        self.next_index = self.next_index.saturating_add(1);
        index
    }

    pub fn open_text_item(&mut self) -> &mut TextItemState {
        if self.text_state.is_none() {
            let output_index = self.allocate_index();
            self.text_state = Some(TextItemState {
                output_index,
                item_id: format!("msg_{output_index}"),
                text: String::new(),
            });
        }
        self.text_state.as_mut().expect("text item just opened")
    }

    pub fn current_text_index(&self) -> Option<u32> {
        self.text_state.as_ref().map(|state| state.output_index)
    }

    pub fn close_text_item(&mut self) -> Option<(u32, String, String)> {
        let state = self.text_state.take()?;
        let item = json!({
            "id": state.item_id.clone(),
            "type": "message",
            "status": "completed",
            "role": "assistant",
            "content": [{ "type": "output_text", "text": state.text.clone() }],
        });
        self.completed_items.push(item);
        Some((state.output_index, state.item_id, state.text))
    }

    pub fn open_reasoning_item(&mut self) -> &mut ReasoningItemState {
        if self.reasoning_state.is_none() {
            let output_index = self.allocate_index();
            self.reasoning_state = Some(ReasoningItemState {
                output_index,
                item_id: format!("rs_{output_index}"),
                summary_index: 0,
                summary_text: String::new(),
                summary_part_open: false,
            });
        }
        self.reasoning_state
            .as_mut()
            .expect("reasoning item just opened")
    }

    pub fn close_reasoning_item(&mut self) -> Option<(u32, String, u32, String)> {
        let state = self.reasoning_state.take()?;
        let item = json!({
            "id": state.item_id.clone(),
            "type": "reasoning",
            "status": "completed",
            "summary": [{
                "type": "summary_text",
                "text": state.summary_text.clone(),
            }],
        });
        self.completed_items.push(item);
        Some((
            state.output_index,
            state.item_id,
            state.summary_index,
            state.summary_text,
        ))
    }

    pub fn open_tool_call(
        &mut self,
        call_id: impl Into<String>,
        name: impl Into<String>,
        kind: CursorToolKind,
    ) -> &ToolCallState {
        let call_id = call_id.into();
        let name = name.into();
        if !self.tool_calls.contains_key(&call_id) {
            let output_index = self.allocate_index();
            let item_id = match kind {
                CursorToolKind::Function => format!("fc_{output_index}"),
                CursorToolKind::Custom => format!("ctc_{output_index}"),
            };
            self.tool_calls.insert(
                call_id.clone(),
                ToolCallState {
                    output_index,
                    item_id,
                    call_id: call_id.clone(),
                    name,
                    kind,
                    arguments_buffer: String::new(),
                },
            );
        }
        self.tool_calls
            .get(&call_id)
            .expect("tool call just inserted")
    }

    pub fn append_tool_arguments(&mut self, call_id: &str, fragment: &str) {
        if let Some(state) = self.tool_calls.get_mut(call_id) {
            state.arguments_buffer.push_str(fragment);
        }
    }

    pub fn close_tool_call(&mut self, call_id: &str) -> Option<ToolCallSnapshot> {
        let state = self.tool_calls.remove(call_id)?;
        let final_arguments = state.arguments_buffer.clone();
        let item = match state.kind {
            CursorToolKind::Function => json!({
                "id": state.item_id.clone(),
                "type": "function_call",
                "status": "completed",
                "call_id": state.call_id.clone(),
                "name": state.name.clone(),
                "arguments": final_arguments.clone(),
            }),
            CursorToolKind::Custom => json!({
                "id": state.item_id.clone(),
                "type": "custom_tool_call",
                "status": "completed",
                "call_id": state.call_id.clone(),
                "name": state.name.clone(),
                "input": final_arguments.clone(),
            }),
        };
        self.completed_items.push(item);
        Some(ToolCallSnapshot {
            output_index: state.output_index,
            item_id: state.item_id,
            call_id: state.call_id,
            name: state.name,
            kind: state.kind,
            final_arguments,
        })
    }

    pub fn tool_call_index(&self, call_id: &str) -> Option<u32> {
        self.tool_calls.get(call_id).map(|state| state.output_index)
    }

    pub fn tool_call_kind(&self, call_id: &str) -> Option<CursorToolKind> {
        self.tool_calls.get(call_id).map(|state| state.kind)
    }

    pub fn tool_call_item_id(&self, call_id: &str) -> Option<&str> {
        self.tool_calls
            .get(call_id)
            .map(|state| state.item_id.as_str())
    }

    pub fn tool_call_name(&self, call_id: &str) -> Option<&str> {
        self.tool_calls
            .get(call_id)
            .map(|state| state.name.as_str())
    }

    pub fn record_usage(
        &mut self,
        input_tokens: u64,
        output_tokens: u64,
        reasoning_tokens: Option<u64>,
    ) {
        self.input_tokens = input_tokens;
        self.output_tokens = output_tokens;
        if let Some(reasoning) = reasoning_tokens {
            self.reasoning_tokens = reasoning;
        }
    }

    pub fn record_finish(&mut self, finish: CursorFinishReason) {
        self.finish_reason = Some(finish);
    }

    pub fn record_done(&mut self, finish: CursorFinishReason, conversation_id: impl Into<String>) {
        self.record_finish(finish);
        self.conversation_id = Some(conversation_id.into());
    }

    /// Status string for `response.completed.response.status`.
    /// Mirrors the existing first-party adapters: `incomplete` only when the
    /// finish reason is `Length`; everything else maps to `completed`.
    pub fn response_status(&self) -> &'static str {
        if matches!(self.finish_reason, Some(CursorFinishReason::Length)) {
            "incomplete"
        } else {
            "completed"
        }
    }

    /// Final, ordered output items for `response.completed.response.output`.
    pub fn completed_items(&self) -> &[Value] {
        &self.completed_items
    }

    pub fn into_completed_items(self) -> Vec<Value> {
        self.completed_items
    }

    pub fn usage_envelope(&self) -> Value {
        usage_envelope(self.input_tokens, self.output_tokens, self.reasoning_tokens)
    }
}

/// Snapshot returned when a tool call is closed.
pub struct ToolCallSnapshot {
    pub output_index: u32,
    pub item_id: String,
    pub call_id: String,
    pub name: String,
    pub kind: CursorToolKind,
    pub final_arguments: String,
}

/// Public OpenAI Responses usage envelope.
pub fn usage_envelope(input_tokens: u64, output_tokens: u64, reasoning_tokens: u64) -> Value {
    json!({
        "input_tokens": input_tokens,
        "output_tokens": output_tokens,
        "total_tokens": input_tokens.saturating_add(output_tokens),
        "input_tokens_details": { "cached_tokens": 0 },
        "output_tokens_details": { "reasoning_tokens": reasoning_tokens },
    })
}

/// Map a Cursor finish reason to the OpenAI Chat `finish_reason` string.
pub fn chat_finish_reason(reason: CursorFinishReason) -> &'static str {
    match reason {
        CursorFinishReason::Stop => "stop",
        CursorFinishReason::ToolCalls => "tool_calls",
        CursorFinishReason::Length => "length",
        CursorFinishReason::ContentFilter => "content_filter",
        CursorFinishReason::Error => "stop",
    }
}

/// Map a Cursor finish reason to the Anthropic Messages `stop_reason`.
pub fn anthropic_stop_reason(reason: CursorFinishReason) -> &'static str {
    match reason {
        CursorFinishReason::Stop => "end_turn",
        CursorFinishReason::ToolCalls => "tool_use",
        CursorFinishReason::Length => "max_tokens",
        CursorFinishReason::ContentFilter => "stop_sequence",
        CursorFinishReason::Error => "end_turn",
    }
}

/// Build a continuation key for the given route + request fingerprint.
///
/// The route layer enforces drift; this helper exists so the three public
/// adapters produce a key with identical shape and stable-field policy.
pub fn build_continuation_key(
    route: CursorRoute,
    request: &CursorAgentRequest,
    response_id: impl Into<String>,
    conversation_id: impl Into<String>,
    stable_request_fields: Value,
) -> CursorContinuationKey {
    CursorContinuationKey {
        route,
        provider: Provider::Cursor,
        upstream_model: request.upstream_model.clone(),
        target_format: TargetFormat::CursorAgent,
        stable_request_fields,
        response_id: response_id.into(),
        conversation_id: conversation_id.into(),
    }
}

/// Compute the canonical stable-fields value used by every adapter when
/// building the continuation key. The set is fixed: `model`, `tools`
/// (sorted by name), `tool_choice`, `temperature`, `top_p`,
/// `max_output_tokens`, and the system / developer instructions.
///
/// `messages`, `metadata`, and `stream` are intentionally excluded.
pub fn stable_request_fields(request: &CursorAgentRequest, raw_request: &Value) -> Value {
    let mut sorted_tools: Vec<&CursorTool> = request.tools.iter().collect();
    sorted_tools.sort_by(|a, b| a.name.cmp(&b.name));
    let tools_value: Value = sorted_tools
        .iter()
        .map(|tool| {
            json!({
                "name": tool.name,
                "description": tool.description.clone().unwrap_or_default(),
                "parameters_schema": tool.parameters_schema.clone(),
                "kind": match tool.kind {
                    CursorToolKind::Function => "function",
                    CursorToolKind::Custom => "custom",
                },
            })
        })
        .collect();
    let raw = raw_request.as_object();
    let pick = |key: &str| -> Value {
        raw.and_then(|object| object.get(key))
            .cloned()
            .unwrap_or(Value::Null)
    };
    json!({
        "model": request.model.clone(),
        "upstream_model": request.upstream_model.clone(),
        "tools": tools_value,
        "tool_choice": pick("tool_choice"),
        "temperature": pick("temperature"),
        "top_p": pick("top_p"),
        "max_output_tokens": pick("max_output_tokens"),
        "system": request.system_instructions.clone().unwrap_or_default(),
        "developer": request.developer_instructions.clone().unwrap_or_default(),
    })
}

/// Translate a `CursorMessage` enum variant to the `role` string used by
/// public adapters when surfacing prior assistant content.
pub fn message_role_str(message: &CursorMessage) -> &'static str {
    match message {
        CursorMessage::System { .. } => "system",
        CursorMessage::Developer { .. } => "developer",
        CursorMessage::User { .. } => "user",
        CursorMessage::Assistant { .. } => "assistant",
    }
}
