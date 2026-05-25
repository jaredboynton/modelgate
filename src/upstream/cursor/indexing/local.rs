//! Local-fallback workspace search.
//!
//! Bounded lexical scan used when the cloud index is disabled, returns no
//! results, or fails. Mirrors `cursor-index.ts` `readRecords` +
//! `renderSearchResults` semantics: tokenized query, score by
//! path/contents matches, cap at 50 hits, 256 KB per snippet.
//!
//! On no results the function emits the literal sentinel
//! `"found no matching files"`. The context-injection layer greps for that
//! exact substring to suppress empty injections (`context-injection.ts:198`).

use std::cmp::Reverse;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{Arc, Mutex, OnceLock},
    time::{Duration, Instant},
};

use tokio::fs;
use tokio::task;

use crate::upstream::cursor::workspace::{enforce_allowlist, is_within_directory};

/// Empty-result sentinel preserved verbatim from the TS plugin.
pub const EMPTY_RESULT_SENTINEL: &str = "found no matching files";

/// Default upper bound on hits returned by `local_search`.
pub const DEFAULT_MAX_HITS: usize = 50;

/// Maximum bytes per excerpt rendered in a search hit.
pub const MAX_SNIPPET_BYTES: usize = 256 * 1024;

/// Hard cap on records walked per search to bound memory.
pub const MAX_RECORDS: usize = 500;

/// Per-file size cap. Matches TS `readRecords` (`size <= 256_000`).
pub const MAX_FILE_BYTES: u64 = 256_000;

const RECORD_CACHE_TTL: Duration = Duration::from_secs(30);

/// Default text extensions walked. Mirrors TS `TEXT_EXTENSIONS`.
pub const TEXT_EXTENSIONS: &[&str] = &[
    "cjs", "css", "go", "html", "js", "json", "jsonc", "jsx", "md", "mjs", "py", "rs", "sh", "sql",
    "ts", "tsx", "txt", "yaml", "yml",
];

/// Default ignore set. Aligned with the upload superset
/// (`indexing-extraction.md` "DEFAULT_IGNORES").
pub const DEFAULT_IGNORES: &[&str] = &[
    ".git",
    ".hg",
    ".svn",
    ".cursor",
    ".omc",
    ".omx",
    ".sisyphus",
    ".live-harness",
    ".wire-harness-runs",
    "node_modules",
    "target",
    "dist",
    "build",
    ".next",
    ".turbo",
    "coverage",
];

/// Search hit returned by `local_search`.
#[derive(Debug, Clone)]
pub struct SearchHit {
    pub path: String,
    pub score: u32,
    pub excerpt: String,
}

#[derive(Clone)]
struct FileRecord {
    relative_path: String,
    text: String,
}

#[derive(Clone)]
struct RecordCacheEntry {
    fetched_at: Instant,
    records: Arc<Vec<FileRecord>>,
}

fn record_cache() -> &'static Mutex<HashMap<PathBuf, RecordCacheEntry>> {
    static CELL: OnceLock<Mutex<HashMap<PathBuf, RecordCacheEntry>>> = OnceLock::new();
    CELL.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Run a bounded local search rooted at `workspace`. Errors during the walk
/// silently collapse to fewer results; this function never returns an
/// error. The allowlist gate is enforced before any filesystem read; an
/// empty allowlist returns `Vec::new()`.
pub async fn local_search(workspace: &Path, query: &str, allowlist: &[PathBuf]) -> Vec<SearchHit> {
    let canonical = match enforce_allowlist(workspace, allowlist) {
        Ok(canonical) => canonical,
        Err(_) => return Vec::new(),
    };
    let tokens = tokenize(query);
    if tokens.is_empty() {
        return Vec::new();
    }
    let records = cached_records(canonical).await;
    let _ = fs::metadata(workspace).await; // touch to stay tokio-aware
    rank_and_render(&records, &tokens, DEFAULT_MAX_HITS)
}

