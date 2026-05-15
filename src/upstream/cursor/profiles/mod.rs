//! Per-client tool-call rendering profiles.
//!
//! Dispatches Cursor exec requests into client-specific tool-call envelopes.

use crate::upstream::cursor::client_profile::ClientProfile;
use crate::upstream::cursor::proto::ExecRequest;

pub mod claude_code;
pub mod codex_cli;
pub mod droid;
pub mod generic_anthropic;
pub mod generic_openai;
mod proto_helpers;

/// Per-profile rendering decision for one Cursor exec request.
#[derive(Debug, Clone, PartialEq)]
pub enum RenderedToolCall {
    Emit {
        tool_name: String,
        arguments: serde_json::Value,
        tool_call_id: String,
    },
    Refuse {
        exec_id: String,
        reason: String,
        code: &'static str,
    },
}

/// Refuse code dictionary shared by all profiles.
pub mod refuse_code {
    pub const UNSUPPORTED_EXEC_KIND: &str = "unsupported_exec_kind";
    pub const SHAPE_UNKNOWN_PENDING_LIVE_PHASE0: &str = "shape_unknown_pending_live_phase0";
    pub const MISSING_REQUIRED_FIELD: &str = "missing_required_field";
    pub const CLIENT_CAPABILITY_UNSUPPORTED: &str = "client_capability_unsupported";
}

/// Dispatch a Cursor ExecRequest to the per-profile renderer.
pub fn render_tool_call(profile: ClientProfile, exec: &ExecRequest) -> RenderedToolCall {
    match profile {
        ClientProfile::CodexCli => codex_cli::render(exec),
        ClientProfile::ClaudeCode => claude_code::render(exec),
        ClientProfile::Droid => droid::render(exec),
        ClientProfile::GenericAnthropic => generic_anthropic::render(exec),
        ClientProfile::GenericOpenAi => generic_openai::render(exec),
    }
}
