use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

const LIVE_HARNESS_OPT_IN: &str = "UMP_V2_LIVE_HARNESS";
const LIVE_COMPOSER_CODEX_CLI_OPT_IN: &str = "UMP_V2_LIVE_COMPOSER_CODEX_CLI";
const LIVE_CI_OPT_IN: &str = "UMP_V2_ALLOW_LIVE_TESTS_IN_CI";

#[test]
fn live_composer_codex_cli_harness_writes_blocked_artifact_without_opt_in() {
    let run_root = tempfile::tempdir().expect("create live harness temp run root");
    let script_path = repo_root().join("scripts/live/run-composer-codex-cli-validation.sh");

    let output = Command::new("bash")
        .arg(script_path)
        .env("UMP_V2_LIVE_HARNESS_RUNS_ROOT", run_root.path())
        .env_remove(LIVE_HARNESS_OPT_IN)
        .env_remove(LIVE_COMPOSER_CODEX_CLI_OPT_IN)
        .output()
        .expect("run live Composer Codex CLI validation script without opt-ins");

    assert_eq!(
        output.status.code(),
        Some(2),
        "missing live gates should produce live-blocked exit 2"
    );

    let summary_path = first_summary_json(run_root.path())
        .expect("missing-gate run should write a summary.json artifact");
    let summary = fs::read_to_string(summary_path).expect("read live-blocked summary");

    assert!(summary.contains(r#""status": "live-blocked""#));
    assert!(summary.contains(LIVE_HARNESS_OPT_IN));
    assert!(summary.contains(LIVE_COMPOSER_CODEX_CLI_OPT_IN));
    assert!(summary.contains(r#""run_dir_hash""#));
    assert!(!summary.contains(env!("CARGO_MANIFEST_DIR")));
}

#[test]
#[ignore = "requires UMP_V2_LIVE_HARNESS=1, UMP_V2_LIVE_COMPOSER_CODEX_CLI=1, and local Composer/Codex CLI setup"]
fn live_composer_codex_cli_validation_when_opted_in() {
    let Some(_guard) = LiveGuard::from_env("composer_codex_cli_validation") else {
        return;
    };

    let script_path = repo_root().join("scripts/live/run-composer-codex-cli-validation.sh");
    assert!(
        script_path.is_file(),
        "live Composer Codex CLI validation script is missing: {}",
        script_path.display()
    );

    let status = Command::new("bash")
        .arg(script_path)
        .status()
        .expect("run live Composer Codex CLI validation script");

    assert!(
        status.success(),
        "live Composer Codex CLI validation script failed with status={status}"
    );
}

struct LiveGuard;

impl LiveGuard {
    fn from_env(test_name: &str) -> Option<Self> {
        if env_flag(LIVE_HARNESS_OPT_IN) != Some(true) {
            eprintln!("live-blocked {test_name}: {LIVE_HARNESS_OPT_IN}=1 is required");
            return None;
        }
        if env_flag(LIVE_COMPOSER_CODEX_CLI_OPT_IN) != Some(true) {
            eprintln!("live-blocked {test_name}: {LIVE_COMPOSER_CODEX_CLI_OPT_IN}=1 is required");
            return None;
        }
        if env::var_os("CI").is_some() && env_flag(LIVE_CI_OPT_IN) != Some(true) {
            eprintln!("live-blocked {test_name}: {LIVE_CI_OPT_IN}=1 is required in CI");
            return None;
        }

        Some(Self)
    }
}

fn env_flag(name: &str) -> Option<bool> {
    env::var(name).ok().map(|value| value == "1")
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

fn first_summary_json(run_root: &Path) -> Option<PathBuf> {
    fs::read_dir(run_root)
        .ok()?
        .filter_map(Result::ok)
        .map(|entry| entry.path().join("summary.json"))
        .find(|path| path.is_file())
}