async fn cached_records(canonical: PathBuf) -> Arc<Vec<FileRecord>> {
    if let Some(records) = lookup_record_cache(&canonical) {
        return records;
    }
    let cache_key = canonical.clone();
    let records =
        match task::spawn_blocking(move || read_records_blocking(&canonical, MAX_RECORDS)).await {
            Ok(records) => Arc::new(records),
            Err(_) => Arc::new(Vec::new()),
        };
    store_record_cache(cache_key, Arc::clone(&records));
    records
}

fn lookup_record_cache(canonical: &Path) -> Option<Arc<Vec<FileRecord>>> {
    let mut guard = record_cache().lock().ok()?;
    if let Some(entry) = guard.get(canonical) {
        if entry.fetched_at.elapsed() < RECORD_CACHE_TTL {
            return Some(Arc::clone(&entry.records));
        }
    }
    guard.remove(canonical);
    None
}

fn store_record_cache(canonical: PathBuf, records: Arc<Vec<FileRecord>>) {
    if let Ok(mut guard) = record_cache().lock() {
        guard.insert(
            canonical,
            RecordCacheEntry {
                fetched_at: Instant::now(),
                records,
            },
        );
        if guard.len() > 32 {
            guard.retain(|_, entry| entry.fetched_at.elapsed() < RECORD_CACHE_TTL);
        }
        if guard.len() > 32 {
            guard.clear();
        }
    }
}

/// Render the body of a search response. When `hits` is empty the function
/// returns `EMPTY_RESULT_SENTINEL`-bearing text so the context-injection
/// layer recognizes it.
pub fn render_local_body(hits: &[SearchHit]) -> String {
    if hits.is_empty() {
        return format!(
            "Cursor index search is not available from this plugin runtime yet.\nLocal fallback search {EMPTY_RESULT_SENTINEL}."
        );
    }
    let mut sections: Vec<String> = Vec::with_capacity(hits.len());
    for hit in hits {
        sections.push(format!(
            "### {} (score {})\n```\n{}\n```",
            hit.path, hit.score, hit.excerpt
        ));
    }
    sections.join("\n\n")
}

fn tokenize(input: &str) -> Vec<String> {
    let lowered = input.to_lowercase();
    lowered
        .split(|c: char| !(c.is_ascii_alphanumeric() || c == '_' || c == '.' || c == '-'))
        .filter(|token| token.len() > 2)
        .map(str::to_owned)
        .collect()
}

fn rank_and_render(records: &[FileRecord], tokens: &[String], max_hits: usize) -> Vec<SearchHit> {
    let mut scored: Vec<(u32, &FileRecord)> = Vec::new();
    for record in records {
        let score = score_record(tokens, record);
        if score > 0 {
            scored.push((score, record));
        }
    }
    scored.sort_by_key(|(score, _)| Reverse(*score));
    let mut hits = Vec::new();
    for (score, record) in scored.into_iter().take(max_hits) {
        let excerpt = build_excerpt(&record.text, tokens);
        hits.push(SearchHit {
            path: record.relative_path.clone(),
            score,
            excerpt,
        });
    }
    hits
}

fn score_record(tokens: &[String], record: &FileRecord) -> u32 {
    let path_lower = record.relative_path.to_lowercase();
    let body_window = record.text.chars().take(12_000).collect::<String>();
    let haystack = format!("{}\n{}", path_lower, body_window.to_lowercase());
    let mut score: u32 = 0;
    for token in tokens {
        if path_lower.contains(token) {
            score = score.saturating_add(5);
        }
        let matches = haystack.matches(token).count() as u32;
        score = score.saturating_add(matches.min(8));
    }
    score
}

fn build_excerpt(text: &str, tokens: &[String]) -> String {
    let lines: Vec<&str> = text.split('\n').collect();
    let first_hit = lines
        .iter()
        .position(|line| {
            let lower = line.to_lowercase();
            tokens.iter().any(|token| lower.contains(token))
        })
        .unwrap_or(0);
    let end = (first_hit + 8).min(lines.len());
    let joined = lines[first_hit..end].join("\n");
    if joined.len() > 1_500 {
        joined.chars().take(1_500).collect()
    } else {
        joined
    }
}

