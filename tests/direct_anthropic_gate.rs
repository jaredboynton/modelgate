use std::{
    fs,
    path::{Path, PathBuf},
};

#[test]
fn codebase_does_not_use_direct_anthropic_credentials_or_endpoint() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let forbidden = [
        format!("ANTHROPIC_{}", "API_KEY"),
        format!("ANTHROPIC_{}", "BASE_URL"),
        format!("api.{}", "anthropic.com"),
        format!("{}.json", "anthropic"),
        format!("auth::{}", "anthropic"),
        format!("{}Auth", "Anthropic"),
    ];
    let mut violations = Vec::new();
    for relative in ["src", "tests"] {
        scan_code_files(&root.join(relative), &forbidden, &mut violations);
    }
    assert!(
        violations.is_empty(),
        "direct Anthropic credential or endpoint path is forbidden; route Claude through Bedrock only: {violations:#?}"
    );
}

fn scan_code_files(path: &Path, forbidden: &[String], violations: &mut Vec<String>) {
    let Ok(entries) = fs::read_dir(path) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            scan_code_files(&path, forbidden, violations);
            continue;
        }
        if !is_code_file(&path) {
            continue;
        }
        let Ok(contents) = fs::read_to_string(&path) else {
            continue;
        };
        for pattern in forbidden {
            if contents.contains(pattern) {
                violations.push(format!("{} contains {pattern}", path.display()));
            }
        }
    }
}

fn is_code_file(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|extension| extension.to_str()),
        Some("rs" | "toml")
    )
}
