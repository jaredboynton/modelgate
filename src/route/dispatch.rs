use serde_json::Value;

use crate::{
    compaction::RemoteCompactionPolicy,
    model_alias::{Provider, ResolvedModel, ResolvedTarget, TargetFormat},
    AppError, AppResult, AppState,
};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum RequestFormat {
    Responses,
    AnthropicMessages,
    ChatCompletions,
}

impl RequestFormat {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Responses => "responses",
            Self::AnthropicMessages => "anthropic_messages",
            Self::ChatCompletions => "chat_completions",
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum DispatchAction {
    CodexResponses,
    BedrockAnthropicMessages,
    GoogleGenerateContent,
    CursorAgent,
    WindsurfChat,
}

impl DispatchAction {
    pub const fn name(self) -> &'static str {
        match self {
            Self::CodexResponses => "responses/codex",
            Self::BedrockAnthropicMessages => "anthropic_messages/bedrock",
            Self::GoogleGenerateContent => "google_generate_content/google",
            Self::CursorAgent => "cursor_agent/cursor",
            Self::WindsurfChat => "windsurf_chat/windsurf",
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum DispatchEdge {
    ResponsesToResponsesCodex,
    ResponsesToAnthropicMessagesBedrock,
    ResponsesToGoogleGenerateContentGoogle,
    AnthropicMessagesToAnthropicMessagesBedrock,
    AnthropicMessagesToResponsesCodex,
    ChatCompletionsToResponsesCodex,
    ChatCompletionsToAnthropicMessagesBedrock,
    ResponsesToCursorAgentCursor,
    ChatCompletionsToCursorAgentCursor,
    AnthropicMessagesToCursorAgentCursor,
    ResponsesToWindsurfChatWindsurf,
    ChatCompletionsToWindsurfChatWindsurf,
}

impl DispatchEdge {
    pub const fn name(self) -> &'static str {
        match self {
            Self::ResponsesToResponsesCodex => "responses->responses/Codex",
            Self::ResponsesToAnthropicMessagesBedrock => "responses->anthropic_messages/Bedrock",
            Self::ResponsesToGoogleGenerateContentGoogle => {
                "responses->google_generate_content/Google"
            }
            Self::AnthropicMessagesToAnthropicMessagesBedrock => {
                "anthropic_messages->anthropic_messages/Bedrock"
            }
            Self::AnthropicMessagesToResponsesCodex => "anthropic_messages->responses/Codex",
            Self::ChatCompletionsToResponsesCodex => "chat_completions->responses/Codex",
            Self::ChatCompletionsToAnthropicMessagesBedrock => {
                "chat_completions->anthropic_messages/Bedrock"
            }
            Self::ResponsesToCursorAgentCursor => "responses->cursor_agent/Cursor",
            Self::ChatCompletionsToCursorAgentCursor => "chat_completions->cursor_agent/Cursor",
            Self::AnthropicMessagesToCursorAgentCursor => "anthropic_messages->cursor_agent/Cursor",
            Self::ResponsesToWindsurfChatWindsurf => "responses->windsurf_chat/Windsurf",
            Self::ChatCompletionsToWindsurfChatWindsurf => {
                "chat_completions->windsurf_chat/Windsurf"
            }
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct DispatchPlan {
    pub source_format: RequestFormat,
    pub requested_model: String,
    pub target: ResolvedTarget,
    pub remote_compaction_policy: RemoteCompactionPolicy,
    pub edge: DispatchEdge,
    pub action: DispatchAction,
}

pub fn plan_with_state(
    state: &AppState,
    source_format: RequestFormat,
    value: &Value,
) -> AppResult<DispatchPlan> {
    let model = required_model(value)?;
    let target = state.resolve_target_for_format(model, source_format.as_str())?;
    let remote_compaction_policy = state.routing_config.remote_compaction_policy_for_format(
        model,
        Some(source_format.as_str()),
        &target,
    )?;
    plan_for_target_with_remote_compaction_policy(
        source_format,
        model,
        target,
        remote_compaction_policy,
    )
}

pub fn plan_with_resolver<F>(
    value: &Value,
    source_format: RequestFormat,
    mut resolve: F,
) -> AppResult<DispatchPlan>
where
    F: FnMut(&str) -> AppResult<ResolvedModel>,
{
    let model = required_model(value)?;
    let target = resolve(model)?;
    let target = ResolvedTarget::from_resolved_model(target, model)?;
    plan_for_target_with_remote_compaction_policy(
        source_format,
        model,
        target.clone(),
        target.default_remote_compaction_policy(),
    )
}

pub fn plan_for_target(
    source_format: RequestFormat,
    requested_model: &str,
    target: ResolvedTarget,
) -> AppResult<DispatchPlan> {
    let remote_compaction_policy = target.default_remote_compaction_policy();
    plan_for_target_with_remote_compaction_policy(
        source_format,
        requested_model,
        target,
        remote_compaction_policy,
    )
}

pub fn plan_for_target_with_remote_compaction_policy(
    source_format: RequestFormat,
    requested_model: &str,
    target: ResolvedTarget,
    remote_compaction_policy: RemoteCompactionPolicy,
) -> AppResult<DispatchPlan> {
    let (edge, action) = match (source_format, target.provider, target.target_format) {
        (RequestFormat::Responses, Provider::Codex, TargetFormat::Responses)
            if is_codex_responses_model(&target) =>
        {
            (
                DispatchEdge::ResponsesToResponsesCodex,
                DispatchAction::CodexResponses,
            )
        }
        (RequestFormat::Responses, Provider::Bedrock, TargetFormat::AnthropicMessages)
            if is_anthropic_messages_model(requested_model, &target) =>
        {
            (
                DispatchEdge::ResponsesToAnthropicMessagesBedrock,
                DispatchAction::BedrockAnthropicMessages,
            )
        }
        (RequestFormat::Responses, Provider::Google, TargetFormat::GoogleGenerateContent) => (
            DispatchEdge::ResponsesToGoogleGenerateContentGoogle,
            DispatchAction::GoogleGenerateContent,
        ),
        (RequestFormat::AnthropicMessages, Provider::Bedrock, TargetFormat::AnthropicMessages) => (
            DispatchEdge::AnthropicMessagesToAnthropicMessagesBedrock,
            DispatchAction::BedrockAnthropicMessages,
        ),
        (RequestFormat::AnthropicMessages, Provider::Codex, TargetFormat::Responses)
            if is_codex_responses_model(&target) =>
        {
            (
                DispatchEdge::AnthropicMessagesToResponsesCodex,
                DispatchAction::CodexResponses,
            )
        }
        (RequestFormat::ChatCompletions, Provider::Codex, TargetFormat::Responses)
            if is_codex_responses_model(&target) =>
        {
            (
                DispatchEdge::ChatCompletionsToResponsesCodex,
                DispatchAction::CodexResponses,
            )
        }
        (RequestFormat::ChatCompletions, Provider::Bedrock, TargetFormat::AnthropicMessages)
            if is_anthropic_messages_model(requested_model, &target) =>
        {
            (
                DispatchEdge::ChatCompletionsToAnthropicMessagesBedrock,
                DispatchAction::BedrockAnthropicMessages,
            )
        }
        (RequestFormat::Responses, Provider::Cursor, TargetFormat::CursorAgent) => (
            DispatchEdge::ResponsesToCursorAgentCursor,
            DispatchAction::CursorAgent,
        ),
        (RequestFormat::ChatCompletions, Provider::Cursor, TargetFormat::CursorAgent) => (
            DispatchEdge::ChatCompletionsToCursorAgentCursor,
            DispatchAction::CursorAgent,
        ),
        (RequestFormat::AnthropicMessages, Provider::Cursor, TargetFormat::CursorAgent) => (
            DispatchEdge::AnthropicMessagesToCursorAgentCursor,
            DispatchAction::CursorAgent,
        ),
        (RequestFormat::Responses, Provider::Windsurf, TargetFormat::WindsurfChat) => (
            DispatchEdge::ResponsesToWindsurfChatWindsurf,
            DispatchAction::WindsurfChat,
        ),
        (RequestFormat::ChatCompletions, Provider::Windsurf, TargetFormat::WindsurfChat) => (
            DispatchEdge::ChatCompletionsToWindsurfChatWindsurf,
            DispatchAction::WindsurfChat,
        ),
        _ => return Err(AppError::ModelNotSupported(requested_model.to_string())),
    };

    Ok(DispatchPlan {
        source_format,
        requested_model: requested_model.to_string(),
        target,
        remote_compaction_policy,
        edge,
        action,
    })
}

/*
    The source/target pairs below are the Phase 1 dispatch matrix:
    responses -> responses/Codex
    responses -> anthropic_messages/Bedrock
    responses -> google_generate_content/Google
    anthropic_messages -> anthropic_messages/Bedrock
    anthropic_messages -> responses/Codex
    chat_completions -> responses/Codex
    chat_completions -> anthropic_messages/Bedrock
    responses -> cursor_agent/Cursor
    chat_completions -> cursor_agent/Cursor
    anthropic_messages -> cursor_agent/Cursor
    responses -> windsurf_chat/Windsurf
    chat_completions -> windsurf_chat/Windsurf
*/
#[allow(dead_code)]
fn _phase_one_matrix_reference() {
    let _ = (
        DispatchEdge::ResponsesToResponsesCodex,
        DispatchEdge::ResponsesToAnthropicMessagesBedrock,
        DispatchEdge::ResponsesToGoogleGenerateContentGoogle,
        DispatchEdge::AnthropicMessagesToAnthropicMessagesBedrock,
        DispatchEdge::AnthropicMessagesToResponsesCodex,
        DispatchEdge::ChatCompletionsToResponsesCodex,
        DispatchEdge::ChatCompletionsToAnthropicMessagesBedrock,
        DispatchEdge::ResponsesToCursorAgentCursor,
        DispatchEdge::ChatCompletionsToCursorAgentCursor,
        DispatchEdge::AnthropicMessagesToCursorAgentCursor,
        DispatchEdge::ResponsesToWindsurfChatWindsurf,
        DispatchEdge::ChatCompletionsToWindsurfChatWindsurf,
    );
}

pub fn required_model(value: &Value) -> AppResult<&str> {
    value
        .get("model")
        .and_then(Value::as_str)
        .ok_or_else(|| AppError::BadRequest("missing model".into()))
}

pub fn resolve_planned_model(
    plan: &DispatchPlan,
    model: &str,
    mut fallback: impl FnMut(&str) -> AppResult<ResolvedModel>,
) -> AppResult<ResolvedModel> {
    if model == plan.requested_model {
        return Ok(plan.target.clone().into());
    }
    fallback(model)
}

fn is_codex_responses_model(target: &ResolvedTarget) -> bool {
    target.upstream_model.starts_with("gpt-")
}

fn is_anthropic_messages_model(requested_model: &str, target: &ResolvedTarget) -> bool {
    requested_model.starts_with("anthropic/") || target.upstream_model.contains("claude")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn target(
        provider: Provider,
        upstream_model: &str,
        target_format: TargetFormat,
    ) -> ResolvedTarget {
        ResolvedTarget {
            provider,
            upstream_model: upstream_model.to_string(),
            target_format,
        }
    }

    #[test]
    fn planner_names_phase_one_allowed_edges() {
        let cases = [
            (
                RequestFormat::Responses,
                "openai:gpt-5.5",
                target(Provider::Codex, "gpt-5.5", TargetFormat::Responses),
                "responses->responses/Codex",
                "responses/codex",
            ),
            (
                RequestFormat::Responses,
                "claude-sonnet-4-6",
                target(
                    Provider::Bedrock,
                    "us.anthropic.claude-sonnet-4-6",
                    TargetFormat::AnthropicMessages,
                ),
                "responses->anthropic_messages/Bedrock",
                "anthropic_messages/bedrock",
            ),
            (
                RequestFormat::Responses,
                "gemini-3.1-flash-lite",
                target(
                    Provider::Google,
                    "gemini-3.1-flash-lite",
                    TargetFormat::GoogleGenerateContent,
                ),
                "responses->google_generate_content/Google",
                "google_generate_content/google",
            ),
            (
                RequestFormat::AnthropicMessages,
                "claude-sonnet-4-6",
                target(
                    Provider::Bedrock,
                    "us.anthropic.claude-sonnet-4-6",
                    TargetFormat::AnthropicMessages,
                ),
                "anthropic_messages->anthropic_messages/Bedrock",
                "anthropic_messages/bedrock",
            ),
            (
                RequestFormat::AnthropicMessages,
                "openai:gpt-5.5",
                target(Provider::Codex, "gpt-5.5", TargetFormat::Responses),
                "anthropic_messages->responses/Codex",
                "responses/codex",
            ),
            (
                RequestFormat::ChatCompletions,
                "openai:gpt-5.5",
                target(Provider::Codex, "gpt-5.5", TargetFormat::Responses),
                "chat_completions->responses/Codex",
                "responses/codex",
            ),
            (
                RequestFormat::ChatCompletions,
                "claude-sonnet-4-6",
                target(
                    Provider::Bedrock,
                    "us.anthropic.claude-sonnet-4-6",
                    TargetFormat::AnthropicMessages,
                ),
                "chat_completions->anthropic_messages/Bedrock",
                "anthropic_messages/bedrock",
            ),
            (
                RequestFormat::Responses,
                "composer-2",
                target(Provider::Cursor, "composer-2", TargetFormat::CursorAgent),
                "responses->cursor_agent/Cursor",
                "cursor_agent/cursor",
            ),
            (
                RequestFormat::ChatCompletions,
                "composer-2-fast",
                target(
                    Provider::Cursor,
                    "composer-2-fast",
                    TargetFormat::CursorAgent,
                ),
                "chat_completions->cursor_agent/Cursor",
                "cursor_agent/cursor",
            ),
            (
                RequestFormat::AnthropicMessages,
                "composer-1.5",
                target(Provider::Cursor, "composer-1.5", TargetFormat::CursorAgent),
                "anthropic_messages->cursor_agent/Cursor",
                "cursor_agent/cursor",
            ),
        ];

        for (source_format, model, target, edge, action) in cases {
            let plan = plan_for_target(source_format, model, target).unwrap();
            assert_eq!(plan.edge.name(), edge);
            assert_eq!(plan.action.name(), action);
        }
    }

    #[test]
    fn planner_rejects_edges_outside_phase_one_matrix() {
        let plan = plan_for_target(
            RequestFormat::ChatCompletions,
            "gemini-3.1-flash-lite",
            target(
                Provider::Google,
                "gemini-3.1-flash-lite",
                TargetFormat::GoogleGenerateContent,
            ),
        );

        assert!(
            matches!(plan, Err(AppError::ModelNotSupported(model)) if model == "gemini-3.1-flash-lite")
        );
    }

    #[test]
    fn planner_rejects_provider_target_format_mismatch() {
        let plan = plan_for_target(
            RequestFormat::Responses,
            "facade-google-model",
            target(
                Provider::Google,
                "gemini-3.1-flash-lite",
                TargetFormat::Responses,
            ),
        );

        assert!(
            matches!(plan, Err(AppError::ModelNotSupported(model)) if model == "facade-google-model")
        );
    }
}
