# Distribution Hygiene

Golden rule: this repo must not have clone-affecting local paths. The previous Warpsock path dependency blocker has been resolved.

## Current Gate

- All dependencies are now portable, with `warpsock` resolved via the GitHub dependency pinned in `Cargo.lock`.
- The repo is fully ready for clean remote CI, clone portability, packaging readiness, and release.

## Hard Fails

- No tracked `.env`, `.env.*`, `.omx/`, `target/`, logs, coverage output, provider captures, or credential dumps.
- No live bearer tokens, OAuth refresh tokens, API keys, cookies, or private auth files in tests, docs, fixtures, traces, or examples.
- No CI or pre-commit hook may require live provider credentials.

## Report-Only Until Policy Lands

- Absolute local paths in docs, planning notes, launchd files, and manifests.
- Task-marker inventory.
- Oversized files that are already known artifacts.

Manifest paths that affect `cargo metadata`, `cargo build --locked`, or remote clone behavior must be labeled pre-remote blockers.

## Review Checklist

- Did this change add a new local absolute path?
- Did it add an example that would read real credentials during tests?
- Did it keep `warpsock` as the single WebSocket dependency?
