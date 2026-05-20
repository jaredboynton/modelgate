use std::env;

use crate::{auth::read_json_string, AppError, AppResult, AppState};

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum BedrockAuth {
    Bearer { token: String, source: &'static str },
}

pub fn resolve_bedrock_auth(state: &AppState) -> AppResult<BedrockAuth> {
    let auth_json = state.auth_home.join("auth.json");
    if let Some(token) = read_json_string(&auth_json, &["bedrock", "bearer"])? {
        return Ok(BedrockAuth::Bearer {
            token,
            source: "bearer_file",
        });
    }

    if let Ok(token) = env::var("AWS_BEARER_TOKEN_BEDROCK") {
        if !token.trim().is_empty() {
            return Ok(BedrockAuth::Bearer {
                token,
                source: "bearer_env",
            });
        }
    }

    Err(AppError::MissingCredential("Bedrock bearer"))
}
