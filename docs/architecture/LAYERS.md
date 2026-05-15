# Architecture layers

UMP v2 is a Rust Axum proxy for Amp. The boundary goal is simple: routes define
the API surface, upstream modules own provider I/O, adapters translate formats,
auth modules own credentials, and state wires environment-derived dependencies.

## Layer responsibilities

| Layer | Files | Owns | Must not own |
|---|---|---|---|
| Entrypoint | `src/main.rs` | tracing, config load, bind/serve lifecycle | provider translation or credential policy |
| Router | `src/router.rs` | route table, middleware, request observation | provider-specific request execution |
| Route | `src/route/**` | HTTP/WebSocket handlers, request parsing, model/provider validation, `AppError` responses | provider credentials internals or reusable transport clients |
| Compaction | `src/compaction/**` | provider-aware compaction policy, carrier inspection, UMP pack bounds/marker handling, visible-context render seams | route registration, provider transport, credential lookup, or adapter-specific request execution |
| Upstream | `src/upstream/**` | provider-specific forwarding, response normalization, transport calls through configured clients | route tables, middleware, or user-facing route policy |
| Adapter | `src/adapter/**` | OpenAI/Anthropic/Google/Responses shape conversion and SSE event mapping | provider auth, route registration, global state |
| Auth | `src/auth/**` | credential discovery, OAuth refresh, signing helpers | route dispatch, upstream retries, adapter behavior |
| Model catalog | `src/model_alias.rs`, `src/codex_catalog.rs` | supported model allowlist, provider resolution, Codex catalog projection | fallback routing for unknown models |
| State | `src/state.rs`, `src/hot_config.rs` | `AppState`, env parsing, local config, test-home guardrails | request translation or provider-specific business logic |
| SSE | `src/sse/**` | SSE filtering/splicing utilities | provider auth or route table decisions |
| Errors | `src/error.rs` | route-facing error type and HTTP conversion | provider-specific retry orchestration |
| Tests | `tests/**` | route/auth/model/upstream/SSE/WebSocket coverage with local mocks | live provider requirements in normal CI |

## Allowed dependency direction

```text
main -> router -> route -> upstream -> auth / adapter / sse / state / model_alias
router -> model_alias for request-observation logging only
route -> adapter / sse / state / model_alias / error
route -> compaction before provider adapter conversion
compaction -> model_alias / error
upstream -> auth / adapter / sse / state / model_alias / error
adapter -> model_alias / error / sse helpers when needed
auth -> state / error / provider SDK or HTTP client
```

Sibling modules may call each other inside the same layer when it keeps behavior
local and tested. Promote helpers only after a second real caller exists.

## Forbidden edges

- `src/auth/**` must not import route, upstream, adapter, router, or stateful route policy.
- `src/adapter/**` must not import route, upstream, auth, router, or mutable app state.
- `src/upstream/**` must not import route handlers or `src/router.rs`.
- `src/route/**` must not reach into auth internals directly; routes go through upstream/state/model/error boundaries.
- `src/compaction/**` must not register routes, call provider transports, load provider credentials, or bypass route-layer model/provider resolution.
- `src/router.rs` wires routes and middleware. It may consult the model catalog for request-observation logging, but must not grow provider request execution, adapter translation, or credential lookup logic.
- Tests must not read or write real `$HOME`, Codex OAuth state, or live provider credentials unless the test is explicitly ignored/live.

When a boundary is awkward, prefer moving the smallest shared helper to the
lowest neutral layer over adding a new abstraction.

## Provider-specific notes

- Bedrock auth/signing belongs in `src/auth/bedrock.rs`; transport behavior belongs in `src/upstream/bedrock.rs`.
- Codex OAuth and refresh belong in `src/auth/codex.rs`; Codex HTTP/WSS behavior belongs in `src/upstream/codex.rs`.
- Google credential discovery belongs in `src/auth/google.rs`; Google GenerateContent/Responses translation belongs in `src/adapter/google_*.rs`.
- Codex WebSocket failures must emit wrapped top-level error events with `type = "error"`, numeric `status`, and `error.code`; do not rely on close-frame-only failures.
- Compaction runs before provider adapters. Unknown provider-native `compaction` or `context_compaction` items for non-Codex targets fail as compaction errors instead of reaching adapter unknown-item errors. UMP-owned packs are recognized only by the `ump.compaction.v1.` marker and rendered as visible context by `src/compaction/`.

## Distribution boundary

The repository is not clone-portable while `Cargo.toml` contains:

```toml
specter = { package = "specters", path = "/Users/jaredboynton/__devlocal/specter" }
```

That local `specter` path is a pre-remote blocker, not a fix made in this docs
lane. Until it is portable, hosted CI is manual/pending or dependency-gated, and
final reports must not claim clean remote-clone readiness.
