# unified-model-proxy-v2 agent notes

## Project Overview

Unified Model Proxy v2 is a Rust HTTP proxy for Amp. The binary binds to
`127.0.0.1:18743` and exposes local OpenAI/Anthropic-compatible routes that
fan out to Bedrock Mantle, Codex/ChatGPT OAuth, and Google Gemini direct.

Keep changes small and provider-aware. `README.md` gives the short product
summary; `PLANNING/v2-plan.md` is the implementation contract. Other files in
`PLANNING/` are durable design notes and active feature plans.

## Tech Stack

- Rust 2021 with Tokio async runtime.
- Axum 0.7 for HTTP routing and `tower-http` tracing.
- Reqwest 0.12 with rustls for outbound HTTP.
- Serde and `serde_json` for request/response handling.
- AWS SDK crates for Bedrock credentials and signing.
- Tests use `tokio::test`, `tower::ServiceExt`, `tempfile`, and `wiremock`.
- Do not add dependencies for small routing, JSON, or adapter helpers until
  existing local helpers and standard crates are clearly insufficient.

## Architecture

```text
Cargo.toml             crate manifest; keep Cargo.lock tracked
README.md              short product summary and local listen address
PLANNING/              durable plans and adapter design notes
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

The route layer should parse and validate request shape, enforce provider/model
fit, and then call the relevant upstream module. Shared service state belongs
in `AppState`, not in globals.

## Coding Conventions

- Keep handlers thin: parse request bytes, validate model/provider, then call
  `src/upstream/*`.
- Use `AppError` and `AppResult` for route-facing failures.
- Validate model/provider before credential lookup when a route can decide that
  locally; tests assert this behavior for several routes.
- Keep model IDs and upstream mappings centralized in `src/model_alias.rs`.
- Preserve `serde_json::Value` passthrough for upstream request bodies unless
  adapter behavior requires a typed struct.
- Use temp dirs for test homes through `AppState::for_tests` or test helpers.
  Tests must not touch real `$HOME` auth or Codex state.
- Prefer module-local helpers first. Promote a helper to a shared module only
  after a second real caller exists.

## Testing and Quality

- Run `cargo fmt --check` before finishing source changes.
- Run `cargo test` for route, auth, model, upstream, or SSE behavior changes.
- Run `cargo clippy --all-targets --all-features -- -D warnings` for behavior,
  dependency, or shared-module changes.
- Add or update tests near the changed behavior in `tests/` or the relevant
  module tests.
- Mock external provider behavior with local tests or `wiremock`. Do not make
  CI or unit tests depend on live credentials.

## File and Component Placement Rules

- New HTTP endpoints go in `src/route/` and are wired in `src/router.rs`.
- New provider forwarding code goes in `src/upstream/<provider>.rs`.
- New provider credential logic goes in `src/auth/<provider>.rs`.
- New shared request/response translation should start near the owning route.
- New durable design notes go in `PLANNING/`.
- Local runtime state, logs, and agent scratch files belong in ignored paths,
  not in tracked source.

## Safe-Change Rules

- Do not change `127.0.0.1:18743` casually; it is the documented local Amp
  proxy address.
- Do not route unknown or unsupported models as fallback traffic. The model
  catalog is an explicit allowlist.
- Do not weaken test-home guards; they prevent accidental reads or writes to
  real auth state.
- Do not reorder model-not-supported and missing-credential failures without
  updating route tests and the relevant planning note.
- Do not commit `.env`, `.omx/`, `target/`, logs, or live runtime captures.

## Commands

- Format: `cargo fmt --check`
- Build/check: `cargo check`
- Test: `cargo test`
- Lint: `cargo clippy --all-targets --all-features -- -D warnings`
- Run locally: `cargo run`
