use std::{
    env,
    path::Path,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use base64::Engine;
use serde_json::json;
use tokio::process::Command;
use tokio::time::timeout;

use crate::{AppError, AppResult, AppState};

/// Default Cursor refresh exchange URL. Cross-checked with v1
/// `cursor-oauth.ts:5-7` which pins
/// `https://api2.cursor.sh/auth/exchange_user_api_key` as the canonical
/// endpoint and allows override via `CURSOR_REFRESH_URL`.
pub const CURSOR_REFRESH_URL: &str = "https://api2.cursor.sh/auth/exchange_user_api_key";

/// Environment variable consulted first for an access token.
pub const CURSOR_ACCESS_TOKEN_ENV: &str = "CURSOR_ACCESS_TOKEN";

/// Environment variable consulted to override the refresh exchange URL,
/// matching v1's `process.env.CURSOR_REFRESH_URL` override.
pub const CURSOR_REFRESH_URL_ENV: &str = "CURSOR_REFRESH_URL";

/// macOS Keychain item name and Linux libsecret service name.
const CURSOR_KEYCHAIN_ITEM: &str = "cursor-access-token";

/// Refresh-near-expiry slack matching the 5-minute window used by v1.
const REFRESH_NEAR_EXPIRY_SLACK: Duration = Duration::from_secs(5 * 60);

/// Shell-out timeout used for both `security` and `secret-tool`.
const SHELL_OUT_TIMEOUT: Duration = Duration::from_secs(1);

/// Diagnostic-only marker for which credential source produced the
/// currently active Cursor credentials. Carries no token material.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CursorAuthSource {
    /// Resolved from `CURSOR_ACCESS_TOKEN` env.
    Env,
    /// macOS Keychain via `security find-generic-password`.
    Keychain,
    /// Linux libsecret via `secret-tool lookup`.
    SecretTool,
    /// `<home>/.cursor/auth.json` with `accessToken` (and optional refresh).
    File,
    /// `<home>/.cursor/auth.json` carrying only an `apiKey` long-lived key.
    ApiKey,
}

impl CursorAuthSource {
    /// Stable string for tracing spans (`auth_source=keychain`, etc.).
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Env => "env",
            Self::Keychain => "keychain",
            Self::SecretTool => "secret_tool",
            Self::File => "file",
            Self::ApiKey => "api_key",
        }
    }
}

/// Active Cursor credentials. Token fields are stored as bare `String` to
/// match the existing project pattern (see `src/auth/codex.rs`); never log
/// these fields, only `source` and the SHA-256 prefix used as a cache key.
#[derive(Clone, Debug)]
pub struct CursorCredentials {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub api_key: Option<String>,
    pub source: CursorAuthSource,
    /// Wall-clock JWT expiry. `None` for non-JWT material (env, api_key) or
    /// when parsing failed.
    pub expires_at: Option<SystemTime>,
}

/// Resolve Cursor credentials from the documented sources, in order:
/// 1. `CURSOR_ACCESS_TOKEN` env (no refresh, no expiry).
/// 2. macOS Keychain item `cursor-access-token` (silent fall-through on err).
/// 3. Linux `secret-tool lookup service cursor account access-token`.
/// 4. `<home>/.cursor/auth.json` (`accessToken` + optional refresh, or only
///    `apiKey`).
///
/// Returns `AppError::MissingCredential("cursor")` if no source resolves.
pub async fn resolve_cursor_credentials(state: &AppState) -> AppResult<CursorCredentials> {
    resolve_cursor_credentials_uncached(state).await
}

pub async fn cached_cursor_credentials(state: &AppState) -> AppResult<CursorCredentials> {
    if let Some(creds) = state
        .cursor_auth
        .lock()
        .await
        .as_ref()
        .filter(|creds| !should_refresh(creds, SystemTime::now()))
        .cloned()
    {
        return Ok(creds);
    }

    let mut creds = resolve_cursor_credentials_uncached(state).await?;
    if should_refresh(&creds, SystemTime::now()) {
        match refresh_cursor_credentials(&state.warpsock, &creds).await {
            Ok(refreshed) => creds = refreshed,
            Err(err) => {
                tracing::debug!(error = %err, "cursor credential refresh failed; using resolved credentials")
            }
        }
    }
    let mut cached = state.cursor_auth.lock().await;
    if let Some(existing) = cached
        .as_ref()
        .filter(|existing| !should_refresh(existing, SystemTime::now()))
        .cloned()
    {
        return Ok(existing);
    }
    *cached = Some(creds.clone());
    Ok(creds)
}

