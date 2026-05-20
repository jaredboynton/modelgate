//! Cursor proxy client-profile detection.
//!
//! Detects connecting client families so Cursor execs render into the right
//! public tool envelopes.

use axum::http::{header::USER_AGENT, HeaderMap};

/// One of five Cursor proxy client profiles.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ClientProfile {
    CodexCli,
    ClaudeCode,
    Droid,
    GenericAnthropic,
    GenericOpenAi,
    Devin,
}

impl From<ClientProfile> for crate::cursor_agent::CursorClientProfile {
    fn from(value: ClientProfile) -> Self {
        match value {
            ClientProfile::CodexCli => Self::CodexCli,
            ClientProfile::ClaudeCode => Self::ClaudeCode,
            ClientProfile::Droid => Self::Droid,
            ClientProfile::GenericAnthropic => Self::GenericAnthropic,
            ClientProfile::GenericOpenAi => Self::GenericOpenAi,
            ClientProfile::Devin => Self::Devin,
        }
    }
}

impl From<crate::cursor_agent::CursorClientProfile> for ClientProfile {
    fn from(value: crate::cursor_agent::CursorClientProfile) -> Self {
        match value {
            crate::cursor_agent::CursorClientProfile::CodexCli => Self::CodexCli,
            crate::cursor_agent::CursorClientProfile::ClaudeCode => Self::ClaudeCode,
            crate::cursor_agent::CursorClientProfile::Droid => Self::Droid,
            crate::cursor_agent::CursorClientProfile::GenericAnthropic => Self::GenericAnthropic,
            crate::cursor_agent::CursorClientProfile::GenericOpenAi => Self::GenericOpenAi,
            crate::cursor_agent::CursorClientProfile::Devin => Self::Devin,
        }
    }
}

impl ClientProfile {
    pub fn as_str(self) -> &'static str {
        match self {
            ClientProfile::CodexCli => "codex_cli",
            ClientProfile::ClaudeCode => "claude_code",
            ClientProfile::Droid => "droid",
            ClientProfile::GenericAnthropic => "generic_anthropic",
            ClientProfile::GenericOpenAi => "generic_openai",
            ClientProfile::Devin => "devin",
        }
    }
}

/// What signal classified this request.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ProfileSignal {
    OverrideEnv,
    OverrideHeader,
    HeaderOriginator,
    HeaderXApp,
    UaCodex,
    UaClaude,
    UaClaudeNative,
    UaFactory,
    UaAnthropicSdk,
    UaOpenAiSdk,
    UaDevin,
    None,
}

impl ProfileSignal {
    pub fn as_str(self) -> &'static str {
        match self {
            ProfileSignal::OverrideEnv => "override_env",
            ProfileSignal::OverrideHeader => "override_header",
            ProfileSignal::HeaderOriginator => "header_originator",
            ProfileSignal::HeaderXApp => "header_x_app",
            ProfileSignal::UaCodex => "ua_codex",
            ProfileSignal::UaClaude => "ua_claude",
            ProfileSignal::UaClaudeNative => "ua_claude_native",
            ProfileSignal::UaFactory => "ua_factory",
            ProfileSignal::UaAnthropicSdk => "ua_anthropic_sdk",
            ProfileSignal::UaOpenAiSdk => "ua_openai_sdk",
            ProfileSignal::UaDevin => "ua_devin",
            ProfileSignal::None => "none",
        }
    }
}

/// Result of [`detect_client_profile`]: which profile and which signal selected it.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct ProfileDetection {
    pub profile: ClientProfile,
    pub signal: ProfileSignal,
}

const FIRST_PARTY_CODEX_ORIGINATORS: &[&str] = &[
    "codex_cli_rs",
    "codex-tui",
    "codex_vscode",
    "codex_atlas",
    "codex_chatgpt_desktop",
];

/// Process-global env var that pins a client profile and bypasses detection.
pub const ENV_OVERRIDE: &str = "UMP_CURSOR_CLIENT_PROFILE_OVERRIDE";
/// Process-global env var that gates trust of request-supplied client headers.
pub const ENV_TRUST_HEADERS: &str = "UMP_CURSOR_TRUST_CLIENT_HEADERS";
/// Per-request override header (gated by `ENV_TRUST_HEADERS`).
pub const HEADER_OVERRIDE: &str = "x-ump-cursor-client-profile";
/// Codex first-party identity header.
pub const HEADER_ORIGINATOR: &str = "originator";
/// Claude Code identity header.
pub const HEADER_X_APP: &str = "x-app";

pub fn is_first_party_codex_originator(value: &str) -> bool {
    FIRST_PARTY_CODEX_ORIGINATORS.contains(&value) || value.starts_with("Codex ")
}

pub fn is_claude_code_x_app(value: &str) -> bool {
    matches!(value.trim(), "cli" | "cli-bg")
}

/// Parses a truthy env value. Accepts `1`, `true`, `yes`, `on`
/// (case-insensitive after trim). Anything else, including missing, is `false`.
pub fn parse_truthy_env(raw: Option<&str>) -> bool {
    matches!(
        raw.map(|v| v.trim()),
        Some(v)
            if v == "1"
                || v.eq_ignore_ascii_case("true")
                || v.eq_ignore_ascii_case("yes")
                || v.eq_ignore_ascii_case("on")
    )
}

