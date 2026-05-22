use std::{
    fs::{self, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::OnceLock,
};

use base64::Engine;
use fs2::FileExt;
use serde_json::json;
use sha2::{Digest, Sha256};
use tokio::sync::Mutex;

use crate::{AppError, AppResult, AppState};

pub const CODEX_CLIENT_ID: &str = "app_EMoamEEZ73f0CkXaXp7hrann";
pub const CODEX_ORIGINATOR: &str = "codex_cli_rs";
pub const CODEX_OPENAI_BETA: &str = "responses_websockets=2026-02-06";
pub const CODEX_OAUTH_TOKEN_URL: &str = "https://auth.openai.com/oauth/token";

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CodexAuth {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub id_token: Option<String>,
    pub account_id: Option<String>,
}

pub fn auth_path(state: &AppState) -> PathBuf {
    state.codex_home.join("auth.json")
}

pub fn load_codex_auth(state: &AppState) -> AppResult<CodexAuth> {
    let path = auth_path(state);
    let key = crate::auth::file_cache::AuthCacheKey::new().file(path.clone())?;
    state.codex_auth.get_or_try_insert_with(key, || {
        let value = state
            .auth_files
            .get_or_load(&path)?
            .ok_or(AppError::MissingCredential("~/.codex/auth.json"))?;
        parse_codex_auth_value(value.as_ref())
    })
}

pub fn parse_codex_auth(raw: &str) -> AppResult<CodexAuth> {
    let value: serde_json::Value = serde_json::from_str(raw)?;
    parse_codex_auth_value(&value)
}

fn parse_codex_auth_value(value: &serde_json::Value) -> AppResult<CodexAuth> {
    if let Some(tokens) = value.get("tokens") {
        return from_ump_tokens(tokens, value.get("account_id"));
    }

    // Try shorthand keys (standard in some codex tool variants)
    if let Some(access_token) = value
        .get("access")
        .and_then(|token| token.as_str())
        .filter(|token| !token.trim().is_empty())
    {
        let id_token = value
            .get("id")
            .or_else(|| value.get("id_token"))
            .and_then(|token| token.as_str())
            .map(ToOwned::to_owned);
        let account_id = value
            .get("accountId")
            .or_else(|| value.get("account_id"))
            .and_then(|id| id.as_str())
            .map(ToOwned::to_owned)
            .or_else(|| id_token.as_deref().and_then(account_id_from_id_token));
        return Ok(CodexAuth {
            access_token: access_token.to_string(),
            refresh_token: value
                .get("refresh")
                .or_else(|| value.get("refresh_token"))
                .and_then(|token| token.as_str())
                .map(ToOwned::to_owned),
            id_token,
            account_id,
        });
    }

    from_ump_tokens(value, value.get("account_id"))
}

fn from_ump_tokens(
    tokens: &serde_json::Value,
    outer_account_id: Option<&serde_json::Value>,
) -> AppResult<CodexAuth> {
    let access_token = tokens
        .get("access_token")
        .and_then(|token| token.as_str())
        .filter(|token| !token.trim().is_empty())
        .ok_or(AppError::MissingCredential("codex access token"))?;

    let id_token = tokens
        .get("id_token")
        .and_then(|token| token.as_str())
        .map(ToOwned::to_owned);
    let account_id = tokens
        .get("account_id")
        .or(outer_account_id)
        .and_then(|id| id.as_str())
        .map(ToOwned::to_owned)
        .or_else(|| id_token.as_deref().and_then(account_id_from_id_token));

    Ok(CodexAuth {
        access_token: access_token.to_string(),
        refresh_token: tokens
            .get("refresh_token")
            .and_then(|token| token.as_str())
            .map(ToOwned::to_owned),
        id_token,
        account_id,
    })
}

fn account_id_from_id_token(id_token: &str) -> Option<String> {
    let payload = id_token.split('.').nth(1)?;
    let decoded = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(payload)
        .or_else(|_| base64::engine::general_purpose::URL_SAFE.decode(payload))
        .ok()?;
    let value: serde_json::Value = serde_json::from_slice(&decoded).ok()?;
    [
        "account_id",
        "accountId",
        "chatgpt_account_id",
        "https://api.openai.com/auth/account_id",
    ]
    .into_iter()
    .find_map(|key| {
        value
            .get(key)
            .and_then(|id| id.as_str())
            .filter(|id| !id.trim().is_empty())
            .map(ToOwned::to_owned)
    })
}

pub async fn refresh_codex_auth(state: &AppState) -> AppResult<CodexAuth> {
    refresh_codex_auth_with_endpoint(&state.specter, state, CODEX_OAUTH_TOKEN_URL).await
}

pub async fn refresh_codex_auth_with_endpoint(
    client: &specter::Client,
    state: &AppState,
    token_url: &str,
) -> AppResult<CodexAuth> {
    static REFRESH_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    let _guard = REFRESH_LOCK.get_or_init(|| Mutex::new(())).lock().await;

    let path = auth_path(state);
    let before = read_auth_snapshot(&path)?;
    let auth =
        parse_codex_auth(std::str::from_utf8(&before.bytes).map_err(|err| {
            AppError::BadRequest(format!("codex auth file is not utf-8: {err}"))
        })?)?;
    let refresh_token = auth
        .refresh_token
        .as_deref()
        .filter(|token| !token.trim().is_empty())
        .ok_or(AppError::MissingCredential("codex refresh token"))?;

    let response = client
        .post(token_url)
        .form(&[
            ("grant_type", "refresh_token"),
            ("client_id", CODEX_CLIENT_ID),
            ("refresh_token", refresh_token),
        ])
        .send()
        .await
        .map_err(|err| AppError::Upstream(format!("Codex OAuth refresh failed: {err}")))?;
    if !response.status().is_success() {
        return Err(AppError::Upstream(format!(
            "Codex OAuth refresh returned {}",
            response.status()
        )));
    }

    let token_body: serde_json::Value = response
        .json()
        .map_err(|err| AppError::Upstream(format!("Codex OAuth refresh body failed: {err}")))?;
    let refreshed = merge_refreshed_auth(auth, token_body)?;
    let current = read_auth_snapshot(&path)?;
    if current.hash != before.hash {
        state.auth_files.invalidate(&path);
        state.codex_auth.invalidate();
        return load_codex_auth(state);
    }

    write_codex_auth(state, &refreshed)?;
    state.auth_files.invalidate(&path);
    state.codex_auth.invalidate();
    Ok(refreshed)
}

fn merge_refreshed_auth(
    previous: CodexAuth,
    token_body: serde_json::Value,
) -> AppResult<CodexAuth> {
    let access_token = token_body
        .get("access_token")
        .and_then(|token| token.as_str())
        .filter(|token| !token.trim().is_empty())
        .ok_or(AppError::MissingCredential("codex access token"))?
        .to_string();
    let refresh_token = token_body
        .get("refresh_token")
        .and_then(|token| token.as_str())
        .filter(|token| !token.trim().is_empty())
        .map(ToOwned::to_owned)
        .or(previous.refresh_token);
    let id_token = token_body
        .get("id_token")
        .and_then(|token| token.as_str())
        .filter(|token| !token.trim().is_empty())
        .map(ToOwned::to_owned)
        .or(previous.id_token);
    let account_id = token_body
        .get("account_id")
        .and_then(|id| id.as_str())
        .filter(|id| !id.trim().is_empty())
        .map(ToOwned::to_owned)
        .or_else(|| id_token.as_deref().and_then(account_id_from_id_token))
        .or(previous.account_id);

    Ok(CodexAuth {
        access_token,
        refresh_token,
        id_token,
        account_id,
    })
}

fn write_codex_auth(state: &AppState, auth: &CodexAuth) -> AppResult<()> {
    let path = auth_path(state);
    let lock_path = state.codex_home.join("auth.json.lock");
    fs::create_dir_all(state.codex_home.as_path())?;
    let lock = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(lock_path)?;
    lock.lock_exclusive()?;

    let result = (|| {
        let auth_json = json!({
            "access_token": auth.access_token,
            "refresh_token": auth.refresh_token,
            "id_token": auth.id_token,
            "account_id": auth.account_id,
        });
        atomic_write_private(&path, serde_json::to_vec_pretty(&auth_json)?.as_slice())?;
        write_diagnostic_mirror(state, auth)
    })();

    let unlock_result = lock.unlock();
    result?;
    unlock_result?;
    Ok(())
}

fn write_diagnostic_mirror(state: &AppState, auth: &CodexAuth) -> AppResult<()> {
    let path = state.auth_home.join("auth.json");
    let mut value = match fs::read_to_string(&path) {
        Ok(raw) => serde_json::from_str(&raw).unwrap_or_else(|_| json!({})),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => json!({}),
        Err(err) => return Err(AppError::Io(err)),
    };
    if !value.is_object() {
        value = json!({});
    }
    value["codex"] = json!({
        "account_id": auth.account_id,
        "has_access_token": !auth.access_token.trim().is_empty(),
        "has_refresh_token": auth.refresh_token.as_deref().is_some_and(|token| !token.trim().is_empty()),
        "originator": CODEX_ORIGINATOR,
    });
    atomic_write_private(&path, serde_json::to_vec_pretty(&value)?.as_slice())
}

struct AuthSnapshot {
    bytes: Vec<u8>,
    hash: Vec<u8>,
}

fn read_auth_snapshot(path: &Path) -> AppResult<AuthSnapshot> {
    let mut file = OpenOptions::new().read(true).open(path).map_err(|err| {
        if err.kind() == std::io::ErrorKind::NotFound {
            AppError::MissingCredential("~/.codex/auth.json")
        } else {
            AppError::Io(err)
        }
    })?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)?;
    let hash = Sha256::digest(&bytes).to_vec();
    Ok(AuthSnapshot { bytes, hash })
}

fn atomic_write_private(path: &Path, bytes: &[u8]) -> AppResult<()> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent)?;
    let tmp_path = parent.join(format!(
        ".{}.{}.tmp",
        path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("auth.json"),
        uuid::Uuid::new_v4()
    ));

    {
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let mut file = options.open(&tmp_path)?;
        file.write_all(bytes)?;
        file.sync_all()?;
    }

    fs::rename(&tmp_path, path)?;
    let parent_dir = OpenOptions::new().read(true).open(parent)?;
    parent_dir.sync_all()?;
    Ok(())
}
