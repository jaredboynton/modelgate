# Testing

Golden rule: tests are credential-free and must be safe on a developer machine, CI worker, or future remote clone.

## Required Shape

- Use `AppState::for_tests`, test helpers, temp dirs, and local fixtures instead of real `$HOME`.
- Mock providers with local servers or fixtures. Do not call Bedrock, Codex/ChatGPT, Google, or OAuth endpoints from unit/integration tests.
- Keep model/provider behavior locked with route tests whenever error ordering changes.
- Keep WebSocket behavior locked with `specter`-backed local integration tests, not live Codex sockets.
- Add tests near the behavior owner: route tests for HTTP decisions, auth tests for parsing/storage, upstream tests for provider shaping, adapter tests for format translation.

## Commands

- Fast formatting gate: `cargo fmt --check`.
- Standard local gate: `cargo check` then `cargo nextest run` (then `cargo clippy --tests --no-deps`).
- Behavior/dependency/shared-module gate: `cargo clippy --all-targets --all-features -- -D warnings`.
- Pre-remote gate: `scripts/gc/run-all.sh` once the harness scripts exist.

## Forbidden Patterns

- No tests that read `~/.codex/auth.json`, `~/.ump/auth.json`, AWS profiles, `GOOGLE_API_KEY`, or local launchd state.
- No tests that require a live network provider.
- No broad snapshots that hide typed error-code regressions.
- No unknown-model fallback assertions; the catalog is an explicit allowlist.

## Review Checklist

- Can this test pass with empty env and an empty temp home?
- Does it prove the exact failure code or provider route that matters?
- Would it still pass on a clean machine before remote creation?
- Does it avoid depending on the unresolved local `specter` path except through Cargo's declared dependency?
