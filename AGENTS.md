# unified-model-proxy-v2 agent notes

## Project Overview

Unified Model Proxy v2 = Rust HTTP proxy for Amp and agent clients. Binary binds `127.0.0.1:18743`, exposes local OpenAI/Anthropic/Google-compatible routes, and fans out to Bedrock Mantle, Codex/ChatGPT OAuth, Google Gemini direct, and Cursor AgentService.

Keep changes small and provider-aware. `README.md` = current product summary and operator guide. Durable architecture notes live under `docs/`.

## Tech Stack

- Rust 2021, Tokio async
- Axum 0.7 for HTTP routing, `tower-http` tracing
- Reqwest 0.12 w/ rustls for outbound HTTP
- Serde, `serde_json` for request/response
- AWS SDK crates for Bedrock creds, signing
- Tests use `tokio::test`, `tower::ServiceExt`, `tempfile`, `wiremock`
- No new deps for small routing/JSON/adapter helpers until existing local helpers + standard crates clearly insufficient

## Architecture

```text
Cargo.toml             crate manifest; keep Cargo.lock tracked
README.md              short product summary and local listen address
LAYERS.md              short layer map; links to canonical docs
docs/architecture/     layer contracts and forbidden edges
docs/guides/           local validation and distribution readiness
launchd/               local service launch assets
src/main.rs            tracing setup, env state, bind/serve entrypoint
src/lib.rs             public module exports for tests and integration
src/router.rs          route table and middleware
src/state.rs           env-derived app state and test-state guardrails
src/route/             HTTP handlers and API surface adapters
src/upstream/          provider-specific forwarding code
src/auth/              provider credential and signing helpers
src/sse/               SSE filtering and splice utilities
src/model_alias.rs     supported model catalog and provider resolution
tests/                 route, auth, model, upstream, and SSE coverage
```

Route layer parse/validate request shape, enforce provider/model fit, call relevant upstream module. Shared service state in `AppState`, not globals.

## Where To Look First

| Need | Start here |
|---|---|
| Layer ownership or forbidden imports | `LAYERS.md`, `docs/architecture/LAYERS.md` |
| Routes, upstreams, adapters, auth | `src/route/`, `src/upstream/`, `src/adapter/`, `src/auth/` |
| Model catalog and aliases | `src/model_alias.rs`, `src/codex_catalog.rs`, `tests/unit_model_alias.rs` |
| SSE or WebSocket behavior | `src/sse/`, `src/route/websocket.rs`, `tests/integration_websocket_*` |
| Launchd/local service setup | `launchd/`, `README.md` |
| Validation and distribution gates | `docs/guides/local-validation.md`, `docs/guides/distribution-readiness.md`, `scripts/gc/run-all.sh`, `.githooks/pre-commit` |
| Multi-agent coordination | `.harness/coordination/board.md`, `.harness/tasks/README.md`, `.harness/routing/tasks.yaml` |

## Coding Conventions

- Handlers thin: parse request bytes, validate model/provider, call `src/upstream/*`
- Use `AppError`, `AppResult` for route-facing failures
- Validate model/provider before credential lookup when route can decide locally; tests assert this
- Model IDs, upstream mappings centralized in `src/model_alias.rs`
- Preserve `serde_json::Value` passthrough for upstream request bodies unless adapter needs typed struct
- Use temp dirs for test homes via `AppState::for_tests` or test helpers. Tests must not touch real `$HOME` auth or Codex state
- Module-local helpers first. Promote to shared module only after 2nd real caller
- Codex CLI `supports_websockets` is provider-wide; docs/config examples must split mixed HTTP profiles from Codex-only WebSocket profiles
- Adapter work must keep the public Responses field policy matrix explicit; no silent field stripping

## Testing and Quality

- Run `cargo fmt --check` before finishing source changes
- Run `cargo test` for route/auth/model/upstream/SSE behavior changes
- Run `cargo clippy --all-targets --all-features -- -D warnings` for behavior/dependency/shared-module changes
- Add/update tests near changed behavior in `tests/` or relevant module tests
- Mock external provider w/ local tests or `wiremock`. No CI/unit tests on live creds

## File and Component Placement Rules

- New HTTP endpoints → `src/route/`, wired in `src/router.rs`
- New provider forwarding → `src/upstream/<provider>.rs`
- New provider credential logic → `src/auth/<provider>.rs`
- New shared request/response translation starts near owning route
- New durable design notes → `docs/architecture/` or `docs/guides/`
- Local runtime state/logs/agent scratch → ignored paths, not tracked source

## Safe-Change Rules

- Don't change `127.0.0.1:18743` casually; documented local Amp proxy address
- Don't route unknown/unsupported models as fallback. Model catalog = explicit allowlist
- Don't weaken test-home guards; prevent accidental reads/writes to real auth state
- Don't reorder model-not-supported/missing-credential failures w/o updating route tests + relevant planning note
- Don't rely on close-frame-only WebSocket failures for Codex CLI. Use wrapped top-level error events with `type = "error"`, numeric `status`, and `error.code`
- Don't claim remote clone readiness while `specter` uses `/Users/jaredboynton/__devlocal/specter`; it is a pre-remote blocker, and CI stays manual/pending or dependency-gated until portable
- Don't commit `.env`, `.omx/`, `target/`, logs, live runtime captures

## Commands

- Format: `cargo fmt --check`
- Build/check: `cargo check`
- Test: `cargo test`
- Lint: `cargo clippy --all-targets --all-features -- -D warnings`
- Run locally: `cargo run`
- Harness GC: `scripts/gc/run-all.sh`
- Install hook: `git config core.hooksPath .githooks`
