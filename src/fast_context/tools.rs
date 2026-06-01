use std::{
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::{AppError, AppResult};

use super::sandbox::RepoSandbox;
use super::SearchType;

const MAX_FILE_BYTES: u64 = 256 * 1024;
const MAX_SNIPPET_LINES: usize = 12;

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct ContextSnippet {
    pub path: String,
    pub start_line: usize,
    pub end_line: usize,
    pub text: String,
    pub score: usize,
}

pub fn search_repo(
    sandbox: &RepoSandbox,
    query: &str,
    max_files: usize,
    search_type: SearchType,
) -> AppResult<Vec<ContextSnippet>> {
    let terms = query_terms(query);
    if terms.is_empty() {
        return Err(AppError::BadRequest(
            "query must include searchable text".into(),
        ));
    }

    let mut files = Vec::new();
    collect_files(
        sandbox.root(),
        &mut files,
        max_files.saturating_mul(64).max(64),
        search_type,
        true,
    )?;

    let mut snippets = Vec::new();
    for path in files {
        let metadata = fs::metadata(&path)?;
        if metadata.len() > MAX_FILE_BYTES {
            continue;
        }
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        if let Some(snippet) = best_snippet(sandbox, &path, &text, &terms) {
            snippets.push(snippet);
        }
    }

    snippets.sort_by(|left, right| {
        right
            .score
            .cmp(&left.score)
            .then_with(|| left.path.cmp(&right.path))
    });
    snippets.truncate(max_files);
    Ok(snippets)
}

pub fn execute_tool(
    sandbox: &RepoSandbox,
    name: &str,
    arguments: &serde_json::Value,
    max_files: usize,
) -> AppResult<serde_json::Value> {
    match name {
        "grep_search" | "find_by_name" => {
            let query = arguments
                .get("query")
                .or_else(|| arguments.get("pattern"))
                .and_then(serde_json::Value::as_str)
                .ok_or_else(|| AppError::BadRequest(format!("{name} requires query or pattern")))?;
            let snippets = search_repo(sandbox, query, max_files, SearchType::All)?;
            Ok(serde_json::json!({ "snippets": snippets }))
        }
        "read_file" | "view_content_chunk" => {
            let path = arguments
                .get("path")
                .or_else(|| arguments.get("file_path"))
                .and_then(serde_json::Value::as_str)
                .ok_or_else(|| AppError::BadRequest(format!("{name} requires path")))?;
            let resolved = sandbox.resolve_existing(path)?;
            let metadata = fs::metadata(&resolved)?;
            if metadata.len() > MAX_FILE_BYTES {
                return Err(AppError::BadRequest(format!("file too large: {path}")));
            }
            let text = fs::read_to_string(&resolved)?;
            Ok(serde_json::json!({
                "path": sandbox.relative_display(&resolved),
                "text": text
            }))
        }
        "list_dir" => {
            let path = arguments
                .get("path")
                .or_else(|| arguments.get("directory"))
                .and_then(serde_json::Value::as_str)
                .unwrap_or(".");
            let resolved = sandbox.resolve_existing(path)?;
            let mut entries = Vec::new();
            for entry in fs::read_dir(&resolved)? {
                let entry = entry?;
                entries.push(sandbox.relative_display(&entry.path()));
            }
            entries.sort();
            entries.truncate(max_files);
            Ok(serde_json::json!({ "entries": entries }))
        }
        _ => Err(AppError::BadRequest(format!("TOOL_NOT_AVAILABLE: {name}"))),
    }
}

fn query_terms(query: &str) -> Vec<String> {
    query
        .split(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_' || ch == '-'))
        .map(str::trim)
        .filter(|term| term.len() >= 3)
        .map(|term| term.to_ascii_lowercase())
        .collect()
}

fn collect_files(
    dir: &Path,
    out: &mut Vec<PathBuf>,
    limit: usize,
    search_type: SearchType,
    is_root: bool,
) -> AppResult<()> {
    if out.len() >= limit || (!is_root && should_skip_dir(dir, search_type)) {
        return Ok(());
    }
    let mut entries = fs::read_dir(dir)?.collect::<Result<Vec<_>, _>>()?;
    entries.sort_by_key(|entry| entry.path());
    for entry in entries {
        if out.len() >= limit {
            break;
        }
        let path = entry.path();
        let file_type = entry.file_type()?;
        if file_type.is_symlink() {
            continue;
        }
        if file_type.is_dir() {
            collect_files(&path, out, limit, search_type, false)?;
        } else if file_type.is_file()
            && looks_textual(&path)
            && search_type_allows(&path, search_type)
        {
            out.push(path);
        }
    }
    Ok(())
}

fn should_skip_dir(path: &Path, search_type: SearchType) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| {
            if name.starts_with('.') {
                return true;
            }
            if name == "node_modules" {
                return search_type != SearchType::NodeModules;
            }
            matches!(name, ".git" | "target" | ".next" | ".cache" | ".omc")
        })
}

fn search_type_allows(path: &Path, search_type: SearchType) -> bool {
    match search_type {
        SearchType::All => true,
        SearchType::NodeModules => path
            .components()
            .any(|component| component.as_os_str().to_string_lossy().as_ref() == "node_modules"),
    }
}

fn looks_textual(path: &Path) -> bool {
    !path
        .extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| {
            matches!(
                ext.to_ascii_lowercase().as_str(),
                "png" | "jpg" | "jpeg" | "gif" | "webp" | "pdf" | "bin" | "lock"
            )
        })
}

fn best_snippet(
    sandbox: &RepoSandbox,
    path: &Path,
    text: &str,
    terms: &[String],
) -> Option<ContextSnippet> {
    let lines = text.lines().collect::<Vec<_>>();
    let relative_path = sandbox.relative_display(path);
    let path_haystack = relative_path.to_ascii_lowercase();
    let path_score = terms
        .iter()
        .filter(|term| path_haystack.contains(term.as_str()))
        .count()
        * 2;
    let mut best_line = None;
    let mut best_score = 0usize;
    for (idx, line) in lines.iter().enumerate() {
        let haystack = line.to_ascii_lowercase();
        let score = path_score
            + terms
                .iter()
                .filter(|term| haystack.contains(term.as_str()))
                .count();
        if score > best_score {
            best_score = score;
            best_line = Some(idx);
        }
    }
    let best_line = best_line.or((path_score > 0).then_some(0))?;
    let best_score = best_score.max(path_score);
    let start = best_line.saturating_sub(3);
    let end = (best_line + MAX_SNIPPET_LINES).min(lines.len());
    let text = lines[start..end].join("\n");
    Some(ContextSnippet {
        path: relative_path,
        start_line: start + 1,
        end_line: end,
        text,
        score: best_score,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_returns_matching_snippet() {
        let temp = tempfile::tempdir().unwrap();
        fs::write(temp.path().join("probe.rs"), "fn assign_model_probe() {}\n").unwrap();
        let sandbox = RepoSandbox::new(temp.path()).unwrap();

        let snippets = search_repo(&sandbox, "assign model probe", 4, SearchType::All).unwrap();

        assert_eq!(snippets[0].path, "probe.rs");
    }

    #[test]
    fn denies_unavailable_tools() {
        let temp = tempfile::tempdir().unwrap();
        let sandbox = RepoSandbox::new(temp.path()).unwrap();

        let error = execute_tool(&sandbox, "bash", &serde_json::json!({}), 4).unwrap_err();

        assert!(error.to_string().contains("TOOL_NOT_AVAILABLE"));
    }
}
