#![allow(dead_code)]

use std::path::{Path, PathBuf};

use tempfile::TempDir;
use unified_model_proxy_v2::AppState;

pub struct TestHomes {
    _temp: TempDir,
    pub state: AppState,
}

impl TestHomes {
    pub fn new() -> Self {
        let temp = tempfile::tempdir().expect("create temp home");
        let codex_home = temp.path().join("codex");
        let auth_home = temp.path().join("ump");
        let state = temp_state(codex_home, auth_home);
        Self { _temp: temp, state }
    }
}

pub fn temp_state(codex_home: PathBuf, auth_home: PathBuf) -> AppState {
    assert_temp_home(&codex_home);
    assert_temp_home(&auth_home);
    AppState::for_tests(codex_home, auth_home)
}

fn assert_temp_home(path: &Path) {
    let temp_root = std::env::temp_dir();
    assert!(
        path.starts_with(&temp_root),
        "test home must live under temp dir: {}",
        path.display()
    );
}

pub fn live_base_url() -> String {
    std::env::var("UMP_V2_LIVE_BASE_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:18743".to_string())
        .trim_end_matches('/')
        .to_string()
}

pub fn optional_env(name: &str) -> Option<String> {
    std::env::var(name)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

pub fn redact_sensitive_values(text: &str) -> String {
    let mut redacted = text.to_string();
    for name in [
        "AWS_BEARER_TOKEN_BEDROCK",
        "GOOGLE_API_KEY",
        "OPENAI_API_KEY",
    ] {
        if let Some(value) = optional_env(name) {
            if value.len() >= 8 {
                redacted = redacted.replace(&value, "[REDACTED]");
            }
        }
    }
    redacted
}

pub fn assert_no_unredacted_sensitive_values(text: &str) {
    for name in [
        "AWS_BEARER_TOKEN_BEDROCK",
        "GOOGLE_API_KEY",
        "OPENAI_API_KEY",
    ] {
        if let Some(value) = optional_env(name) {
            if value.len() >= 8 {
                assert!(
                    !text.contains(&value),
                    "output leaked sensitive env value from {name}"
                );
            }
        }
    }
}
