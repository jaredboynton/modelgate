use serde::{ser::SerializeStruct, Deserialize, Serialize};

use crate::{compaction::RemoteCompactionPolicy, AppError, AppResult};

#[derive(Debug, Clone, Copy, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Provider {
    Bedrock,
    Codex,
    Cursor,
    Google,
    Windsurf,
    Unsupported,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub struct ModelAlias {
    pub provider: Provider,
    pub upstream_model: &'static str,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub struct ResolvedModel {
    pub provider: Provider,
    pub upstream_model: String,
}

#[derive(Debug, Clone, Copy, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceFormat {
    Responses,
    AnthropicMessages,
    ChatCompletions,
    GoogleGenerateContent,
    OpenaiImages,
}

#[derive(Debug, Clone, Copy, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TargetFormat {
    Responses,
    AnthropicMessages,
    GoogleGenerateContent,
    OpenaiImages,
    CursorAgent,
    WindsurfChat,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub struct ResolvedTarget {
    pub provider: Provider,
    pub upstream_model: String,
    pub target_format: TargetFormat,
}

impl From<ModelAlias> for ResolvedModel {
    fn from(alias: ModelAlias) -> Self {
        Self {
            provider: alias.provider,
            upstream_model: alias.upstream_model.to_string(),
        }
    }
}

impl From<ResolvedTarget> for ResolvedModel {
    fn from(target: ResolvedTarget) -> Self {
        Self {
            provider: target.provider,
            upstream_model: target.upstream_model,
        }
    }
}

impl Provider {
    pub fn default_target_format(self) -> Option<TargetFormat> {
        match self {
            Provider::Bedrock => Some(TargetFormat::AnthropicMessages),
            Provider::Codex => Some(TargetFormat::Responses),
            Provider::Cursor => Some(TargetFormat::CursorAgent),
            Provider::Google => Some(TargetFormat::GoogleGenerateContent),
            Provider::Windsurf => Some(TargetFormat::WindsurfChat),
            Provider::Unsupported => None,
        }
    }
}

impl SourceFormat {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Responses => "responses",
            Self::AnthropicMessages => "anthropic_messages",
            Self::ChatCompletions => "chat_completions",
            Self::GoogleGenerateContent => "google_generate_content",
            Self::OpenaiImages => "openai_images",
        }
    }
}

impl TargetFormat {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Responses => "responses",
            Self::AnthropicMessages => "anthropic_messages",
            Self::GoogleGenerateContent => "google_generate_content",
            Self::OpenaiImages => "openai_images",
            Self::CursorAgent => "cursor_agent",
            Self::WindsurfChat => "windsurf_chat",
        }
    }
}

impl ResolvedTarget {
    pub fn from_resolved_model(model: ResolvedModel, requested_model: &str) -> AppResult<Self> {
        let target_format = model
            .provider
            .default_target_format()
            .ok_or_else(|| AppError::ModelNotSupported(requested_model.to_string()))?;
        Ok(Self {
            provider: model.provider,
            upstream_model: model.upstream_model,
            target_format,
        })
    }

    pub fn default_remote_compaction_policy(&self) -> RemoteCompactionPolicy {
        default_remote_compaction_policy(self.provider, self.target_format)
    }
}

