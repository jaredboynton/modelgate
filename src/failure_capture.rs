use std::{fs, path::PathBuf};

use axum::http::{HeaderMap, StatusCode};
use serde_json::Value;
use uuid::Uuid;

use crate::{AppResult, AppState};

const REDACTED: &str = "[redacted]";
const FAILURE_DIR: &str = "v2-failures";

pub fn generate_request_id() -> String {
    Uuid::new_v4().to_string()
}

pub fn failure_dir(state: &AppState) -> PathBuf {
    state.auth_home.join(FAILURE_DIR)
}

pub fn failure_path(state: &AppState, request_id: &str, label: &str) -> AppResult<PathBuf> {
    let dir = failure_dir(state);
    fs::create_dir_all(&dir)?;
    Ok(dir.join(format!(
        "{}-{}.json",
        sanitize_filename_part(request_id),
        sanitize_filename_part(label)
    )))
}

pub fn write_failure_json(
    state: &AppState,
    request_id: &str,
    label: &str,
    mut value: Value,
) -> AppResult<PathBuf> {
    redact_failure_value(&mut value);
    let path = failure_path(state, request_id, label)?;
    fs::write(&path, serde_json::to_vec_pretty(&value)?)?;
    enforce_failure_cap(state, 100)?;
    Ok(path)
}

pub fn write_upstream_failure_json(
    state: &AppState,
    request_id: &str,
    provider: &str,
    status: StatusCode,
    headers: &HeaderMap,
    body: Value,
) -> AppResult<Option<PathBuf>> {
    if !status.is_server_error() {
        return Ok(None);
    }

    write_failure_json(
        state,
        request_id,
        &format!("{provider}-upstream-{}", status.as_u16()),
        serde_json::json!({
            "provider": provider,
            "status": status.as_u16(),
            "headers": header_map_json(headers),
            "body": body,
        }),
    )
    .map(Some)
}

pub fn write_oauth_failure_json(
    state: &AppState,
    request_id: &str,
    provider: &str,
    status: StatusCode,
    headers: &HeaderMap,
    body: Value,
) -> AppResult<Option<PathBuf>> {
    if status.is_success() {
        return Ok(None);
    }

    write_failure_json(
        state,
        request_id,
        &format!("{provider}-oauth-{}", status.as_u16()),
        serde_json::json!({
            "provider": provider,
            "status": status.as_u16(),
            "headers": header_map_json(headers),
            "body": body,
        }),
    )
    .map(Some)
}

pub fn list_failure_filenames(state: &AppState, cap: usize) -> AppResult<Vec<String>> {
    let dir = failure_dir(state);
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut names = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() || path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }
        if let Some(name) = path.file_name().and_then(|name| name.to_str()) {
            names.push(name.to_string());
        }
    }
    names.sort();
    names.truncate(cap);
    Ok(names)
}

pub fn enforce_failure_cap(state: &AppState, cap: usize) -> AppResult<()> {
    let dir = failure_dir(state);
    if !dir.exists() {
        return Ok(());
    }

    let mut entries = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() || path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }
        let modified = entry.metadata()?.modified().ok();
        entries.push((modified, path));
    }

    entries.sort_by(|left, right| left.0.cmp(&right.0).then_with(|| left.1.cmp(&right.1)));
    let remove_count = entries.len().saturating_sub(cap);
    for (_, path) in entries.into_iter().take(remove_count) {
        fs::remove_file(path)?;
    }
    Ok(())
}

pub fn redact_failure_value(value: &mut Value) {
    redact_value(value, None);
}

fn redact_value(value: &mut Value, parent_key: Option<&str>) {
    match value {
        Value::Object(map) => {
            for (key, child) in map {
                if should_redact_key(key, parent_key) {
                    *child = Value::String(REDACTED.to_string());
                } else {
                    redact_value(child, Some(key));
                }
            }
        }
        Value::Array(values) => {
            for child in values {
                redact_value(child, parent_key);
            }
        }
        Value::String(text) => {
            if should_redact_string(text) {
                *value = Value::String(REDACTED.to_string());
            } else if let Some(redacted) = redact_url_query(text) {
                *value = Value::String(redacted);
            }
        }
        _ => {}
    }
}

fn should_redact_string(text: &str) -> bool {
    is_base64_data_url(text) || looks_like_sdp(text)
}

fn is_base64_data_url(text: &str) -> bool {
    let lower = text
        .get(..text.len().min(128))
        .unwrap_or(text)
        .to_ascii_lowercase();
    lower.starts_with("data:") && lower.contains(";base64,")
}

fn looks_like_sdp(text: &str) -> bool {
    let trimmed = text.trim_start();
    trimmed.starts_with("v=0\r\n") || trimmed.starts_with("v=0\n")
}

