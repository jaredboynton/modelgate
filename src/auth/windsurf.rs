use std::env;

use crate::{auth::read_json_string, AppError, AppResult, AppState};

pub fn api_key(state: &AppState) -> AppResult<String> {
    let auth_json = state.auth_home.join("auth.json");
    if let Some(key) = read_json_string(&auth_json, &["windsurf", "api_key"])? {
        return Ok(key);
    }
    if let Some(key) = read_json_string(&auth_json, &["windsurf", "apiKey"])? {
        return Ok(key);
    }

    let auth_home_legacy = state.auth_home.join("windsurf/auth.json");
    if let Some(key) = read_json_string(&auth_home_legacy, &["apiKey"])? {
        return Ok(key);
    }
    if let Some(key) = read_json_string(&auth_home_legacy, &["api_key"])? {
        return Ok(key);
    }

    if state.auth_home.file_name().and_then(|value| value.to_str()) == Some(".ump") {
        if let Some(home) = state.auth_home.parent() {
            let legacy = home.join(".windsurf/auth.json");
            if let Some(key) = read_json_string(&legacy, &["apiKey"])? {
                return Ok(key);
            }
            if let Some(key) = read_json_string(&legacy, &["api_key"])? {
                return Ok(key);
            }
        }
    }

    if let Ok(key) = env::var("WINDSURF_API_KEY") {
        if !key.trim().is_empty() {
            return Ok(key.trim().to_string());
        }
    }

    Err(AppError::MissingCredential("WINDSURF_API_KEY"))
}
