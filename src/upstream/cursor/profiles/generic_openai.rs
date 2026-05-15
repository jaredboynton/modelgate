//! GenericOpenAi profile renderer.
//!
//! Preserves the current Cursor proxy public-tool-call behavior: lowercase
//! OpenAI-conventional names (read, ls, grep, shell, write, delete, etc.).
//! This is the residual default profile when no client signature is
//! detected and the compatibility profile for curl/OpenAI SDK consumers.

use super::RenderedToolCall;
use crate::upstream::cursor::proto::{decode_exec_public_tool_call, ExecRequest};

/// Render a Cursor `ExecRequest` as a GenericOpenAi public tool call.
///
/// Wraps the existing `decode_exec_public_tool_call` baseline so the
/// lowercase Cursor public-tool-call names stay byte-identical to the
/// pre-profile-dispatch behavior. GenericOpenAi never refuses; unknown
/// exec kinds collapse to `cursor_exec` with `field_<n>`-keyed args via
/// the proto decoder's existing fallback.
pub fn render(exec: &ExecRequest) -> RenderedToolCall {
    let (tool_name, tool_call_id, arguments) = decode_exec_public_tool_call(exec);
    RenderedToolCall::Emit {
        tool_name,
        arguments,
        tool_call_id,
    }
}