fn redact_url_query(text: &str) -> Option<String> {
    let lower = text
        .get(..text.len().min(16))
        .unwrap_or(text)
        .to_ascii_lowercase();
    if !(lower.starts_with("wss://") || lower.starts_with("https://")) {
        return None;
    }

    let query_start = text.find('?')?;
    let fragment_start = text[query_start..]
        .find('#')
        .map(|offset| query_start + offset);
    let prefix = &text[..query_start];
    let suffix = fragment_start.map_or("", |index| &text[index..]);
    Some(format!("{prefix}?[redacted]{suffix}"))
}

fn should_redact_key(key: &str, parent_key: Option<&str>) -> bool {
    let normalized = normalize_key(key);
    let parent_normalized = parent_key.map(normalize_key);

    if parent_normalized.as_deref() == Some("headers") && is_sensitive_header_key(&normalized) {
        return true;
    }

    if parent_normalized.as_deref() == Some("inline_data") && normalized == "data" {
        return true;
    }

    if parent_normalized.as_deref() == Some("transcript")
        && matches!(normalized.as_str(), "text" | "delta")
    {
        return true;
    }

    matches!(
        normalized.as_str(),
        "access_token"
            | "refresh_token"
            | "id_token"
            | "bearer"
            | "api_key"
            | "apikey"
            | "client_secret"
            | "account_id"
            | "chatgpt_account_id"
            | "b64_json"
            | "base64"
            | "base64_payload"
            | "sdp"
            | "offer_sdp"
            | "answer_sdp"
            | "local_sdp"
            | "remote_sdp"
            | "audio"
            | "audio_bytes"
            | "audio_data"
            | "input_audio"
            | "input_audio_buffer"
            | "multipart"
            | "multipart_body"
            | "multipart_text"
            | "transcript"
            | "transcript_text"
            | "transcript_delta"
    )
}

fn is_sensitive_header_key(normalized: &str) -> bool {
    matches!(
        normalized,
        "authorization"
            | "proxy_authorization"
            | "x_api_key"
            | "x_goog_api_key"
            | "api_key"
            | "cookie"
            | "set_cookie"
            | "chatgpt_account_id"
    )
}

fn normalize_key(key: &str) -> String {
    key.to_ascii_lowercase().replace(['-', ' '], "_")
}

fn header_map_json(headers: &HeaderMap) -> Value {
    let mut value = serde_json::Map::new();
    for (name, header_value) in headers {
        value.insert(
            name.as_str().to_string(),
            header_value
                .to_str()
                .map_or_else(|_| "<non-utf8>".to_string(), ToOwned::to_owned)
                .into(),
        );
    }
    Value::Object(value)
}

fn sanitize_filename_part(value: &str) -> String {
    let sanitized: String = value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch
            } else {
                '-'
            }
        })
        .collect();

    sanitized.trim_matches('-').to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_failure_json_enforces_fifo_cap_of_100_json_files() {
        let temp = tempfile::tempdir().unwrap();
        let state = AppState::for_tests(temp.path().join("codex"), temp.path().join("ump"));

        for index in 0..105 {
            write_failure_json(
                &state,
                &format!("req-{index:03}"),
                "upstream",
                serde_json::json!({ "index": index }),
            )
            .unwrap();
        }

        let dir = failure_dir(&state);
        let mut files = fs::read_dir(dir)
            .unwrap()
            .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
            .collect::<Vec<_>>();
        files.sort();

        assert_eq!(files.len(), 100);
        assert!(!files.contains(&"req-000-upstream.json".to_string()));
        assert!(!files.contains(&"req-004-upstream.json".to_string()));
        assert!(files.contains(&"req-005-upstream.json".to_string()));
        assert!(files.contains(&"req-104-upstream.json".to_string()));
    }

    #[test]
    fn upstream_and_oauth_capture_skip_non_matching_statuses() {
        let temp = tempfile::tempdir().unwrap();
        let state = AppState::for_tests(temp.path().join("codex"), temp.path().join("ump"));
        let headers = HeaderMap::new();

        assert!(write_upstream_failure_json(
            &state,
            "req",
            "bedrock",
            StatusCode::BAD_REQUEST,
            &headers,
            serde_json::json!({})
        )
        .unwrap()
        .is_none());
        assert!(write_oauth_failure_json(
            &state,
            "req",
            "codex",
            StatusCode::OK,
            &headers,
            serde_json::json!({})
        )
        .unwrap()
        .is_none());
    }

    #[test]
    fn upstream_and_oauth_capture_redact_sensitive_headers() {
        let temp = tempfile::tempdir().unwrap();
        let state = AppState::for_tests(temp.path().join("codex"), temp.path().join("ump"));
        let mut headers = HeaderMap::new();
        headers.insert("authorization", "Bearer secret".parse().unwrap());

        let path = write_upstream_failure_json(
            &state,
            "req",
            "bedrock",
            StatusCode::BAD_GATEWAY,
            &headers,
            serde_json::json!({ "message": "failed" }),
        )
        .unwrap()
        .unwrap();
        let stored: Value = serde_json::from_slice(&fs::read(path).unwrap()).unwrap();

        assert_eq!(stored["headers"]["authorization"], "[redacted]");
        assert_eq!(stored["body"]["message"], "failed");
    }
}
