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
cargo nextest run                                                 # primary; reuses rlibs for clippy
cargo clippy --tests --no-deps --all-features -- -D warnings      # run AFTER nextest
```

`cargo test` no longer routes through the retired `scripts/test-runner.sh`
shim, and the `[target.'cfg(all())'].runner` line in `.cargo/config.toml`
has been removed. Invoke `cargo nextest run` (or
`scripts/dev-test.sh`) directly. Plain `cargo test` will silently revert
to libtest harness output without nextest's parallel scheduler.

For targeted behavior changes, run the closest test first, then the wider gates.
Nextest filter expressions are more precise than positional name matches:

- route/model behavior: `cargo nextest run -E 'test(integration_routes) + test(unit_model_alias)'`
- auth behavior: `cargo nextest run -E 'test(unit_auth)'`
- upstream behavior: `cargo nextest run -E 'test(unit_upstreams) + test(integration_codex_transport) + test(integration_google_transport) + test(integration_bedrock_transport)'`
- SSE behavior: `cargo nextest run -E 'test(unit_sse) + test(integration_responses_sse)'`
- WebSocket behavior: `cargo nextest run -E 'test(integration_websocket_facade) + test(integration_websocket_passthrough)'`

For launchd-managed local runtime changes, verify the installed binary and live
listener after the Rust checks:

```sh
launchctl print gui/$(id -u)/dev.unified-model-proxy-v2
curl -fsS http://127.0.0.1:18743/health
lsof -nP -iTCP:18743 -sTCP:LISTEN
```

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
