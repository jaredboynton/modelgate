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
    ]
}

fn normalized_import_edges(source: &str) -> Vec<String> {
    let without_comments = strip_line_comments(source);
    let mut imports = Vec::new();

    for statement in semicolon_statements(&without_comments) {
        let compact = compact_whitespace(&statement);

        if compact.starts_with("usecrate::") || compact.starts_with("usesuper::") {
            imports.extend(expand_use_statement(&compact));
        }

        imports.extend(crate_paths_in_statement(&compact));
    }

    imports.sort();
    imports.dedup();
    imports
}

fn strip_line_comments(source: &str) -> String {
    source
        .lines()
        .map(|line| line.split_once("//").map_or(line, |(before, _)| before))
        .collect::<Vec<_>>()
        .join("\n")
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

fn compact_whitespace(value: &str) -> String {
    value
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect()
}

fn expand_use_statement(statement: &str) -> Vec<String> {
    let Some(path) = statement
        .strip_prefix("use")
        .and_then(|value| value.strip_suffix(';'))
    else {
        return Vec::new();
    };

    expand_path(path)
        .into_iter()
        .map(|path| {
            path.strip_prefix("super::")
                .map(|rest| format!("super::{rest}"))
                .unwrap_or(path)
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
            .filter(|part| !part.is_empty() && *part != "self")
            .flat_map(|part| expand_path(&format!("{prefix}{part}{suffix}")))
            .collect()
    } else {
        vec![path.trim_end_matches(';').to_string()]
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
        || import.starts_with(&format!("super::{target}::"))
}

fn router_imports_provider_runtime_layers(imports: &[String]) -> bool {
    imports.iter().any(|import| {
        import_targets(import, "upstream")
            || import_targets(import, "adapter")
            || import_targets(import, "auth")
    })
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
