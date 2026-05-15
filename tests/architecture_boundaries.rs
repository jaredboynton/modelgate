use std::{fs, path::Path};

const LAYERS_DOC: &str = "docs/architecture/LAYERS.md";

#[derive(Debug, Clone)]
struct Rule {
    source_prefix: &'static str,
    forbidden_targets: &'static [&'static str],
    rule: &'static str,
    remediation: &'static str,
}

#[derive(Debug)]
struct Violation {
    file: String,
    forbidden_target: &'static str,
    rule: &'static str,
    remediation: &'static str,
}

#[test]
fn source_layers_do_not_import_forbidden_edges() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let src_dir = manifest_dir.join("src");
    let mut violations = Vec::new();

    for file in rust_files(&src_dir) {
        let relative_file = file
            .strip_prefix(manifest_dir)
            .expect("source file should be under the crate root")
            .to_string_lossy()
            .replace('\\', "/");
        let source = fs::read_to_string(&file)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", file.display()));
        let imports = normalized_import_edges(&source);

        violations.extend(
            forbidden_import_violations(&relative_file, &imports)
                .into_iter()
                .map(|(forbidden_target, rule, remediation)| Violation {
                    file: relative_file.clone(),
                    forbidden_target,
                    rule,
                    remediation,
                }),
        );

        if relative_file == "src/router.rs" && router_imports_provider_runtime_layers(&imports) {
            violations.push(Violation {
                file: relative_file,
                forbidden_target: "provider runtime layer",
                rule: "router.rs wires routes, middleware, and request observation; provider execution belongs in route/upstream/adapter/auth modules",
                remediation: "move provider execution, adapter translation, or credential lookup out of router.rs and keep router.rs as endpoint wiring",
            });
        }
    }

    assert!(violations.is_empty(), "{}", format_violations(&violations));
}

fn rust_files(dir: &Path) -> Vec<std::path::PathBuf> {
    let mut files = Vec::new();
    collect_rust_files(dir, &mut files);
    files.sort();
    files
}

fn collect_rust_files(dir: &Path, files: &mut Vec<std::path::PathBuf>) {
    for entry in fs::read_dir(dir).unwrap_or_else(|error| {
        panic!("failed to read source directory {}: {error}", dir.display())
    }) {
        let entry = entry.expect("failed to read source directory entry");
        let path = entry.path();

        if path.is_dir() {
            collect_rust_files(&path, files);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            files.push(path);
        }
    }
}

fn forbidden_import_violations(
    relative_file: &str,
    imports: &[String],
) -> Vec<(&'static str, &'static str, &'static str)> {
    rules()
        .into_iter()
        .filter(|rule| relative_file.starts_with(rule.source_prefix))
        .flat_map(|rule| {
            rule.forbidden_targets
                .iter()
                .copied()
                .filter(move |target| imports.iter().any(|import| import_targets(import, target)))
                .map(move |target| (target, rule.rule, rule.remediation))
        })
        .collect()
}

fn rules() -> Vec<Rule> {
    vec![
        Rule {
            source_prefix: "src/auth/",
            forbidden_targets: &["route", "upstream", "adapter", "router"],
            rule: "auth is credential loading only and must not depend on request routing, upstream transport, adapters, or router wiring",
            remediation: "move orchestration into route/upstream modules and keep auth functions provider-credential focused",
        },
        Rule {
            source_prefix: "src/adapter/",
            forbidden_targets: &["route", "upstream", "auth", "router", "state"],
            rule: "adapter modules translate payload shapes and must not depend on routing, upstream transport, auth, router wiring, or state",
            remediation: "pass plain request/response data into adapters and keep provider calls, credentials, and app state outside adapter modules",
        },
        Rule {
            source_prefix: "src/upstream/",
            forbidden_targets: &["route", "router"],
            rule: "upstream modules own provider transport and must not call back into route handlers or router wiring",
            remediation: "move request-shaping decisions to route modules or shared model helpers before calling upstream",
        },
        Rule {
            source_prefix: "src/route/",
            forbidden_targets: &["auth"],
            rule: "route modules must not reach into auth internals directly",
            remediation: "route through upstream/state/model/error boundaries instead of loading provider credentials in handlers",
        },
        Rule {
            source_prefix: "src/cursor_agent",
            forbidden_targets: &["route", "upstream", "adapter", "auth", "router", "state"],
            rule: "src/cursor_agent.rs is the neutral DTO boundary between Cursor adapters and Cursor upstream and must not import any other crate-internal layer",
            remediation: "keep src/cursor_agent.rs to plain data types; move provider logic into upstream/cursor and adapter logic into adapter/cursor_*",
        },
    ]
}

