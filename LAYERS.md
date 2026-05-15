# UMP v2 layer map

This file is the short entrypoint. The canonical contract lives in
`docs/architecture/LAYERS.md`.

## Read order

1. `AGENTS.md` for repo commands, safe-change rules, and where-to-look pointers.
2. `docs/architecture/LAYERS.md` for module ownership and forbidden edges.
3. `docs/guides/local-validation.md` for local validation order.
4. `docs/guides/distribution-readiness.md` before creating a remote or shipping.

## Runtime layers

```text
src/main.rs       process setup and bind/serve
src/router.rs     route table, middleware, request observation
src/route/        HTTP and WebSocket API handlers
src/compaction/   provider-aware compaction policy, carrier inspection, pack seams
src/upstream/     provider transports and provider request execution
src/adapter/      cross-format request/response translation
src/auth/         provider credential loading, refresh, and signing
src/sse/          SSE filtering and splice utilities
src/state.rs      environment-derived AppState and test-state guardrails
```

Handlers stay thin, provider behavior stays in `src/upstream/`, credential logic
stays in `src/auth/`, and translation code stays near `src/adapter/` or the
owning route until there is a second real caller.

Compaction is a semantic boundary, not transport compression. `src/compaction/`
owns provider-aware carrier detection, UMP marker recognition, pack limits, and
visible-context rendering seams. Routes may call it before adapter conversion;
adapters keep their unknown-item rejection as defense in depth.

## Current distribution gate

`Cargo.toml` depends on `specter` through the local path
`/Users/jaredboynton/__devlocal/specter`. That is a pre-remote blocker. Do not
claim the repo is clone-portable or that hosted CI is required-green until
`specter` is made portable through a published crate, git dependency, vendor
policy, or another explicit dependency decision.