pub const fn default_remote_compaction_policy(
    provider: Provider,
    target_format: TargetFormat,
) -> RemoteCompactionPolicy {
    match (provider, target_format) {
        (Provider::Codex, TargetFormat::Responses) => RemoteCompactionPolicy::Native,
        (Provider::Cursor, TargetFormat::CursorAgent) => RemoteCompactionPolicy::Local,
        (Provider::Windsurf, TargetFormat::WindsurfChat) => RemoteCompactionPolicy::Local,
        (Provider::Unsupported, _) => RemoteCompactionPolicy::Off,
        _ => RemoteCompactionPolicy::Local,
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct KnownModel {
    pub id: &'static str,
    pub provider: Provider,
    pub upstream_model: &'static str,
    /// When true, `resolve_model` also accepts inputs of the form `<id>-YYYYMMDD`
    /// (Anthropic-style dated snapshot aliases) and routes them to this row.
    pub accepts_dated_snapshots: bool,
}

impl Serialize for KnownModel {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("KnownModel", 5)?;
        state.serialize_field("id", self.id)?;
        state.serialize_field("provider", &self.provider)?;
        state.serialize_field("upstream_model", self.upstream_model)?;
        state.serialize_field("accepts_dated_snapshots", &self.accepts_dated_snapshots)?;
        state.serialize_field(
            "remote_compaction_policy",
            &remote_compaction_policy_name(self.default_remote_compaction_policy()),
        )?;
        state.end()
    }
}

fn remote_compaction_policy_name(policy: RemoteCompactionPolicy) -> &'static str {
    match policy {
        RemoteCompactionPolicy::Native => "native",
        RemoteCompactionPolicy::ProxyVisibleSummary => "proxy_visible_summary",
        RemoteCompactionPolicy::Local => "local",
        RemoteCompactionPolicy::Off => "off",
    }
}

impl KnownModel {
    pub fn default_remote_compaction_policy(&self) -> RemoteCompactionPolicy {
        self.provider
            .default_target_format()
            .map(|target_format| default_remote_compaction_policy(self.provider, target_format))
            .unwrap_or(RemoteCompactionPolicy::Off)
    }
}

