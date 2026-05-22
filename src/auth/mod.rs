use std::path::Path;

use crate::{AppResult, AppState};

pub mod bedrock;
pub mod codex;
pub mod cursor;
pub mod file_cache;
pub mod google;
pub mod windsurf;

pub(crate) fn read_json_string(
    state: &AppState,
    path: &Path,
    keys: &[&str],
) -> AppResult<Option<String>> {
    let Some(value) = state.auth_files.get_or_load(path)? else {
        return Ok(None);
    };
    let mut cursor: &serde_json::Value = value.as_ref();
    for key in keys {
        let Some(next) = cursor.get(*key) else {
            return Ok(None);
        };
        cursor = next;
    }
    Ok(cursor
        .as_str()
        .filter(|value| !value.trim().is_empty())
        .map(ToOwned::to_owned))
}