pub async fn invalidate_cached_cursor_credentials(state: &AppState) {
    *state.cursor_auth.lock().await = None;
}

async fn resolve_cursor_credentials_uncached(state: &AppState) -> AppResult<CursorCredentials> {
    let home = state.auth_home.as_path();
    if let Some(token) = env_access_token() {
        return Ok(CursorCredentials {
            access_token: token,
            refresh_token: None,
            api_key: None,
            source: CursorAuthSource::Env,
            expires_at: None,
        });
    }

    if allow_system_secret_sources(home) {
        if cfg!(target_os = "macos") {
            if let Some(token) = keychain_access_token().await {
                let expires_at = parse_jwt_expiry(&token);
                return Ok(CursorCredentials {
                    access_token: token,
                    refresh_token: None,
                    api_key: None,
                    source: CursorAuthSource::Keychain,
                    expires_at,
                });
            }
        }

        if cfg!(target_os = "linux") {
            if let Some(token) = secret_tool_access_token().await {
                let expires_at = parse_jwt_expiry(&token);
                return Ok(CursorCredentials {
                    access_token: token,
                    refresh_token: None,
                    api_key: None,
                    source: CursorAuthSource::SecretTool,
                    expires_at,
                });
            }
        }
    }

    if let Some(creds) = file_access_credentials(state, home)? {
        return Ok(creds);
    }

    Err(AppError::MissingCredential("cursor"))
}

/// Decode the JWT middle segment and read the `exp` claim. Returns `None`
/// for any parsing failure or for non-JWT material; callers treat absent
/// expiry as "do not pre-emptively refresh".
pub fn parse_jwt_expiry(token: &str) -> Option<SystemTime> {
    let mut parts = token.split('.');
    let _header = parts.next()?;
    let payload = parts.next()?;
    if payload.is_empty() {
        return None;
    }
    parts.next()?;

    let bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(payload)
        .ok()?;
    let value: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
    let exp = value.get("exp")?.as_i64()?;
    if exp < 0 {
        return None;
    }
    UNIX_EPOCH.checked_add(Duration::from_secs(exp as u64))
}

/// True when `expires_at` is within the 5-minute safety window of `now` or
/// already past. Returns `false` when expiry is unknown.
pub fn should_refresh(creds: &CursorCredentials, now: SystemTime) -> bool {
    let Some(expires_at) = creds.expires_at else {
        return false;
    };
    match expires_at.duration_since(now) {
        Ok(remaining) => remaining <= REFRESH_NEAR_EXPIRY_SLACK,
        Err(_) => true,
    }
}

/// Exchange a refresh token for fresh credentials. POSTs to
/// `CURSOR_REFRESH_URL_ENV` if set, otherwise [`CURSOR_REFRESH_URL`].
///
/// Per v1 `cursor-oauth.ts:89-116`: send `Authorization: Bearer
/// <refreshToken>` with content-type `application/json` and a literal
/// `"{}"` body. On success the response carries
/// `{ accessToken, refreshToken }`; an empty/absent `refreshToken` falls
/// back to the previous one. The fresh access JWT drives `expires_at`.
///
/// Caller decides when to invoke; this helper does not loop. The refresh
/// retry budget is one attempt per request.
pub async fn refresh_cursor_credentials(
    client: &warpsock::Client,
    creds: &CursorCredentials,
) -> AppResult<CursorCredentials> {
    let refresh_token = creds
        .refresh_token
        .as_deref()
        .filter(|token| !token.trim().is_empty())
        .ok_or(AppError::MissingCredential("cursor refresh token"))?
        .to_string();

    let url = env::var(CURSOR_REFRESH_URL_ENV)
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| CURSOR_REFRESH_URL.to_string());

    let client = client.clone();
    let request_refresh_token = refresh_token.clone();
    let response = client
        .post(url)
        .bearer_auth(request_refresh_token)
        .header("content-type", "application/json")
        .body("{}")
        .send()
        .await
        .map_err(|err| AppError::Upstream(format!("Cursor refresh request failed: {err}")))?;

    let body: serde_json::Value = response.json()?;

    let access_token = body
        .get("accessToken")
        .and_then(|token| token.as_str())
        .filter(|token| !token.trim().is_empty())
        .ok_or(AppError::MissingCredential("cursor access token"))?
        .to_string();

    let next_refresh = body
        .get("refreshToken")
        .and_then(|token| token.as_str())
        .map(str::trim)
        .filter(|token| !token.is_empty())
        .map(ToOwned::to_owned)
        .or_else(|| Some(refresh_token.to_string()));

    let expires_at = parse_jwt_expiry(&access_token);

    Ok(CursorCredentials {
        access_token,
        refresh_token: next_refresh,
        api_key: creds.api_key.clone(),
        source: creds.source,
        expires_at,
    })
}

