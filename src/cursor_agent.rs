//! Neutral Cursor Agent DTO boundary.
//!
//! This module is the only shared data boundary between `src/adapter/cursor_*`
//! and `src/upstream/cursor/**`. It owns plain request, event, continuation,
//! and model-catalog types. It contains no business logic, no provider I/O,
//! no route policy, no auth, no state, and no adapter code.
//!
//! Forbidden imports (enforced by `tests/architecture_boundaries.rs`):
//! `crate::route`, `crate::upstream`, `crate::adapter`, `crate::auth`,
//! `crate::router`, `crate::state`. Provider clients (`axum`, `reqwest`,
//! `h2`, `rustls`, etc.) are also forbidden here. Adapters convert the
//! public JSON wire to these DTOs; the upstream layer converts these DTOs
//! to and from the Cursor protobuf wire.
//!
//! Raw provider payloads (e.g. `bytes::Bytes` of protobuf frames) are
//! intentionally not exposed here; they belong inside `src/upstream/cursor`.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::model_alias::{Provider, TargetFormat};

// ---------------------------------------------------------------------------
// Request side
// ---------------------------------------------------------------------------

/// Provider-neutral Cursor request the upstream layer consumes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CursorAgentRequest {
    /// Public model slug, e.g. `composer-1.5`, `composer-2`, `composer-2-fast`.
    pub model: String,
    /// Resolved upstream model identifier, sourced from the model catalog.
    pub upstream_model: String,
    /// Optional system instructions (top of conversation).
    pub system_instructions: Option<String>,
    /// Optional developer instructions (Responses-style developer role).
    pub developer_instructions: Option<String>,
    /// Normalized conversation messages.
    pub messages: Vec<CursorMessage>,
    /// Normalized tool definitions.
    pub tools: Vec<CursorTool>,
    /// Tool results carried into a continuation turn.
    pub tool_results: Vec<CursorToolResult>,
    /// Optional continuation key bound to a prior response.
    pub continuation_key: Option<CursorContinuationKey>,
    /// Optional workspace context (root, branch, allowlist, etc.).
    pub workspace: Option<CursorWorkspaceContext>,
    /// Whether the caller asked for a streamed response.
    pub stream: bool,
    /// Per-request identifier; surfaces as `x-request-id` upstream.
    pub request_id: Uuid,
    /// Detected client family used by the run engine to pick a per-profile
    /// tool-call renderer. Defaults to `GenericOpenAi` so the public
    /// lowercase tool-name baseline is preserved when a route does not
    /// supply a detection result.
    #[serde(default)]
    pub client_profile: CursorClientProfile,
}

/// Local DTO mirror of `crate::upstream::cursor::client_profile::ClientProfile`.
///
/// Lives here because `src/cursor_agent.rs` is a pure DTO boundary and is
/// forbidden from importing the upstream layer (see
/// `tests/architecture_boundaries.rs`). Conversion `From` impls live next
/// to the upstream enum so the upstream side owns the mapping. The variants
/// must stay in lockstep with the upstream enum.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum CursorClientProfile {
    CodexCli,
    ClaudeCode,
    Droid,
    GenericAnthropic,
    #[default]
    GenericOpenAi,
}

/// Normalized conversation message.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "role", rename_all = "snake_case")]
pub enum CursorMessage {
    System {
        content: String,
    },
    Developer {
        content: String,
    },
    User {
        blocks: Vec<CursorContentBlock>,
    },
    Assistant {
        blocks: Vec<CursorContentBlock>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        tool_calls: Vec<CursorToolCall>,
    },
}

/// Content block inside a user or assistant message.
///
/// Only text blocks are accepted. Image blocks must be rejected with an
/// explicit unsupported error at the adapter layer (see ralplan Section 7).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CursorContentBlock {
    Text(String),
}

/// Normalized tool definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CursorTool {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// JSON Schema for tool parameters; preserved verbatim.
    pub parameters_schema: serde_json::Value,
    pub kind: CursorToolKind,
}

/// Tool kind.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum CursorToolKind {
    /// Standard function tool (Chat Completions and Responses).
    Function,
    /// Responses-only custom tool used for the
    /// `response.custom_tool_call_input.delta` stream path.
    Custom,
}

/// A tool call emitted by the assistant turn.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CursorToolCall {
    /// Stable identifier (`call_<uuid>`); preserved across continuation turns.
    pub id: String,
    pub name: String,
    /// Final flushed arguments (after all `arguments.delta` events).
    pub arguments: serde_json::Value,
}

/// A tool result delivered back into a continuation turn.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CursorToolResult {
    pub call_id: String,
    /// Tool output, preserved verbatim.
    pub output: serde_json::Value,
    /// Tool-side error, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Bound continuation handle for a prior response.
