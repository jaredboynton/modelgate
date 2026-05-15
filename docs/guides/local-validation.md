# Local validation

Run the smallest validation that proves the change, then widen only when the
change affects shared behavior. Normal validation must not use live provider
credentials.

## Command order

For docs-only changes:

```sh
git diff --check -- AGENTS.md LAYERS.md README.md docs/architecture/LAYERS.md docs/guides/distribution-readiness.md docs/guides/local-validation.md docs/guides/live-composer-codex-cli-validation.md scripts/smoke-local.sh scripts/live/run-composer-codex-cli-validation.sh
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

## Provider-aware compaction safety rail

Until UMP implements provider-aware compaction, mixed-provider Codex profiles use
transport compression without remote compaction. Local config validation should
confirm:

- `[features].enable_request_compression = true`;
- `[features].remote_compaction_v2 = false` for mixed UMP profiles such as
  `profiles.proxy`, `profiles.composer-2`, `profiles.composer-2-fast`, and
  `profiles.composer-1-5`;
- `name = "OpenAI"` under `model_providers.ump-v2` is only the compatibility
  shim that lets Codex use OpenAI-shaped Responses transport and compression;
- only Codex/OpenAI-only rollback profiles such as `profiles.proxy-ws` may
  enable profile-scoped `remote_compaction_v2 = true`;
- do not add `remote_compaction_provider_kind` to `~/.codex/config.toml` until
  the installed Codex build parses that field.

Use this non-secret check after config edits:

```sh
grep -nE 'enable_request_compression|remote_compaction_v2|\[profiles\.proxy-ws\.features\]|\[model_providers\.ump-v2\]|name = "OpenAI"' ~/.codex/config.toml
```

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
