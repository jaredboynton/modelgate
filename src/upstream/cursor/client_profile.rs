//! Cursor proxy client-profile detection.
//!
//! Per the round-5 APPROVED design at
//! `.omx/research/cursor-phase0/client-profile-design-v3-deltas.md`,
//! detect which client (Codex CLI fork family, Claude Code, Droid,
//! GenericAnthropic, GenericOpenAi) is connecting so the per-profile
//! tool renderer can emit the right tool envelopes.
//!
//! Detection precedence:
//! 1. Process-global env override `UMP_CURSOR_CLIENT_PROFILE_OVERRIDE`.
//! 2. Trust gate: if `UMP_CURSOR_TRUST_CLIENT_HEADERS=1`,
//!    consult request override header `x-ump-cursor-client-profile`,
//!    then `originator` (Codex), then `x-app` (Claude Code).
//! 3. UA fallback (always consulted).
//! 4. `GenericOpenAi` default.

use axum::http::{header::USER_AGENT, HeaderMap};

/// One of five Cursor proxy client profiles.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ClientProfile {
    /// Codex CLI fork family (codex_cli_rs, codex-tui, codex_vscode,
    /// codex_atlas, codex_chatgpt_desktop, plus any `Codex `-prefixed value).
    CodexCli,
    /// Anthropic Claude Code (`claude-cli/<ver>` or `claude-cli-native-...`).
    ClaudeCode,
    /// Factory Droid (`factory-cli/<ver>`).
    Droid,
    /// Generic Anthropic SDK consumer with no Claude Code signature.
    GenericAnthropic,
    /// Residual default; preserves the current lowercase OpenAI-canonical
    /// Cursor public-tool-call names.
    GenericOpenAi,
}

impl ClientProfile {
    /// Canonical snake_case telemetry token for this profile.
    pub fn as_str(self) -> &'static str {
        match self {
            ClientProfile::CodexCli => "codex_cli",
            ClientProfile::ClaudeCode => "claude_code",
            ClientProfile::Droid => "droid",
            ClientProfile::GenericAnthropic => "generic_anthropic",
            ClientProfile::GenericOpenAi => "generic_openai",
        }
    }
}

/// What signal classified this request.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ProfileSignal {
    /// Process-global env var `UMP_CURSOR_CLIENT_PROFILE_OVERRIDE`.
    OverrideEnv,
    /// Trusted request header `x-ump-cursor-client-profile`.
    OverrideHeader,
    /// Trusted Codex `originator` request header.
    HeaderOriginator,
    /// Trusted Claude Code `x-app` request header.
    HeaderXApp,
    /// `User-Agent` matched a Codex first-party prefix.
    UaCodex,
    /// `User-Agent` matched `claude-cli/<ver>`.
    UaClaude,
    /// `User-Agent` matched `claude-cli-native-...`.
    UaClaudeNative,
    /// `User-Agent` matched `factory-cli/<ver>`.
    UaFactory,
    /// `User-Agent` matched `anthropic-sdk-python/` or `anthropic-sdk-typescript/`.
    UaAnthropicSdk,
    /// `User-Agent` matched `OpenAI/` or `openai-`.
    UaOpenAiSdk,
    /// No signal matched; profile fell through to `GenericOpenAi`.
    None,
}

impl ProfileSignal {
    /// Canonical telemetry token for this signal.
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

/// First-party Codex originator tokens (exact match) per
/// `codex-rs/login/src/auth/default_client.rs:122-131`. Values starting with
/// the literal `Codex ` prefix (capital C, trailing space) are also accepted.
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

/// Returns true when `value` is a recognized first-party Codex originator
/// (exact match against the canonical list, or the case-sensitive `Codex `
/// prefix).
pub fn is_first_party_codex_originator(value: &str) -> bool {
    FIRST_PARTY_CODEX_ORIGINATORS.contains(&value) || value.starts_with("Codex ")
}

/// Returns true when `value` (after trimming) is exactly `cli` or `cli-bg`.
///
/// Per the v3 CRIT-1 decision, the `x-app` predicate is an exact allowlist:
/// no prefixes, no co-presence checks. Comparison is case-sensitive after trim.
pub fn is_claude_code_x_app(value: &str) -> bool {
    matches!(value.trim(), "cli" | "cli-bg")
}

/// Parses a truthy env value. Accepts `1`, `true`, `yes`, `on`
/// (case-insensitive after trim). Anything else, including missing, is `false`.
pub fn parse_truthy_env(raw: Option<&str>) -> bool {
    match raw.map(|v| v.trim()) {
        Some(v)
            if v == "1"
                || v.eq_ignore_ascii_case("true")
                || v.eq_ignore_ascii_case("yes")
                || v.eq_ignore_ascii_case("on") =>
        {
            true
        }
        _ => false,
    }
}

/// Parses a profile token (env or header value) into a [`ClientProfile`].
///
/// Accepts canonical snake_case tokens, hyphenated aliases, and the `codex`
/// short alias. Trim-tolerant and ASCII-case-insensitive.
pub fn parse_profile_token(raw: &str) -> Option<ClientProfile> {
    let normalized = raw.trim().to_ascii_lowercase();
    match normalized.as_str() {
        "codex" | "codex_cli" | "codex-cli" => Some(ClientProfile::CodexCli),
        "claude_code" | "claude-code" => Some(ClientProfile::ClaudeCode),
        "droid" => Some(ClientProfile::Droid),
        "generic_openai" | "generic-openai" => Some(ClientProfile::GenericOpenAi),
        "generic_anthropic" | "generic-anthropic" => Some(ClientProfile::GenericAnthropic),
        _ => None,
    }
}

fn header_value<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    headers.get(name).and_then(|v| v.to_str().ok())
}

/// Detect the connecting client's profile from request headers and process env.
///
/// See module docs for the precedence rules. Invalid override values fail open:
/// the proxy emits a WARN log and continues detection.
pub fn detect_client_profile(headers: &HeaderMap) -> ProfileDetection {
    // 1. Process-global env override (always wins, never gated).
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
        // 2a. Trusted request override header.
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

        // 2b. Codex originator.
        if let Some(raw) = header_value(headers, HEADER_ORIGINATOR) {
            if is_first_party_codex_originator(raw) {
                return ProfileDetection {
                    profile: ClientProfile::CodexCli,
                    signal: ProfileSignal::HeaderOriginator,
                };
            }
        }

        // 2c. Claude Code x-app.
        if let Some(raw) = header_value(headers, HEADER_X_APP) {
            if is_claude_code_x_app(raw) {
                return ProfileDetection {
                    profile: ClientProfile::ClaudeCode,
                    signal: ProfileSignal::HeaderXApp,
                };
            }
        }
    }

    // 3. UA fallback (always consulted).
    let ua = header_value(headers, USER_AGENT.as_str()).unwrap_or("");
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

    // 4. Residual default.
    ProfileDetection {
        profile: ClientProfile::GenericOpenAi,
        signal: ProfileSignal::None,
    }
}
