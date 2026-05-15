# Coordination Board

Use this board for additive harness/runtime-safe work only. Do not use it to claim ownership of runtime proxy files unless a lead explicitly widens scope.

## Active Lanes

| Lane | Owner | Scope | Status | Blocker | Last Evidence |
|---|---|---|---|---|---|
| 1 | ulw-helper | `.omx/research/cursor-phase0/**` Phase 0 protocol/protobuf/auth/indexing extraction from reference repos. Read-only against `/Users/jaredboynton/__devlocal/unified-model-proxy-rs`, `/Users/jaredboynton/__devlocal/unified-model-proxy`, `/Users/jaredboynton/__devlocal/cursor-oauth-opencode`. No runtime/`src/`/`Cargo.toml` writes. | done | n/a | 17 notes, 5124 lines under `.omx/research/cursor-phase0/`; bundle complete |
| 2 | ultraqa-impl-A | EXCLUSIVE: `src/model_alias.rs`, `src/route/dispatch.rs`, `src/config_graph.rs`. Phase 1 catalog/dispatch/config graph. Reads `.omx/research/cursor-phase0/touchpoints-extraction.md`. | in progress | none | started 2026-05-15T06:30Z |
| 3 | ultraqa-impl-B | EXCLUSIVE: `src/route/responses_executor.rs`, `src/route/chat.rs`, `src/route/messages.rs`, `src/route/websocket.rs`. Phase 1 route enum + `not_implemented` arms (collateral exhaustive-match coverage). | in progress | depends on lane 2 enums | started 2026-05-15T06:30Z |
| 4 | ultraqa-impl-C | EXCLUSIVE: `src/auth/cursor.rs` (new), `src/auth/mod.rs` (extend). Phase 2 auth module. | in progress | none | started 2026-05-15T06:30Z |
| 5 | ultraqa-impl-D | EXCLUSIVE: `src/upstream/cursor/` (new dir + mod.rs, transport.rs, proto.rs, connect.rs, models.rs), `src/upstream/mod.rs` (extend), `Cargo.toml` (add h2/rustls/rustls-native-certs/tokio-rustls). Phase 3 transport + protobuf wire layer. | in progress | none | started 2026-05-15T06:30Z |
| 6 | ultraqa-impl-E | EXCLUSIVE: `src/cursor_agent.rs` (new). Neutral DTO boundary. | in progress | none | started 2026-05-15T06:30Z |
| 7 | ultraqa-impl-F | EXCLUSIVE: `src/upstream/cursor/run.rs`, `src/upstream/cursor/session.rs`, `src/state.rs` (Arc<CursorSessionStore> wiring only). Phase 3-4 run engine + session store. | in progress | depends on lanes 5+6 | started 2026-05-15T06:30Z |
| 8 | ultraqa-impl-G | EXCLUSIVE: `src/adapter/cursor_responses.rs`, `src/adapter/cursor_chat.rs`, `src/adapter/cursor_messages.rs`, `src/adapter/cursor_events.rs`, `src/adapter/mod.rs` (extend). Phase 4 public adapters. | in progress | depends on lane 6 DTOs | started 2026-05-15T06:30Z |
| 9 | ultraqa-impl-H | EXCLUSIVE: `src/upstream/cursor/indexing/`, `src/upstream/cursor/workspace.rs`. Phase 5 indexing + workspace. | in progress | depends on lanes 5+7 | started 2026-05-15T06:30Z |
| 10 | ultraqa-impl-I | EXCLUSIVE: `src/route/models.rs` (Composer rows on /v1/models, keep /api/provider/openai/v1/models Cursor-free), `src/compaction/policy.rs`, `src/compaction/render.rs`. Phase 1 catalog + compaction collateral. | in progress | depends on lane 2 enums | started 2026-05-15T06:30Z |
| 11 | ultraqa-impl-J | EXCLUSIVE: `tests/architecture_boundaries.rs`, `tests/unit_model_alias.rs`, `tests/integration_routes.rs`, `tests/integration_responses_executor.rs` plus new `tests/unit_cursor_proto.rs`, `tests/integration_cursor_responses.rs`, `tests/integration_cursor_chat.rs`, `tests/integration_cursor_messages.rs`, `tests/integration_cursor_indexing.rs`, `tests/integration_cursor_continuation.rs`, `tests/live_cursor_composer.rs` (gated by UMP_LIVE_CURSOR=1). | in progress | depends on lanes 2-9 | started 2026-05-15T06:30Z |

## Status Values

- `todo`: not started.
- `in progress`: files are being edited.
- `review`: ready for another agent to inspect.
- `blocked`: cannot proceed without lead decision.
- `done`: validated and no follow-up remains in this scope.

## Coordination Rules

- Write scope before editing.
- Keep claims evidence-backed: command output, changed files, or blocker.
- Do not revert another lane. Escalate conflicts in `Blocker`.
