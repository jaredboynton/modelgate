use serde::{Deserialize, Serialize};

use crate::{AppError, AppResult};

#[derive(Debug, Clone, Copy, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Provider {
    Bedrock,
    Codex,
    Google,
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

impl From<ModelAlias> for ResolvedModel {
    fn from(alias: ModelAlias) -> Self {
        Self {
            provider: alias.provider,
            upstream_model: alias.upstream_model.to_string(),
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
pub struct KnownModel {
    pub id: &'static str,
    pub provider: Provider,
    pub upstream_model: &'static str,
    /// When true, `resolve_model` also accepts inputs of the form `<id>-YYYYMMDD`
    /// (Anthropic-style dated snapshot aliases) and routes them to this row.
    pub accepts_dated_snapshots: bool,
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
