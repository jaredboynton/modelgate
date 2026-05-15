//! Unit tests for the Cursor client-profile detector.
//!
//! Spec source: `.omx/research/cursor-phase0/client-profile-design-v3-deltas.md`
//! (CRIT-2 pseudocode lines 60-126) and `.omx/research/cursor-phase0/
//! client-headers-inventory.md`. The detector module at
//! `src/upstream/cursor/client_profile.rs` is implemented by Lane 2 in
//! parallel. Tests are written against the API spec; they will compile and
//! pass once Lane 2 lands the module.
//!
//! Trust-gated and override env tests acquire `ENV_LOCK` to serialize env
//! mutation across the suite. The `EnvGuard` RAII helpers restore the prior
//! env state on drop so tests do not leak across boundaries.

use std::ffi::OsString;
use std::sync::Mutex;

use axum::http::header::USER_AGENT;
use axum::http::{HeaderMap, HeaderValue};

use unified_model_proxy_v2::upstream::cursor::client_profile::*;

// Single global mutex guarding all env mutations in this file. Acquired by
// every env-touching test so concurrent threads cannot stomp each other.
static ENV_LOCK: Mutex<()> = Mutex::new(());

struct EnvGuard {
    key: &'static str,
    previous: Option<OsString>,
}

impl EnvGuard {
    fn set(key: &'static str, value: &str) -> Self {
        let previous = std::env::var_os(key);
        std::env::set_var(key, value);
        Self { key, previous }
    }

    fn unset(key: &'static str) -> Self {
        let previous = std::env::var_os(key);
        std::env::remove_var(key);
        Self { key, previous }
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        match &self.previous {
            Some(value) => std::env::set_var(self.key, value),
            None => std::env::remove_var(self.key),
        }
    }
}