///
/// Field semantics match the existing `previous_response_id` policy in
/// WebSocket storage: route, provider, model, target format, stable request
/// fields, response ID, conversation ID. Drift in any of these is rejected
/// before provider calls (see ralplan Section 4 plan item 12).
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct CursorContinuationKey {
    pub route: CursorRoute,
    pub provider: Provider,
    pub upstream_model: String,
    pub target_format: TargetFormat,
    /// Canonical hash payload of stable request fields. The full set is:
    /// `model`, `tools` (sorted by name), `tool_choice`, `temperature`,
    /// `top_p`, `max_output_tokens`, `system`, `developer` instructions.
    /// `messages`, `metadata`, and the `stream` flag are intentionally
    /// excluded.
    pub stable_request_fields: serde_json::Value,
    pub response_id: String,
    pub conversation_id: String,
}

impl CursorContinuationKey {
    /// SHA-256 over canonical-JSON serialization of the full key.
    ///
    /// Used as the lookup hash in the upstream session store. Canonical-JSON
    /// here means the `serde_json` serialization of this struct as a whole;
    /// `serde_json::to_vec` orders struct fields by declaration order and
    /// `BTreeMap` keys lexicographically, which is sufficient for stability
    /// because the struct shape is fixed and `stable_request_fields` is
    /// expected to be canonicalized by the producer (sorted tool list,
    /// no insignificant whitespace).
    pub fn canonical_hash(&self) -> [u8; 32] {
        // Serializing `self` directly gives a deterministic byte sequence
        // because struct field order is fixed at declaration time and
        // `serde_json` does not reorder `Value::Object` entries other than
        // preserving insertion order, which the canonicalization upstream
        // is responsible for. If serialization fails (it should not for a
        // well-formed value tree), we fall back to hashing the empty byte
        // slice so callers see a deterministic but clearly-distinct hash.
        let bytes = serde_json::to_vec(self).unwrap_or_default();
        let digest = Sha256::digest(&bytes);
        let mut out = [0u8; 32];
        out.copy_from_slice(&digest);
        out
    }
}

/// Source format / route the continuation belongs to.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum CursorRoute {
    Responses,
    ChatCompletions,
    AnthropicMessages,
}

/// Workspace context passed alongside a request when the route opted into it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CursorWorkspaceContext {
    pub root: PathBuf,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub worktree: Option<PathBuf>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remote: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status_summary: Option<String>,
    /// Index metadata read from `~/.cursor` when available.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub index_metadata: Option<serde_json::Value>,
    /// Allowlist sourced from `UMP_CURSOR_WORKSPACE_ALLOWLIST`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub allowlist: Vec<PathBuf>,
}

// ---------------------------------------------------------------------------
// Event side
// ---------------------------------------------------------------------------

/// Provider-neutral event emitted by the Cursor upstream stream.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CursorAgentEvent {
    TextDelta {
        delta: String,
        content_index: u32,
    },
    /// Composer 2-family thinking stream.
    ReasoningDelta {
        delta: String,
    },
    ToolCallStarted {
        call_id: String,
        name: String,
        kind: CursorToolKind,
        argument_index: u32,
    },
    /// Partial JSON argument fragment.
    ToolCallArgumentsDelta {
        call_id: String,
        delta: String,
    },
    ToolCallDone {
        call_id: String,
        arguments: serde_json::Value,
    },
    UsageUpdate {
        input_tokens: u64,
        output_tokens: u64,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        reasoning_tokens: Option<u64>,
    },
    /// Opaque checkpoint token; the upstream session store keys blob store
    /// and pending tool calls by this ID.
    Checkpoint {
        checkpoint_id: String,
    },
    ProviderError {
        code: String,
        message: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        cursor_request_id: Option<String>,
    },
    Done {
        finish_reason: CursorFinishReason,
        response_id: String,
        conversation_id: String,
    },
}

/// Terminal reason carried on `Done`.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum CursorFinishReason {
    Stop,
    ToolCalls,
    Length,
    ContentFilter,
    Error,
}

// ---------------------------------------------------------------------------
// Model catalog DTO
// ---------------------------------------------------------------------------

/// Catalog descriptor for a Cursor-served model.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct CursorModelDescriptor {
    /// Public ID, e.g. `composer-2`, `composer-2-fast`, `composer-1.5`.
    pub id: String,
    /// Upstream identifier; currently equal to `id` but kept distinct to
    /// allow a future translation table.
    pub upstream_id: String,
    pub discovery: CursorDiscoverySource,
    pub context_window: u32,
    pub max_output_tokens: u32,
    /// True for `composer-2` and `composer-2-fast`.
    pub supports_reasoning: bool,
}

/// Where the descriptor was sourced from.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum CursorDiscoverySource {
    /// Returned by the Cursor live discovery endpoint.
    Live,
    /// Static fallback used when discovery is unavailable.
    Fallback,
}