#[test]
fn state_holds_only_dependency_wiring() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let state_path = manifest_dir.join("src/state.rs");
    let source = fs::read_to_string(&state_path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", state_path.display()));

    // Strip comments and string literals before counting Cursor mentions.
    // Doc comments and string literals frequently mention Cursor in ways that
    // do not constitute provider-specific business logic. The grep guard's
    // job is to flag actual Rust-level Cursor coupling: type references,
    // function calls, etc.
    let stripped = strip_comments_and_strings(&source);

    let mut occurrences = Vec::new();
    let mut remainder = stripped.as_str();
    while let Some(index) = remainder.find("Cursor") {
        let line_number = stripped[..(stripped.len() - remainder.len() + index)]
            .matches('\n')
            .count()
            + 1;
        occurrences.push(line_number);
        remainder = &remainder[index + "Cursor".len()..];
    }

    // Per ralplan Section 4: AppState carries only `Arc<CursorSessionStore>`
    // and equivalent dependency wiring. Allowlist:
    //   - `use crate::upstream::cursor::session::CursorSessionStore;` (1 import)
    //   - `pub cursor_sessions: Arc<CursorSessionStore>,` field declaration (1 type ref)
    //   - `cursor_sessions: Arc::new(CursorSessionStore::new()),` initializer
    //     in `from_env_with_config` (1 ref)
    //   - same initializer in `for_tests` (1 ref)
    //   - optional accessor (e.g. `pub fn cursor_sessions(...) -> ...`) (allow 1)
    //
    // 5 references covers a typical wiring without leaving room for any
    // provider business logic to leak into state.rs. If the grep count
    // climbs beyond 6, fail and force the offender to relocate the logic
    // into `src/upstream/cursor/` per ralplan Section 4.
    let allowlist_max = 6;
    assert!(
        occurrences.len() <= allowlist_max,
        "src/state.rs holds too many Cursor references ({} > {} allowed); provider invariants belong in src/upstream/cursor/. Lines: {:?}",
        occurrences.len(),
        allowlist_max,
        occurrences,
    );
}

fn normalized_import_edges(source: &str) -> Vec<String> {
    let without_comments = strip_comments_and_strings(source);
    let mut imports = Vec::new();

    for statement in semicolon_statements(&without_comments) {
        let trimmed = statement.trim();

        if trimmed.starts_with("use crate::") || trimmed.starts_with("use super::") {
            imports.extend(expand_use_statement(trimmed));
        }

        imports.extend(crate_paths_in_statement(trimmed));
    }

    imports.sort();
    imports.dedup();
    imports
}

fn semicolon_statements(source: &str) -> Vec<String> {
    let mut statements = Vec::new();
    let mut current = String::new();

    for character in source.chars() {
        current.push(character);
        if character == ';' {
            statements.push(current.clone());
            current.clear();
        }
    }

    if !current.trim().is_empty() {
        statements.push(current);
    }

    statements
}

fn expand_use_statement(statement: &str) -> Vec<String> {
    let Some(path) = statement
        .strip_prefix("use ")
        .and_then(|value| value.strip_suffix(';'))
    else {
        return Vec::new();
    };

    expand_path(path)
        .into_iter()
        .map(|path| {
            strip_alias(&path)
                .strip_prefix("super::")
                .map(|rest| format!("super::{rest}"))
                .unwrap_or_else(|| strip_alias(&path))
        })
        .collect()
}

fn expand_path(path: &str) -> Vec<String> {
    if let Some(open_brace) = path.find('{') {
        let Some(close_brace) = matching_close_brace(path, open_brace) else {
            return vec![path.to_string()];
        };
        let prefix = &path[..open_brace];
        let suffix = &path[close_brace + 1..];
        let inner = &path[open_brace + 1..close_brace];

        split_top_level(inner, ',')
            .into_iter()
            .map(str::trim)
            .filter(|part| !part.is_empty() && *part != "self")
            .flat_map(|part| expand_path(&format!("{}{}{}", prefix.trim(), part, suffix.trim())))
            .collect()
    } else {
        vec![path.trim_end_matches(';').trim().to_string()]
    }
}

fn matching_close_brace(value: &str, open_brace: usize) -> Option<usize> {
    let mut depth = 0;

    for (index, character) in value
        .char_indices()
        .skip_while(|(index, _)| *index < open_brace)
    {
        match character {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(index);
                }
            }
            _ => {}
        }
    }

    None
}

fn split_top_level(value: &str, separator: char) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut depth = 0;
    let mut start = 0;

    for (index, character) in value.char_indices() {
        match character {
            '{' => depth += 1,
            '}' => depth -= 1,
            character if character == separator && depth == 0 => {
                parts.push(&value[start..index]);
                start = index + character.len_utf8();
            }
            _ => {}
        }
    }

    parts.push(&value[start..]);
    parts
}