fn headers_with(pairs: &[(&'static str, &str)]) -> HeaderMap {
    let mut headers = HeaderMap::new();
    for (name, value) in pairs {
        headers.insert(*name, HeaderValue::try_from(*value).expect("valid header"));
    }
    headers
}

// ----------------------------------------------------------------------------
// parse_truthy_env
// ----------------------------------------------------------------------------

#[test]
fn parse_truthy_env_accepts_one() {
    assert!(parse_truthy_env(Some("1")));
}

#[test]
fn parse_truthy_env_accepts_lower_true() {
    assert!(parse_truthy_env(Some("true")));
}

#[test]
fn parse_truthy_env_accepts_mixed_case_true() {
    assert!(parse_truthy_env(Some("True")));
}

#[test]
fn parse_truthy_env_accepts_upper_true() {
    assert!(parse_truthy_env(Some("TRUE")));
}

#[test]
fn parse_truthy_env_accepts_lower_yes() {
    assert!(parse_truthy_env(Some("yes")));
}

#[test]
fn parse_truthy_env_accepts_upper_yes() {
    assert!(parse_truthy_env(Some("YES")));
}

#[test]
fn parse_truthy_env_accepts_lower_on() {
    assert!(parse_truthy_env(Some("on")));
}

#[test]
fn parse_truthy_env_accepts_upper_on() {
    assert!(parse_truthy_env(Some("ON")));
}

#[test]
fn parse_truthy_env_trims_whitespace() {
    assert!(parse_truthy_env(Some(" 1 ")));
}

#[test]
fn parse_truthy_env_rejects_zero() {
    assert!(!parse_truthy_env(Some("0")));
}

#[test]
fn parse_truthy_env_rejects_false() {
    assert!(!parse_truthy_env(Some("false")));
}

#[test]
fn parse_truthy_env_rejects_no() {
    assert!(!parse_truthy_env(Some("no")));
}

#[test]
fn parse_truthy_env_rejects_off() {
    assert!(!parse_truthy_env(Some("off")));
}

#[test]
fn parse_truthy_env_rejects_empty_string() {
    assert!(!parse_truthy_env(Some("")));
}

#[test]
fn parse_truthy_env_rejects_whitespace_only() {
    assert!(!parse_truthy_env(Some(" ")));
}

#[test]
fn parse_truthy_env_rejects_arbitrary_string() {
    assert!(!parse_truthy_env(Some("foo")));
}

#[test]
fn parse_truthy_env_rejects_trueish() {
    assert!(!parse_truthy_env(Some("trueish")));
}

#[test]
fn parse_truthy_env_rejects_none() {
    assert!(!parse_truthy_env(None));
}

// ----------------------------------------------------------------------------
// parse_profile_token
// ----------------------------------------------------------------------------

#[test]
fn parse_profile_token_codex_short_alias() {
    assert!(matches!(
        parse_profile_token("codex"),
        Some(ClientProfile::CodexCli)
    ));
}

#[test]
fn parse_profile_token_codex_snake() {
    assert!(matches!(
        parse_profile_token("codex_cli"),
        Some(ClientProfile::CodexCli)
    ));
}

#[test]
fn parse_profile_token_codex_kebab() {
    assert!(matches!(
        parse_profile_token("codex-cli"),
        Some(ClientProfile::CodexCli)
    ));
}

#[test]
fn parse_profile_token_codex_mixed_case() {
    assert!(matches!(
        parse_profile_token("Codex"),
        Some(ClientProfile::CodexCli)
    ));
}

#[test]
fn parse_profile_token_codex_upper_case() {
    assert!(matches!(
        parse_profile_token("CODEX_CLI"),
        Some(ClientProfile::CodexCli)
    ));
}

#[test]
fn parse_profile_token_claude_code_snake() {
    assert!(matches!(
        parse_profile_token("claude_code"),
        Some(ClientProfile::ClaudeCode)
    ));
}

#[test]
fn parse_profile_token_claude_code_kebab() {
    assert!(matches!(
        parse_profile_token("claude-code"),
        Some(ClientProfile::ClaudeCode)
    ));
}

#[test]
fn parse_profile_token_claude_code_mixed_case() {
    assert!(matches!(
        parse_profile_token("Claude_Code"),
        Some(ClientProfile::ClaudeCode)
    ));
}

#[test]
fn parse_profile_token_droid_lower() {
    assert!(matches!(
        parse_profile_token("droid"),
        Some(ClientProfile::Droid)
    ));
}

#[test]
fn parse_profile_token_droid_mixed_case() {
    assert!(matches!(
        parse_profile_token("Droid"),
        Some(ClientProfile::Droid)
    ));
}

#[test]
fn parse_profile_token_generic_openai_snake() {
    assert!(matches!(
        parse_profile_token("generic_openai"),
        Some(ClientProfile::GenericOpenAi)
    ));
}

#[test]
fn parse_profile_token_generic_openai_kebab() {
    assert!(matches!(
        parse_profile_token("generic-openai"),
        Some(ClientProfile::GenericOpenAi)
    ));
}

#[test]
fn parse_profile_token_generic_anthropic_snake() {
    assert!(matches!(
        parse_profile_token("generic_anthropic"),
        Some(ClientProfile::GenericAnthropic)
    ));
}

#[test]
fn parse_profile_token_generic_anthropic_kebab() {
    assert!(matches!(
        parse_profile_token("generic-anthropic"),
        Some(ClientProfile::GenericAnthropic)
    ));
}

#[test]
fn parse_profile_token_rejects_empty() {
    assert!(parse_profile_token("").is_none());
}

#[test]
fn parse_profile_token_rejects_whitespace_only() {
    assert!(parse_profile_token("   ").is_none());
}

#[test]
fn parse_profile_token_rejects_codex_v3() {
    assert!(parse_profile_token("codex_v3").is_none());
}

#[test]
fn parse_profile_token_rejects_unknown() {
    assert!(parse_profile_token("unknown").is_none());
}

#[test]
fn parse_profile_token_rejects_bare_openai() {
    // Bare "openai" without the generic_ prefix is not accepted.
    assert!(parse_profile_token("openai").is_none());
}

#[test]
fn parse_profile_token_trims_whitespace() {
    assert!(matches!(
        parse_profile_token(" droid "),
        Some(ClientProfile::Droid)
    ));
}

// ----------------------------------------------------------------------------
// is_first_party_codex_originator
// ----------------------------------------------------------------------------

#[test]
fn codex_originator_accepts_cli_rs() {
    assert!(is_first_party_codex_originator("codex_cli_rs"));
}

#[test]
fn codex_originator_accepts_codex_tui() {
    assert!(is_first_party_codex_originator("codex-tui"));
}

#[test]
fn codex_originator_accepts_codex_vscode() {
    assert!(is_first_party_codex_originator("codex_vscode"));
}

#[test]
fn codex_originator_accepts_codex_atlas() {
    assert!(is_first_party_codex_originator("codex_atlas"));
}

#[test]
fn codex_originator_accepts_codex_chatgpt_desktop() {
    assert!(is_first_party_codex_originator("codex_chatgpt_desktop"));
}

#[test]
fn codex_originator_accepts_capital_codex_space_only() {
    // Exact prefix `Codex ` with a trailing space.
    assert!(is_first_party_codex_originator("Codex "));
}

#[test]
fn codex_originator_accepts_capital_codex_with_suffix() {
    assert!(is_first_party_codex_originator("Codex Foo"));
}

#[test]
fn codex_originator_accepts_capital_codex_versioned() {
    assert!(is_first_party_codex_originator("Codex Bar v2.1"));
}

#[test]
fn codex_originator_rejects_lowercase_codex_space() {
    // Case-sensitive: `codex ` (lowercase) is not accepted.
    assert!(!is_first_party_codex_originator("codex "));
}

#[test]
fn codex_originator_rejects_uppercase_codex_space() {
    // Case-sensitive: `CODEX ` is not accepted either.
    assert!(!is_first_party_codex_originator("CODEX "));
}

#[test]
fn codex_originator_rejects_versioned_cli_rs() {
    // First-party token list is exact match; UA-style versioned tokens are rejected.
    assert!(!is_first_party_codex_originator("codex_cli_rs/2.1.0"));
}

#[test]
fn codex_originator_rejects_unknown_codex_variant() {
    assert!(!is_first_party_codex_originator("codex_unknown"));
}

#[test]
fn codex_originator_rejects_empty() {
    assert!(!is_first_party_codex_originator(""));
}

#[test]
fn codex_originator_rejects_trailing_space_on_exact_token() {
    // `codex_cli_rs ` with trailing space is not the exact first-party token,
    // and does not have the `Codex ` capital prefix.
    assert!(!is_first_party_codex_originator("codex_cli_rs "));
}

// ----------------------------------------------------------------------------
// is_claude_code_x_app
// ----------------------------------------------------------------------------

#[test]
fn claude_code_x_app_accepts_cli() {
    assert!(is_claude_code_x_app("cli"));
}

#[test]
fn claude_code_x_app_accepts_cli_bg() {
    assert!(is_claude_code_x_app("cli-bg"));
}

#[test]
fn claude_code_x_app_trims_cli() {
    assert!(is_claude_code_x_app(" cli "));
}

#[test]
fn claude_code_x_app_trims_cli_bg() {
    assert!(is_claude_code_x_app("  cli-bg "));
}

#[test]
fn claude_code_x_app_rejects_cli_headless() {
    // Only exact `cli` / `cli-bg` after trim; future modes are not implicitly
    // accepted via prefix matching.
    assert!(!is_claude_code_x_app("cli-headless"));
}

#[test]
fn claude_code_x_app_rejects_cli_foreground() {
    assert!(!is_claude_code_x_app("cli-foreground"));
}

#[test]
fn claude_code_x_app_rejects_claude_code_string() {
    assert!(!is_claude_code_x_app("claude-code"));
}

#[test]
fn claude_code_x_app_rejects_sandbox() {
    assert!(!is_claude_code_x_app("sandbox"));
}

#[test]
fn claude_code_x_app_rejects_empty() {
    assert!(!is_claude_code_x_app(""));
}

#[test]
fn claude_code_x_app_rejects_uppercase_cli() {
    // Value comparison is case-sensitive; CLI != cli.
    assert!(!is_claude_code_x_app("CLI"));
}

#[test]
fn claude_code_x_app_rejects_uppercase_cli_bg() {
    assert!(!is_claude_code_x_app("CLI-bg"));
}

// ----------------------------------------------------------------------------
// detect_client_profile — UA fallback and trust-off behavior (env-isolated)
// ----------------------------------------------------------------------------

#[test]
fn detect_empty_headers_falls_back_to_generic_openai() {
    let _guard = ENV_LOCK.lock().unwrap();
    let _trust = EnvGuard::unset(ENV_TRUST_HEADERS);
    let _override = EnvGuard::unset(ENV_OVERRIDE);

    let headers = HeaderMap::new();
    let detection = detect_client_profile(&headers);
    assert!(matches!(detection.profile, ClientProfile::GenericOpenAi));
    assert!(matches!(detection.signal, ProfileSignal::None));
}

#[test]
fn detect_originator_ignored_when_trust_off() {
    let _guard = ENV_LOCK.lock().unwrap();
    let _trust = EnvGuard::unset(ENV_TRUST_HEADERS);
    let _override = EnvGuard::unset(ENV_OVERRIDE);

    let headers = headers_with(&[(HEADER_ORIGINATOR, "codex_cli_rs")]);
    let detection = detect_client_profile(&headers);
    // Trust off: originator is ignored. UA is empty so detection falls
    // through to GenericOpenAi/None.
    assert!(matches!(detection.profile, ClientProfile::GenericOpenAi));
    assert!(matches!(detection.signal, ProfileSignal::None));
}

#[test]
fn detect_factory_cli_user_agent_resolves_droid() {
    let _guard = ENV_LOCK.lock().unwrap();
    let _trust = EnvGuard::unset(ENV_TRUST_HEADERS);
    let _override = EnvGuard::unset(ENV_OVERRIDE);

    let headers = headers_with(&[(USER_AGENT.as_str(), "factory-cli/1.0.0")]);
    let detection = detect_client_profile(&headers);
    assert!(matches!(detection.profile, ClientProfile::Droid));
    assert!(matches!(detection.signal, ProfileSignal::UaFactory));
}

#[test]
fn detect_codex_user_agent_resolves_codex_cli() {
    let _guard = ENV_LOCK.lock().unwrap();
    let _trust = EnvGuard::unset(ENV_TRUST_HEADERS);
    let _override = EnvGuard::unset(ENV_OVERRIDE);

    let headers = headers_with(&[(
        USER_AGENT.as_str(),
        "codex_cli_rs/2.1.0 (Mac OS 15.5.0; arm64) iTerm.app",
    )]);
    let detection = detect_client_profile(&headers);
    assert!(matches!(detection.profile, ClientProfile::CodexCli));
    assert!(matches!(detection.signal, ProfileSignal::UaCodex));
}

#[test]
fn detect_claude_cli_user_agent_resolves_claude_code() {
    let _guard = ENV_LOCK.lock().unwrap();
    let _trust = EnvGuard::unset(ENV_TRUST_HEADERS);
    let _override = EnvGuard::unset(ENV_OVERRIDE);

    let headers = headers_with(&[(USER_AGENT.as_str(), "claude-cli/1.0.0")]);
    let detection = detect_client_profile(&headers);
    assert!(matches!(detection.profile, ClientProfile::ClaudeCode));
    assert!(matches!(detection.signal, ProfileSignal::UaClaude));
}

#[test]
fn detect_claude_cli_native_user_agent_resolves_claude_code() {
    let _guard = ENV_LOCK.lock().unwrap();
    let _trust = EnvGuard::unset(ENV_TRUST_HEADERS);
    let _override = EnvGuard::unset(ENV_OVERRIDE);

    let headers = headers_with(&[(USER_AGENT.as_str(), "claude-cli-native-arm64/2.1.142")]);
    let detection = detect_client_profile(&headers);
    assert!(matches!(detection.profile, ClientProfile::ClaudeCode));
    assert!(matches!(detection.signal, ProfileSignal::UaClaudeNative));
}

#[test]
fn detect_anthropic_python_sdk_user_agent_resolves_generic_anthropic() {
    let _guard = ENV_LOCK.lock().unwrap();
    let _trust = EnvGuard::unset(ENV_TRUST_HEADERS);
    let _override = EnvGuard::unset(ENV_OVERRIDE);

    let headers = headers_with(&[(USER_AGENT.as_str(), "anthropic-sdk-python/0.31.0")]);
    let detection = detect_client_profile(&headers);
    assert!(matches!(detection.profile, ClientProfile::GenericAnthropic));
    assert!(matches!(detection.signal, ProfileSignal::UaAnthropicSdk));
}

#[test]
fn detect_anthropic_typescript_sdk_user_agent_resolves_generic_anthropic() {
    let _guard = ENV_LOCK.lock().unwrap();
    let _trust = EnvGuard::unset(ENV_TRUST_HEADERS);
    let _override = EnvGuard::unset(ENV_OVERRIDE);

    let headers = headers_with(&[(USER_AGENT.as_str(), "anthropic-sdk-typescript/0.21.0")]);
    let detection = detect_client_profile(&headers);
    assert!(matches!(detection.profile, ClientProfile::GenericAnthropic));
    assert!(matches!(detection.signal, ProfileSignal::UaAnthropicSdk));
}

#[test]
fn detect_openai_v1_user_agent_resolves_generic_openai() {
    let _guard = ENV_LOCK.lock().unwrap();
    let _trust = EnvGuard::unset(ENV_TRUST_HEADERS);
    let _override = EnvGuard::unset(ENV_OVERRIDE);

    let headers = headers_with(&[(USER_AGENT.as_str(), "OpenAI/v1")]);
    let detection = detect_client_profile(&headers);
    assert!(matches!(detection.profile, ClientProfile::GenericOpenAi));
    assert!(matches!(detection.signal, ProfileSignal::UaOpenAiSdk));
}

#[test]
fn detect_openai_python_user_agent_resolves_generic_openai() {
    let _guard = ENV_LOCK.lock().unwrap();
    let _trust = EnvGuard::unset(ENV_TRUST_HEADERS);
    let _override = EnvGuard::unset(ENV_OVERRIDE);

    let headers = headers_with(&[(USER_AGENT.as_str(), "openai-python/1.30.0")]);
    let detection = detect_client_profile(&headers);
    assert!(matches!(detection.profile, ClientProfile::GenericOpenAi));
    assert!(matches!(detection.signal, ProfileSignal::UaOpenAiSdk));
}

#[test]
fn detect_curl_user_agent_falls_through_to_generic_openai() {
    let _guard = ENV_LOCK.lock().unwrap();
    let _trust = EnvGuard::unset(ENV_TRUST_HEADERS);
    let _override = EnvGuard::unset(ENV_OVERRIDE);

    let headers = headers_with(&[(USER_AGENT.as_str(), "curl/7.64.1")]);
    let detection = detect_client_profile(&headers);
    assert!(matches!(detection.profile, ClientProfile::GenericOpenAi));
    assert!(matches!(detection.signal, ProfileSignal::None));
}

// ----------------------------------------------------------------------------
// detect_client_profile — trust-gated branches
// ----------------------------------------------------------------------------

#[test]
fn detect_trusted_originator_resolves_codex_cli() {
    let _guard = ENV_LOCK.lock().unwrap();
    let _trust = EnvGuard::set(ENV_TRUST_HEADERS, "1");
    let _override = EnvGuard::unset(ENV_OVERRIDE);

    let headers = headers_with(&[(HEADER_ORIGINATOR, "codex_cli_rs")]);
    let detection = detect_client_profile(&headers);
    assert!(matches!(detection.profile, ClientProfile::CodexCli));
    assert!(matches!(detection.signal, ProfileSignal::HeaderOriginator));
}

#[test]
fn detect_trusted_x_app_cli_resolves_claude_code() {
    let _guard = ENV_LOCK.lock().unwrap();
    let _trust = EnvGuard::set(ENV_TRUST_HEADERS, "1");
    let _override = EnvGuard::unset(ENV_OVERRIDE);

    let headers = headers_with(&[(HEADER_X_APP, "cli")]);
    let detection = detect_client_profile(&headers);
    assert!(matches!(detection.profile, ClientProfile::ClaudeCode));
    assert!(matches!(detection.signal, ProfileSignal::HeaderXApp));
}

#[test]
fn detect_trusted_x_app_cli_bg_resolves_claude_code() {
    let _guard = ENV_LOCK.lock().unwrap();
    let _trust = EnvGuard::set(ENV_TRUST_HEADERS, "1");
    let _override = EnvGuard::unset(ENV_OVERRIDE);

    let headers = headers_with(&[(HEADER_X_APP, "cli-bg")]);
    let detection = detect_client_profile(&headers);
    assert!(matches!(detection.profile, ClientProfile::ClaudeCode));
    assert!(matches!(detection.signal, ProfileSignal::HeaderXApp));
}

#[test]
fn detect_trusted_x_app_cli_headless_falls_through() {
    let _guard = ENV_LOCK.lock().unwrap();
    let _trust = EnvGuard::set(ENV_TRUST_HEADERS, "1");
    let _override = EnvGuard::unset(ENV_OVERRIDE);

    // Trust is on, but the predicate rejects `cli-headless`. UA falls
    // through; with no UA, the residual default applies.
    let headers = headers_with(&[(HEADER_X_APP, "cli-headless")]);
    let detection = detect_client_profile(&headers);
    assert!(matches!(detection.profile, ClientProfile::GenericOpenAi));
    assert!(matches!(detection.signal, ProfileSignal::None));
}

#[test]
fn detect_trusted_override_header_droid() {
    let _guard = ENV_LOCK.lock().unwrap();
    let _trust = EnvGuard::set(ENV_TRUST_HEADERS, "1");
    let _override = EnvGuard::unset(ENV_OVERRIDE);

    let headers = headers_with(&[(HEADER_OVERRIDE, "droid")]);
    let detection = detect_client_profile(&headers);
    assert!(matches!(detection.profile, ClientProfile::Droid));
    assert!(matches!(detection.signal, ProfileSignal::OverrideHeader));
}

#[test]
fn detect_override_header_ignored_when_trust_off() {
    let _guard = ENV_LOCK.lock().unwrap();
    let _trust = EnvGuard::set(ENV_TRUST_HEADERS, "0");
    let _override = EnvGuard::unset(ENV_OVERRIDE);

    let headers = headers_with(&[(HEADER_OVERRIDE, "codex")]);
    let detection = detect_client_profile(&headers);
    // Trust off: override header is ignored. No UA, so residual default.
    assert!(matches!(detection.profile, ClientProfile::GenericOpenAi));
    assert!(matches!(detection.signal, ProfileSignal::None));
}

#[test]
fn detect_env_override_wins_regardless_of_trust() {
    let _guard = ENV_LOCK.lock().unwrap();
    // Env override is process-global; trust gate does not apply.
    let _trust = EnvGuard::unset(ENV_TRUST_HEADERS);
    let _override = EnvGuard::set(ENV_OVERRIDE, "claude_code");

    let headers = HeaderMap::new();
    let detection = detect_client_profile(&headers);
    assert!(matches!(detection.profile, ClientProfile::ClaudeCode));
    assert!(matches!(detection.signal, ProfileSignal::OverrideEnv));
}

#[test]
fn detect_invalid_env_override_falls_through_safely() {
    let _guard = ENV_LOCK.lock().unwrap();
    let _trust = EnvGuard::unset(ENV_TRUST_HEADERS);
    let _override = EnvGuard::set(ENV_OVERRIDE, "invalid_value");

    // Invalid override: must not crash; falls through to detection. With
    // empty headers and trust off, residual default is GenericOpenAi/None.
    let headers = HeaderMap::new();
    let detection = detect_client_profile(&headers);
    assert!(matches!(detection.profile, ClientProfile::GenericOpenAi));
    assert!(matches!(detection.signal, ProfileSignal::None));
}
