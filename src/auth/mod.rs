use std::{fs, path::Path};

use crate::AppResult;

pub mod bedrock;
pub mod codex;
pub mod cursor;
pub mod google;
pub mod windsurf;

pub(crate) fn read_json_string(path: &Path, keys: &[&str]) -> AppResult<Option<String>> {
    if !path.exists() {
        return Ok(None);
    }
    let value: serde_json::Value = serde_json::from_slice(&fs::read(path)?)?;
    let mut cursor = &value;
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