fn read_records_blocking(root: &Path, limit: usize) -> Vec<FileRecord> {
    let mut records: Vec<FileRecord> = Vec::new();
    walk(root, root, limit, &mut records);
    records
}

fn walk(workspace_root: &Path, dir: &Path, limit: usize, records: &mut Vec<FileRecord>) {
    if records.len() >= limit {
        return;
    }
    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return,
    };
    let mut sorted: Vec<std::fs::DirEntry> = entries.filter_map(Result::ok).collect();
    sorted.sort_by_key(|a| a.file_name());
    for entry in sorted {
        if records.len() >= limit {
            return;
        }
        let name = entry.file_name();
        let name_str = match name.to_str() {
            Some(value) => value,
            None => continue,
        };
        if DEFAULT_IGNORES.contains(&name_str) {
            continue;
        }
        let path = entry.path();
        let metadata = match entry.metadata() {
            Ok(meta) => meta,
            Err(_) => continue,
        };
        if metadata.file_type().is_symlink() {
            continue;
        }
        let real_path = match std::fs::canonicalize(&path) {
            Ok(value) => value,
            Err(_) => continue,
        };
        if !is_within_directory(workspace_root, &real_path) {
            continue;
        }
        if metadata.is_dir() {
            walk(workspace_root, &real_path, limit, records);
        } else if metadata.is_file() && metadata.len() <= MAX_FILE_BYTES && is_text_file(&real_path)
        {
            if let Ok(contents) = std::fs::read_to_string(&real_path) {
                let relative = match real_path.strip_prefix(workspace_root) {
                    Ok(path) => path.to_string_lossy().into_owned(),
                    Err(_) => real_path.to_string_lossy().into_owned(),
                };
                if contents.len() <= MAX_SNIPPET_BYTES {
                    records.push(FileRecord {
                        relative_path: relative,
                        text: contents,
                    });
                }
            }
        }
    }
}

fn is_text_file(path: &Path) -> bool {
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(str::to_lowercase);
    match extension {
        Some(value) => TEXT_EXTENSIONS.iter().any(|known| *known == value),
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_local_body_emits_sentinel_when_empty() {
        let body = render_local_body(&[]);
        assert!(body.contains(EMPTY_RESULT_SENTINEL));
    }

    #[test]
    fn render_local_body_formats_hits() {
        let hit = SearchHit {
            path: "src/lib.rs".into(),
            score: 7,
            excerpt: "fn main() {}".into(),
        };
        let body = render_local_body(&[hit]);
        assert!(body.contains("### src/lib.rs (score 7)"));
        assert!(body.contains("```\nfn main() {}\n```"));
    }

    #[test]
    fn tokenize_drops_short_tokens_and_lowercases() {
        let tokens = tokenize("Hello, World! foo bar baz!!!");
        assert!(tokens.contains(&"hello".to_string()));
        assert!(tokens.contains(&"world".to_string()));
        assert!(tokens.contains(&"foo".to_string()));
        assert!(tokens.contains(&"bar".to_string()));
        assert!(tokens.contains(&"baz".to_string()));
    }

    #[test]
    fn local_walk_skips_runtime_artifacts_before_record_limit() {
        let temp = tempfile::tempdir().expect("tempdir");
        let root = temp.path();
        std::fs::create_dir(root.join(".live-harness")).expect("live dir");
        for index in 0..(MAX_RECORDS + 20) {
            std::fs::write(
                root.join(".live-harness").join(format!("noise-{index}.rs")),
                "struct Noise;",
            )
            .expect("write noise");
        }
        std::fs::create_dir(root.join("src")).expect("src dir");
        std::fs::write(
            root.join("src").join("cursor_agent.rs"),
            "pub struct CursorAgentRequest;",
        )
        .expect("write source");

        let canonical_root = std::fs::canonicalize(root).expect("canonical root");
        let records = read_records_blocking(&canonical_root, MAX_RECORDS);
        let hits = rank_and_render(&records, &tokenize("CursorAgentRequest"), DEFAULT_MAX_HITS);

        assert!(hits.iter().any(|hit| hit.path == "src/cursor_agent.rs"));
    }
}
