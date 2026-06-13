use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src");
    println!("cargo:rerun-if-changed=tests");
    println!("cargo:rerun-if-changed=Cargo.toml");
    println!("cargo:rerun-if-changed=Cargo.lock");

    enforce_warpsock_only_transport();

    if let Some(git_dir) = git_dir() {
        println!("cargo:rerun-if-changed={}", git_dir.join("HEAD").display());
        println!("cargo:rerun-if-changed={}", git_dir.join("index").display());

        if let Some(head_ref) = git_head_ref(&git_dir) {
            println!("cargo:rerun-if-changed={}", head_ref.display());
        }
    }

    println!("cargo:rustc-env=UMP_BUILD_GIT_REVISION={}", git_revision());
    println!("cargo:rustc-env=UMP_BUILD_TIME_UTC={}", build_time_utc());
}

fn repo_root() -> PathBuf {
    PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is set"))
}

fn git_dir() -> Option<PathBuf> {
    let output = Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .current_dir(repo_root())
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let git_dir = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if git_dir.is_empty() {
        return None;
    }

    let git_dir = PathBuf::from(git_dir);
    Some(if git_dir.is_absolute() {
        git_dir
    } else {
        repo_root().join(git_dir)
    })
}

fn git_head_ref(git_dir: &Path) -> Option<PathBuf> {
    let head = fs::read_to_string(git_dir.join("HEAD")).ok()?;
    let head_ref = head.strip_prefix("ref: ")?.trim();
    Some(git_dir.join(head_ref))
}

fn git_revision() -> String {
    run_command("git", &["describe", "--tags", "--always", "--dirty"])
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "unknown".to_string())
}

fn build_time_utc() -> String {
    run_command("date", &["-u", "+%Y-%m-%dT%H:%M:%SZ"])
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "unknown".to_string())
}

fn run_command(program: &str, args: &[&str]) -> Option<String> {
    let output = Command::new(program)
        .args(args)
        .current_dir(repo_root())
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn enforce_warpsock_only_transport() {
    let repo_root = repo_root();
    let mut violations = Vec::new();

    scan_for_reqwest(&repo_root.join("src"), &mut violations);
    scan_for_reqwest(&repo_root.join("tests"), &mut violations);
    scan_file_for_reqwest(&repo_root.join("Cargo.toml"), &mut violations);

    if !violations.is_empty() {
        let joined = violations.join("\n - ");
        panic!(
            "reqwest is forbidden in this repo. Use warpsock instead.\nViolations:\n - {joined}"
        );
    }
}

fn scan_for_reqwest(path: &Path, violations: &mut Vec<String>) {
    let Ok(entries) = fs::read_dir(path) else {
        return;
    };

    for entry in entries.flatten() {
        let entry_path = entry.path();
        if entry_path.is_dir() {
            scan_for_reqwest(&entry_path, violations);
            continue;
        }
        scan_file_for_reqwest(&entry_path, violations);
    }
}

fn scan_file_for_reqwest(path: &Path, violations: &mut Vec<String>) {
    let is_scannable = matches!(
        path.extension().and_then(|extension| extension.to_str()),
        Some("rs" | "toml")
    );
    if !is_scannable {
        return;
    }

    let Ok(contents) = fs::read_to_string(path) else {
        return;
    };

    if contents.contains("reqwest") {
        violations.push(path.display().to_string());
    }
}
