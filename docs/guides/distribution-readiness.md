# Distribution readiness

Use this guide before creating a remote, sharing the repo, or shipping binaries.
It describes the current blocker state; it does not change runtime behavior.

## Current status

- Local development can validate the current checkout with Cargo commands.
- Remote clone portability is fully active: the dependency is resolved via GitHub.
- Hosted CI can run normally.
- Runtime secrets, `.env`, `.omx/`, `target/`, logs, and live captures must stay untracked.
- Mixed UMP Codex profiles keep `enable_request_compression = true` and
  `remote_compaction_v2 = false` until provider-aware compaction is implemented.

## Pre-remote blocker

There are currently no active clone-affecting local paths. The pre-remote blocker for Warpsock has been resolved by depending on the portable GitHub repository.

## Manual checklist

Before remote creation:

- `git status --short` shows only intended tracked files.
- No tracked `.env`, `.omx/`, `target/`, logs, live captures, or local auth files.
- `Cargo.toml` has no clone-breaking local dependency paths.
- `cargo metadata --locked --format-version=1` works outside the original machine path assumptions.
- `cargo build --locked` works from a clean clone.
- License, visibility, package naming, and release target are decided.
- README can be updated after active provider/WSS work lands.
- Codex config examples describe `name = "OpenAI"` as a compatibility shim, not
  provider-native compaction support for Bedrock or Google routes.
- Smoke and live-harness artifacts pass redaction scans before any summary is
  copied into tracked docs or issues.

Before distribution:

- `cargo fmt --check`
- `cargo check`
- `cargo nextest run` (formerly `cargo test`; runner shim retired May 2026)
- `cargo clippy --all-targets --all-features -- -D warnings`
- launch assets reviewed for local-only paths and developer-specific state.

## CI status

Hosted CI is no longer blocked by a local Warpsock path. CI can run formatting, check, test, clippy, and build, with no live credentials.

## Local path inventory

Absolute local paths in docs and planning notes are inventory items unless they
affect build or clone behavior. Absolute local paths in manifests, workflows,
launch assets, or scripts need stronger review. There are no remaining hard pre-remote blockers.
