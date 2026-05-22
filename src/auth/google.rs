use crate::{
    auth::{file_cache::AuthCacheKey, read_json_string},
    AppError, AppResult, AppState,
};

pub fn api_key(state: &AppState) -> AppResult<String> {
    let auth_json = state.auth_home.join("auth.json");
    let key = AuthCacheKey::new()
        .file(auth_json.clone())?
        .value("state.google_api_key", state.google_api_key.as_deref());
    state.google_auth.get_or_try_insert_with(key, || {
        if let Some(key) = read_json_string(state, &auth_json, &["gemini", "api_key"])? {
            return Ok(key);
        }
        if let Some(key) = read_json_string(state, &auth_json, &["google", "api_key"])? {
            return Ok(key);
        }

        state
            .google_api_key
            .as_ref()
            .map(|value| value.to_string())
            .ok_or(AppError::MissingCredential("GOOGLE_API_KEY"))
    })
}
