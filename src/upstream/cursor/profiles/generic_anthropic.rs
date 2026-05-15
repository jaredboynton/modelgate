//! GenericAnthropic profile renderer.
//!
//! Delegates Anthropic SDK requests to the generic OpenAI-shaped renderer.

use super::{generic_openai, RenderedToolCall};
use crate::upstream::cursor::proto::ExecRequest;

pub fn render(exec: &ExecRequest) -> RenderedToolCall {
    generic_openai::render(exec)
}
