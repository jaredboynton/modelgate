# Local validation

Run the smallest validation that proves the change, then widen only when the
change affects shared behavior. Normal validation must not use live provider
credentials.

## Command order

For docs-only changes:

```sh
git diff --check -- AGENTS.md LAYERS.md docs/architecture/LAYERS.md docs/guides/distribution-readiness.md docs/guides/local-validation.md
```

For Rust source changes:

```sh
cargo fmt --check
cargo check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
```

For targeted behavior changes, run the closest test first, then the wider gates.
Examples:

- route/model behavior: `cargo test integration_routes unit_model_alias`
- auth behavior: `cargo test unit_auth`
- upstream behavior: `cargo test unit_upstreams integration_codex_transport integration_google_transport integration_bedrock_transport`
- SSE behavior: `cargo test unit_sse integration_responses_sse`
- WebSocket behavior: `cargo test integration_websocket_facade integration_websocket_passthrough`

## Credential policy

- Unit and CI tests must not read real `$HOME`, Codex OAuth state, or live provider credentials.
- Use `AppState::for_tests`, temp dirs, local fixtures, or `wiremock`.
- Live-provider checks stay ignored or opt-in and must be named as live smoke tests.
- Do not commit `.env`, `.omx/`, `target/`, logs, live captures, or local auth files.

## CI caveat

Hosted CI remains manual/pending or dependency-gated until the `specter` local
path in `Cargo.toml` is replaced with a portable dependency strategy. Local
Cargo validation on this machine is useful, but it is not evidence that a clean
remote clone can build.

## Stop condition

Before claiming completion, report:

- changed files;
- validation commands run and their result;
- skipped validation and why;
- known blockers, especially the pre-remote `specter` portability gate when the
  work touches distribution or CI docs.