pub fn parse_profile_token(raw: &str) -> Option<ClientProfile> {
    let normalized = raw.trim().to_ascii_lowercase();
    match normalized.as_str() {
        "codex" | "codex_cli" | "codex-cli" => Some(ClientProfile::CodexCli),
        "claude_code" | "claude-code" => Some(ClientProfile::ClaudeCode),
        "droid" => Some(ClientProfile::Droid),
        "generic_openai" | "generic-openai" => Some(ClientProfile::GenericOpenAi),
        "generic_anthropic" | "generic-anthropic" => Some(ClientProfile::GenericAnthropic),
        "devin" | "devin_cli" | "devin-cli" => Some(ClientProfile::Devin),
        _ => None,
    }
}

fn header_value<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    headers.get(name).and_then(|v| v.to_str().ok())
}

pub fn detect_client_profile(headers: &HeaderMap) -> ProfileDetection {
    let env_override = std::env::var(ENV_OVERRIDE).ok();
    if let Some(raw) = env_override.as_deref() {
        match parse_profile_token(raw) {
            Some(profile) => {
                return ProfileDetection {
                    profile,
                    signal: ProfileSignal::OverrideEnv,
                };
            }
            None if !raw.trim().is_empty() => {
                tracing::warn!(
                    target: "cursor.client_profile",
                    source = "env",
                    raw_prefix = %raw.chars().take(32).collect::<String>(),
                    normalized = %raw.trim().to_ascii_lowercase(),
                    action = "ignored_fallthrough_detection",
                    "invalid env override profile token",
                );
            }
            None => {}
        }
    }

    let trust_headers = parse_truthy_env(std::env::var(ENV_TRUST_HEADERS).ok().as_deref());

    if trust_headers {
        if let Some(raw) = header_value(headers, HEADER_OVERRIDE) {
            match parse_profile_token(raw) {
                Some(profile) => {
                    return ProfileDetection {
                        profile,
                        signal: ProfileSignal::OverrideHeader,
                    };
                }
                None if !raw.trim().is_empty() => {
                    tracing::warn!(
                        target: "cursor.client_profile",
                        source = "header",
                        raw_prefix = %raw.chars().take(32).collect::<String>(),
                        normalized = %raw.trim().to_ascii_lowercase(),
                        action = "ignored_fallthrough_detection",
                        "invalid header override profile token",
                    );
                }
                None => {}
            }
        }

        if let Some(raw) = header_value(headers, HEADER_ORIGINATOR) {
            if is_first_party_codex_originator(raw) {
                return ProfileDetection {
                    profile: ClientProfile::CodexCli,
                    signal: ProfileSignal::HeaderOriginator,
                };
            }
        }

        if let Some(raw) = header_value(headers, HEADER_X_APP) {
            if is_claude_code_x_app(raw) {
                return ProfileDetection {
                    profile: ClientProfile::ClaudeCode,
                    signal: ProfileSignal::HeaderXApp,
                };
            }
        }
    }

    let ua = header_value(headers, USER_AGENT.as_str()).unwrap_or("");
    if ua.starts_with("devin-cli/")
        || ua.starts_with("chisel-agent/")
        || ua.starts_with("chisel-cli/")
        || ua.starts_with("chisel/")
    {
        return ProfileDetection {
            profile: ClientProfile::Devin,
            signal: ProfileSignal::UaDevin,
        };
    }
    if ua.starts_with("factory-cli/") {
        return ProfileDetection {
            profile: ClientProfile::Droid,
            signal: ProfileSignal::UaFactory,
        };
    }
    for token in FIRST_PARTY_CODEX_ORIGINATORS {
        let with_slash = format!("{token}/");
        if ua.starts_with(&with_slash) {
            return ProfileDetection {
                profile: ClientProfile::CodexCli,
                signal: ProfileSignal::UaCodex,
            };
        }
    }
    if ua.starts_with("Codex ") {
        return ProfileDetection {
            profile: ClientProfile::CodexCli,
            signal: ProfileSignal::UaCodex,
        };
    }
    if ua.starts_with("claude-cli/") {
        return ProfileDetection {
            profile: ClientProfile::ClaudeCode,
            signal: ProfileSignal::UaClaude,
        };
    }
    if ua.starts_with("claude-cli-native-") {
        return ProfileDetection {
            profile: ClientProfile::ClaudeCode,
            signal: ProfileSignal::UaClaudeNative,
        };
    }
    if ua.starts_with("anthropic-sdk-python/") || ua.starts_with("anthropic-sdk-typescript/") {
        return ProfileDetection {
            profile: ClientProfile::GenericAnthropic,
            signal: ProfileSignal::UaAnthropicSdk,
        };
    }
    if ua.starts_with("OpenAI/") || ua.starts_with("openai-") {
        return ProfileDetection {
            profile: ClientProfile::GenericOpenAi,
            signal: ProfileSignal::UaOpenAiSdk,
        };
    }

    ProfileDetection {
        profile: ClientProfile::GenericOpenAi,
        signal: ProfileSignal::None,
    }
}
