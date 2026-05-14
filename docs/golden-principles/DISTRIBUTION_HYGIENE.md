# Distribution Hygiene

Golden rule: this repo is not remote/distribution-ready until clone-affecting local paths are gone, especially the `specter` path dependency.

## Current Gate

- `Cargo.toml` currently depends on `specter = { package = "specters", path = "/Users/jaredboynton/__devlocal/specter" }`.
- That is acceptable for local harness work, but it is a pre-remote blocker.
- Do not claim clean remote CI, clone portability, packaging readiness, or release readiness while this dependency remains local.

## Hard Fails

- No tracked `.env`, `.env.*`, `.omx/`, `target/`, logs, coverage output, provider captures, or credential dumps.
- No live bearer tokens, OAuth refresh tokens, API keys, cookies, or private auth files in tests, docs, fixtures, traces, or examples.
- No CI or pre-commit hook may require live provider credentials.

## Report-Only Until Policy Lands

- Absolute local paths in docs, planning notes, launchd files, and manifests.
- Task-marker inventory.
- Oversized files that are already known artifacts.

Manifest paths that affect `cargo metadata`, `cargo build --locked`, or remote clone behavior must be labeled pre-remote blockers even if the script exits report-only.

## Review Checklist

- Did this change add a new local absolute path?
- Did it make CI sound mandatory or green on GitHub-hosted runners before `specter` is portable?
- Did it add an example that would read real credentials during tests?
- Did it keep `specter` as the single WebSocket dependency rather than hiding portability with another stack?