fn crate_paths_in_statement(statement: &str) -> Vec<String> {
    let mut paths = Vec::new();
    let mut remainder = statement;

    while let Some(index) = remainder.find("crate::") {
        let path_start = index + "crate::".len();
        let path_end = path_start + path_tail_len(&remainder[path_start..]);
        let path = &remainder[index..path_end];
        paths.push(path.trim_end_matches("::").to_string());
        remainder = &remainder[path_start..];
    }

    paths
}

fn strip_alias(path: &str) -> String {
    path.split_once(" as ")
        .map_or(path, |(before_alias, _)| before_alias)
        .trim()
        .to_string()
}

fn path_tail_len(value: &str) -> usize {
    value
        .char_indices()
        .take_while(|(_, character)| {
            character.is_alphanumeric()
                || matches!(
                    character,
                    '_' | ':' | '{' | '}' | ',' | '(' | ')' | '<' | '>'
                )
        })
        .map(|(index, character)| index + character.len_utf8())
        .last()
        .unwrap_or(0)
}

fn import_targets(import: &str, target: &str) -> bool {
    import == format!("crate::{target}")
        || import.starts_with(&format!("crate::{target}::"))
        || import == format!("super::{target}")
        || import.starts_with(&format!("super::{target}::"))
}

fn router_imports_provider_runtime_layers(imports: &[String]) -> bool {
    imports.iter().any(|import| {
        import_targets(import, "upstream")
            || import_targets(import, "adapter")
            || import_targets(import, "auth")
    })
}

fn strip_comments_and_strings(source: &str) -> String {
    let mut output = String::with_capacity(source.len());
    let mut chars = source.chars().peekable();
    let mut in_line_comment = false;
    let mut block_comment_depth = 0usize;
    let mut in_string = false;
    let mut in_char = false;
    let mut raw_string_hashes: Option<usize> = None;

    while let Some(character) = chars.next() {
        if in_line_comment {
            if character == '\n' {
                in_line_comment = false;
                output.push('\n');
            } else {
                output.push(' ');
            }
            continue;
        }

        if block_comment_depth > 0 {
            if character == '/' && chars.peek() == Some(&'*') {
                chars.next();
                block_comment_depth += 1;
                output.push_str("  ");
            } else if character == '*' && chars.peek() == Some(&'/') {
                chars.next();
                block_comment_depth -= 1;
                output.push_str("  ");
            } else if character == '\n' {
                output.push('\n');
            } else {
                output.push(' ');
            }
            continue;
        }

        if let Some(hash_count) = raw_string_hashes {
            if character == '"' {
                let mut matched_hashes = 0;
                while matched_hashes < hash_count && chars.peek() == Some(&'#') {
                    chars.next();
                    matched_hashes += 1;
                }
                if matched_hashes == hash_count {
                    raw_string_hashes = None;
                }
                output.push(' ');
            } else if character == '\n' {
                output.push('\n');
            } else {
                output.push(' ');
            }
            continue;
        }

        if in_string {
            if character == '\\' {
                output.push(' ');
                if chars.next().is_some() {
                    output.push(' ');
                }
            } else if character == '"' {
                in_string = false;
                output.push(' ');
            } else if character == '\n' {
                output.push('\n');
            } else {
                output.push(' ');
            }
            continue;
        }

        if in_char {
            if character == '\\' {
                output.push(' ');
                if chars.next().is_some() {
                    output.push(' ');
                }
            } else if character == '\'' {
                in_char = false;
                output.push(' ');
            } else if character == '\n' {
                output.push('\n');
            } else {
                output.push(' ');
            }
            continue;
        }

        if character == '/' && chars.peek() == Some(&'/') {
            chars.next();
            in_line_comment = true;
            output.push_str("  ");
        } else if character == '/' && chars.peek() == Some(&'*') {
            chars.next();
            block_comment_depth = 1;
            output.push_str("  ");
        } else if character == 'r' {
            let mut probe = chars.clone();
            let mut hash_count = 0;
            while probe.peek() == Some(&'#') {
                probe.next();
                hash_count += 1;
            }
            if probe.peek() == Some(&'"') {
                for _ in 0..hash_count {
                    chars.next();
                }
                chars.next();
                raw_string_hashes = Some(hash_count);
                output.push_str(&" ".repeat(hash_count + 2));
            } else {
                output.push(character);
            }
        } else if character == '"' {
            in_string = true;
            output.push(' ');
        } else if character == '\'' {
            in_char = true;
            output.push(' ');
        } else {
            output.push(character);
        }
    }

    output
}

fn format_violations(violations: &[Violation]) -> String {
    let mut message = format!(
        "architecture boundary violations found; see {LAYERS_DOC} for the layer contract\n"
    );

    for violation in violations {
        message.push_str(&format!(
            "\nfile: {}\nforbidden target: {}\nrule: {}\nremediation: {}\npointer: {}\n",
            violation.file,
            violation.forbidden_target,
            violation.rule,
            violation.remediation,
            LAYERS_DOC
        ));
    }

    message
}