pub const KNOWN_MODELS: &[KnownModel] = &[
    KnownModel {
        id: "anthropic/claude-sonnet-4-6",
        provider: Provider::Bedrock,
        upstream_model: "us.anthropic.claude-sonnet-4-6",
        accepts_dated_snapshots: false,
    },
    KnownModel {
        id: "claude-sonnet-4-6",
        provider: Provider::Bedrock,
        upstream_model: "us.anthropic.claude-sonnet-4-6",
        accepts_dated_snapshots: false,
    },
    KnownModel {
        id: "anthropic/claude-sonnet-4-6-max",
        provider: Provider::Bedrock,
        upstream_model: "us.anthropic.claude-sonnet-4-6",
        accepts_dated_snapshots: false,
    },
    KnownModel {
        id: "claude-sonnet-4-6-max",
        provider: Provider::Bedrock,
        upstream_model: "us.anthropic.claude-sonnet-4-6",
        accepts_dated_snapshots: false,
    },
    KnownModel {
        id: "claude-haiku-4-5",
        provider: Provider::Bedrock,
        upstream_model: "anthropic.claude-haiku-4-5",
        accepts_dated_snapshots: true,
    },
    KnownModel {
        id: "anthropic/claude-haiku-4-5",
        provider: Provider::Bedrock,
        upstream_model: "anthropic.claude-haiku-4-5",
        accepts_dated_snapshots: true,
    },
    KnownModel {
        id: "anthropic/claude-opus-4-6",
        provider: Provider::Bedrock,
        upstream_model: "us.anthropic.claude-opus-4-6-v1",
        accepts_dated_snapshots: false,
    },
    KnownModel {
        id: "claude-opus-4-6",
        provider: Provider::Bedrock,
        upstream_model: "us.anthropic.claude-opus-4-6-v1",
        accepts_dated_snapshots: false,
    },
    KnownModel {
        id: "anthropic/claude-opus-4-6-max",
        provider: Provider::Bedrock,
        upstream_model: "us.anthropic.claude-opus-4-6-v1",
        accepts_dated_snapshots: false,
    },
    KnownModel {
        id: "claude-opus-4-6-max",
        provider: Provider::Bedrock,
        upstream_model: "us.anthropic.claude-opus-4-6-v1",
        accepts_dated_snapshots: false,
    },
    KnownModel {
        id: "anthropic/claude-opus-4-7",
        provider: Provider::Bedrock,
        upstream_model: "anthropic.claude-opus-4-7",
        accepts_dated_snapshots: false,
    },
    KnownModel {
        id: "claude-opus-4-7",
        provider: Provider::Bedrock,
        upstream_model: "anthropic.claude-opus-4-7",
        accepts_dated_snapshots: false,
    },
    KnownModel {
        id: "anthropic/claude-opus-4-7-max",
        provider: Provider::Bedrock,
        upstream_model: "anthropic.claude-opus-4-7",
        accepts_dated_snapshots: false,
    },
    KnownModel {
        id: "claude-opus-4-7-max",
        provider: Provider::Bedrock,
        upstream_model: "anthropic.claude-opus-4-7",
        accepts_dated_snapshots: false,
    },
    KnownModel {
        id: "openai:gpt-5.5",
        provider: Provider::Codex,
        upstream_model: "gpt-5.5",
        accepts_dated_snapshots: false,
    },
    KnownModel {
        id: "openai/gpt-5.5",
        provider: Provider::Codex,
        upstream_model: "gpt-5.5",
        accepts_dated_snapshots: false,
    },
    KnownModel {
        id: "gpt-5.5",
        provider: Provider::Codex,
        upstream_model: "gpt-5.5",
        accepts_dated_snapshots: false,
    },
    KnownModel {
        id: "openai/gpt-5.4",
        provider: Provider::Codex,
        upstream_model: "gpt-5.4",
        accepts_dated_snapshots: false,
    },
    KnownModel {
        id: "gpt-5.4",
        provider: Provider::Codex,
        upstream_model: "gpt-5.4",
        accepts_dated_snapshots: false,
    },
    KnownModel {
        id: "gpt-5.4-mini",
        provider: Provider::Codex,
        upstream_model: "gpt-5.4-mini",
        accepts_dated_snapshots: false,
    },
    KnownModel {
        id: "gpt-5.3-codex",
        provider: Provider::Codex,
        upstream_model: "gpt-5.3-codex",
        accepts_dated_snapshots: false,
    },
    KnownModel {
        id: "gpt-5.2",
        provider: Provider::Codex,
        upstream_model: "gpt-5.2",
        accepts_dated_snapshots: false,
    },
    KnownModel {
        id: "codex-auto-review",
        provider: Provider::Codex,
        upstream_model: "codex-auto-review",
        accepts_dated_snapshots: false,
    },
    KnownModel {
        id: "vertexai/gemini-3-flash-preview",
        provider: Provider::Google,
        upstream_model: "gemini-3-flash-preview",
        accepts_dated_snapshots: false,
    },
    KnownModel {
        id: "gemini-3-flash-preview",
        provider: Provider::Google,
        upstream_model: "gemini-3-flash-preview",
        accepts_dated_snapshots: false,
    },
    KnownModel {
        id: "vertexai/gemini-3.1-pro-preview",
        provider: Provider::Google,
        upstream_model: "gemini-3.1-pro-preview",
        accepts_dated_snapshots: false,
    },
    KnownModel {
        id: "gemini-3.1-pro-preview",
        provider: Provider::Google,
        upstream_model: "gemini-3.1-pro-preview",
        accepts_dated_snapshots: false,
    },
    KnownModel {
        id: "gemini-3.1-flash-lite",
        provider: Provider::Google,
        upstream_model: "gemini-3.1-flash-lite",
        accepts_dated_snapshots: false,
    },
    KnownModel {
        id: "vertexai/gemini-3.1-flash-lite",
        provider: Provider::Google,
        upstream_model: "gemini-3.1-flash-lite",
        accepts_dated_snapshots: false,
    },
    KnownModel {
        id: "vertexai/gemini-3-pro-image",
        provider: Provider::Google,
        upstream_model: "gemini-3-pro-image-preview",
        accepts_dated_snapshots: false,
    },
    KnownModel {
        id: "gemini-3-pro-image",
        provider: Provider::Google,
        upstream_model: "gemini-3-pro-image-preview",
        accepts_dated_snapshots: false,
    },
    KnownModel {
        id: "gemini-3-pro-image-preview",
        provider: Provider::Google,
        upstream_model: "gemini-3-pro-image-preview",
        accepts_dated_snapshots: false,
    },
    KnownModel {
        id: "composer-1.5",
        provider: Provider::Cursor,
        upstream_model: "composer-1.5",
        accepts_dated_snapshots: false,
    },
    KnownModel {
        id: "composer-2",
        provider: Provider::Cursor,
        upstream_model: "composer-2",
        accepts_dated_snapshots: false,
    },
    KnownModel {
        id: "composer-2-fast",
        provider: Provider::Cursor,
        upstream_model: "composer-2-fast",
        accepts_dated_snapshots: false,
    },
    KnownModel {
        id: "swe-1.6-fast",
        provider: Provider::Windsurf,
        upstream_model: "swe-1-6-fast",
        accepts_dated_snapshots: false,
    },
    KnownModel {
        id: "swe-1-6-fast",
        provider: Provider::Windsurf,
        upstream_model: "swe-1-6-fast",
        accepts_dated_snapshots: false,
    },
    KnownModel {
        id: "windsurf/swe-1.6-fast",
        provider: Provider::Windsurf,
        upstream_model: "swe-1-6-fast",
        accepts_dated_snapshots: false,
    },
    KnownModel {
        id: "swe-1.6",
        provider: Provider::Windsurf,
        upstream_model: "swe-1-6",
        accepts_dated_snapshots: false,
    },
    KnownModel {
        id: "swe-1-6",
        provider: Provider::Windsurf,
        upstream_model: "swe-1-6",
        accepts_dated_snapshots: false,
    },
    KnownModel {
        id: "windsurf/swe-1.6",
        provider: Provider::Windsurf,
        upstream_model: "swe-1-6",
        accepts_dated_snapshots: false,
    },
    KnownModel {
        id: "swe-1.5-fast",
        provider: Provider::Windsurf,
        upstream_model: "swe-1-5",
        accepts_dated_snapshots: false,
    },
    KnownModel {
        id: "swe-1-5-fast",
        provider: Provider::Windsurf,
        upstream_model: "swe-1-5",
        accepts_dated_snapshots: false,
    },
    KnownModel {
        id: "windsurf/swe-1.5-fast",
        provider: Provider::Windsurf,
        upstream_model: "swe-1-5",
        accepts_dated_snapshots: false,
    },
    KnownModel {
        id: "adaptive",
        provider: Provider::Windsurf,
        upstream_model: "adaptive",
        accepts_dated_snapshots: false,
    },
    KnownModel {
        id: "windsurf/adaptive",
        provider: Provider::Windsurf,
        upstream_model: "adaptive",
        accepts_dated_snapshots: false,
    },
    KnownModel {
        id: "gpt-image-2",
        provider: Provider::Unsupported,
        upstream_model: "gpt-image-2",
        accepts_dated_snapshots: false,
    },
];