/// Build a 16-hex-character SHA-256 prefix to use as a cache or log key for
/// a Cursor token. Mirrors v1 `models.ts:128-131` so log artifacts line up.
pub fn token_cache_key(token: &str) -> String {
    use sha2::{Digest, Sha256};
    let digest = Sha256::digest(token.as_bytes());
    let hex = hex_lower(&digest);
    hex[..16].to_string()
}

fn hex_lower(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0x0f) as usize] as char);
    }
    out
}

fn env_access_token() -> Option<String> {
    let value = env::var(CURSOR_ACCESS_TOKEN_ENV).ok()?;
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn allow_system_secret_sources(home: &Path) -> bool {
    !home.starts_with(env::temp_dir())
}

async fn keychain_access_token() -> Option<String> {
    let mut command = Command::new("security");
    command
        .arg("find-generic-password")
        .arg("-s")
        .arg(CURSOR_KEYCHAIN_ITEM)
        .arg("-w");
    run_secret_command(command, "keychain").await
}

async fn secret_tool_access_token() -> Option<String> {
    let mut command = Command::new("secret-tool");
    command
        .arg("lookup")
        .arg("service")
        .arg("cursor")
        .arg("account")
        .arg("access-token");
    run_secret_command(command, "secret_tool").await
}

async fn run_secret_command(mut command: Command, source_label: &str) -> Option<String> {
    command.stdin(std::process::Stdio::null());
    command.stdout(std::process::Stdio::piped());
    command.stderr(std::process::Stdio::piped());
    command.kill_on_drop(true);

    let result = match timeout(SHELL_OUT_TIMEOUT, command.output()).await {
        Ok(result) => result,
        Err(_) => {
            tracing::debug!(auth_source = source_label, "cursor secret lookup timed out");
            return None;
        }
    };

    let output = match result {
        Ok(output) => output,
        Err(err) => {
            tracing::debug!(
                auth_source = source_label,
                error = %err,
                "cursor secret lookup failed to spawn",
            );
            return None;
        }
    };

    if !output.status.success() {
        tracing::debug!(
            auth_source = source_label,
            status = ?output.status.code(),
            "cursor secret lookup returned non-success",
        );
        return None;
    }

    let raw = String::from_utf8(output.stdout).ok()?;
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        tracing::debug!(
            auth_source = source_label,
            "cursor secret lookup returned empty value",
        );
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn file_access_credentials(state: &AppState, home: &Path) -> AppResult<Option<CursorCredentials>> {
    let path = home.join(".cursor").join("auth.json");
    let Some(value) = state.auth_files.get_or_load(&path)? else {
        return Ok(None);
    };

    let access_token = value
        .get("accessToken")
        .and_then(|token| token.as_str())
        .map(str::trim)
        .filter(|token| !token.is_empty())
        .map(ToOwned::to_owned);
    let refresh_token = value
        .get("refreshToken")
        .and_then(|token| token.as_str())
        .map(str::trim)
        .filter(|token| !token.is_empty())
        .map(ToOwned::to_owned);
    let api_key = value
        .get("apiKey")
        .and_then(|key| key.as_str())
        .map(str::trim)
        .filter(|key| !key.is_empty())
        .map(ToOwned::to_owned);

    if let Some(token) = access_token {
        let expires_at = parse_jwt_expiry(&token);
        return Ok(Some(CursorCredentials {
            access_token: token,
            refresh_token,
            api_key,
            source: CursorAuthSource::File,
            expires_at,
        }));
    }

    if let Some(key) = api_key {
        return Ok(Some(CursorCredentials {
            access_token: key.clone(),
            refresh_token: None,
            api_key: Some(key),
            source: CursorAuthSource::ApiKey,
            expires_at: None,
        }));
    }

    Ok(None)
}

/// Helper for downstream layers that want to surface auth diagnostics on a
/// missing-credential failure without leaking token material.
pub fn missing_credential_diagnostic() -> serde_json::Value {
    json!({
        "auth_source": "missing",
        "provider": "cursor",
    })
}
