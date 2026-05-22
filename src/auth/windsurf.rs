use std::env;

use crate::{
    auth::{file_cache::AuthCacheKey, read_json_string},
    AppError, AppResult, AppState,
};

pub fn api_key(state: &AppState) -> AppResult<String> {
    let auth_json = state.auth_home.join("auth.json");
    let auth_home_legacy = state.auth_home.join("windsurf/auth.json");
    let dot_windsurf_legacy =
        if state.auth_home.file_name().and_then(|value| value.to_str()) == Some(".ump") {
            state
                .auth_home
                .parent()
                .map(|home| home.join(".windsurf/auth.json"))
        } else {
            None
        };
    let cache_key = {
        let key = AuthCacheKey::new()
            .file(auth_json.clone())?
            .file(auth_home_legacy.clone())?
            .env("WINDSURF_API_KEY");
        match dot_windsurf_legacy.as_ref() {
            Some(path) => key.file(path.clone())?,
            None => key,
        }
    };

    state.windsurf_auth.get_or_try_insert_with(cache_key, || {
        if let Some(key) = read_json_string(state, &auth_json, &["windsurf", "api_key"])? {
            return Ok(key);
        }
        if let Some(key) = read_json_string(state, &auth_json, &["windsurf", "apiKey"])? {
            return Ok(key);
        }

        if let Some(key) = read_json_string(state, &auth_home_legacy, &["apiKey"])? {
            return Ok(key);
        }
        if let Some(key) = read_json_string(state, &auth_home_legacy, &["api_key"])? {
            return Ok(key);
        }

        if let Some(legacy) = dot_windsurf_legacy.as_ref() {
            if let Some(key) = read_json_string(state, legacy, &["apiKey"])? {
                return Ok(key);
            }
            if let Some(key) = read_json_string(state, legacy, &["api_key"])? {
                return Ok(key);
            }
        }

        if let Ok(key) = env::var("WINDSURF_API_KEY") {
            if !key.trim().is_empty() {
                return Ok(key.trim().to_string());
            }
        }

        Err(AppError::MissingCredential("WINDSURF_API_KEY"))
    })
}