pub fn resolve_model(input: &str) -> Option<ModelAlias> {
    if let Some(hit) = lookup_exact(input) {
        return Some(hit);
    }
    if let Some(stripped) = strip_dated_suffix(input) {
        if let Some(model) = KNOWN_MODELS
            .iter()
            .find(|m| m.id == stripped && m.accepts_dated_snapshots)
        {
            return Some(ModelAlias {
                provider: model.provider,
                upstream_model: model.upstream_model,
            });
        }
    }
    None
}

pub fn resolve_model_required(input: &str) -> AppResult<ResolvedModel> {
    resolve_model(input)
        .map(ResolvedModel::from)
        .ok_or_else(|| AppError::ModelNotSupported(input.into()))
}

pub fn resolve_target_required(input: &str) -> AppResult<ResolvedTarget> {
    ResolvedTarget::from_resolved_model(resolve_model_required(input)?, input)
}

fn lookup_exact(input: &str) -> Option<ModelAlias> {
    KNOWN_MODELS
        .iter()
        .find(|model| model.id == input)
        .map(|model| ModelAlias {
            provider: model.provider,
            upstream_model: model.upstream_model,
        })
}

/// Strip a trailing `-YYYYMMDD` if present. Returns None on no match.
/// `bytes.len() > 9` ensures the stripped prefix is never empty.
fn strip_dated_suffix(input: &str) -> Option<&str> {
    let bytes = input.as_bytes();
    if bytes.len() <= 9 {
        return None;
    }
    let split_at = bytes.len() - 9;
    if bytes[split_at] != b'-' {
        return None;
    }
    if !bytes[split_at + 1..].iter().all(|b| b.is_ascii_digit()) {
        return None;
    }
    Some(&input[..split_at])
}
