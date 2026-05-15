//! Per-client tool-call rendering profiles.
//!
//! Cursor `ExecRequest` events translate into public `tool_call` envelopes.
//! Each public client (Codex CLI, Claude Code, Droid, generic Anthropic SDK,
//! generic OpenAI SDK) has different built-in tool names and arg shapes.
//! `RenderedToolCall` is the per-profile output; `render_tool_call` is the
//! dispatcher.

use crate::upstream::cursor::client_profile::ClientProfile;
use crate::upstream::cursor::proto::ExecRequest;

pub mod claude_code;
pub mod codex_cli;
pub mod droid;
pub mod generic_openai;
// Lanes 7/8/9 land these modules; until their files are on disk, the
// dispatch falls through to GenericOpenAi with a WARN per the migration-
// window telemetry contract in `client-profile-policy.md`.
// pub mod generic_anthropic;

/// Per-profile rendering decision for a single Cursor `ExecRequest`.
///
/// The run engine receives this from the dispatcher and either surfaces a
/// public tool-call event (`Emit`) or routes a structured provider error
/// through the public adapter layer (`Refuse`). Refused execs must not be
/// added to the pending continuation cache.
#[derive(Debug, Clone, PartialEq)]
pub enum RenderedToolCall {
    /// Tool call to surface to the client.
    Emit {
        tool_name: String,
        arguments: serde_json::Value,
        tool_call_id: String,
    },
    /// Refuse to render; emit a structured ProviderError instead.
    Refuse {
        exec_id: String,
        reason: String,
        code: &'static str,
    },
}

/// Refuse code dictionary (per design MAJ-1).
///
/// These four canonical strings cover the Wave 0 refuse surface for every
/// profile. Renderers must not invent profile-specific codes; put profile,
/// exec kind, field name, and emitted tool name in the human `reason` and
/// in structured tracing fields instead.
pub mod refuse_code {
    /// Cursor emitted an exec kind the proxy will not represent for any
    /// public client (e.g. `RecordScreen`, `ComputerUse`, unknown
    /// `Other(N)`).
    pub const UNSUPPORTED_EXEC_KIND: &str = "unsupported_exec_kind";
    /// Exec kind may be supportable but the Cursor protobuf field schema is
    /// not captured yet (e.g. `Diagnostics` pending Live Phase 0 capture).
    pub const SHAPE_UNKNOWN_PENDING_LIVE_PHASE0: &str = "shape_unknown_pending_live_phase0";
    /// The chosen client profile requires an argument absent from the
    /// decoded Cursor wire (e.g. ClaudeCode `Write.content`).
    pub const MISSING_REQUIRED_FIELD: &str = "missing_required_field";
    /// The client profile cannot represent the operation at all (e.g. Droid
    /// `WriteShellStdin`, Claude Code stdin write).
    pub const CLIENT_CAPABILITY_UNSUPPORTED: &str = "client_capability_unsupported";
}

/// Dispatch a Cursor ExecRequest to the per-profile renderer.
///
/// During the migration window, profiles other than GenericOpenAi may not
/// have their renderer module yet. Those fall through to GenericOpenAi with
/// a WARN log emitting `client.profile.unimplemented = "<profile>"`. The
/// field is removed once all profile renderers ship.
pub fn render_tool_call(profile: ClientProfile, exec: &ExecRequest) -> RenderedToolCall {
    match profile {
        ClientProfile::CodexCli => codex_cli::render(exec),
        ClientProfile::ClaudeCode => claude_code::render(exec),
        ClientProfile::Droid => droid::render(exec),
        ClientProfile::GenericOpenAi => generic_openai::render(exec),
        // ClaudeCode, Droid, GenericAnthropic land in Lanes 7/8/9. Until
        // their modules are on disk, they fall through to GenericOpenAi
        // with a WARN per the migration-window telemetry contract.
        other => {
            tracing::warn!(
                target: "cursor.profiles",
                client_profile_unimplemented = %other.as_str(),
                client_profile = "generic_openai",
                "fallthrough to GenericOpenAi until profile renderer ships",
            );
            generic_openai::render(exec)
        }
    }
}
