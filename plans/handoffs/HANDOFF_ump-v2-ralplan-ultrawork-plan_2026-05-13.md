# Handoff: UMP v2 Ralplan To Ultrawork Plan

Date: 2026-05-13
Working directory: /Users/jaredboynton/__devlocal/amp-research/unified-model-proxy-v2
Branch: none - this directory is not a git repository
Parent handoff: none

## How to resume

Open a new session in the working directory above, paste this file's content
as the first message, and say "Continue from this handoff: run $ralplan to
generate an ultrawork-ready implementation plan from PLANNING/v2-plan.md".

---

## User requests (verbatim)

- "with all of this complete, take another look at /Users/jaredboynton/__devlocal/amp-research/unified-model-proxy-v2/PLANNING/v2-plan.md and revise it as applicable. the websocket stuff probably needs to be revised"
- "give me a handoff so i can start a new session of codex in that dir. i'll need it to run a ralplan loop to generate an ultrawork-ready plan there"

## Goal

Run `$ralplan` in this directory to turn `PLANNING/v2-plan.md` into an ultrawork-ready execution plan for building UMP v2.

## Work completed

- I reviewed `PLANNING/v2-plan.md` after the Specter WebSocket work landed.
- I revised `PLANNING/v2-plan.md` so the Codex WSS plan now defaults to Specter's RFC 6455 WebSocket client, not an ad hoc WSS implementation.
- I updated the v0.1 scope to say Codex Responses WSS must work through Specter's RFC 6455 WebSocket client in integration tests.
- I added a v0.1 non-goal: Codex WebSocket-over-HTTP/2 or WebSocket-over-HTTP/3 selection.
- I added a core decision that UMP should use `Client::websocket()` for Codex by default.
- I documented that Specter also exposes RFC 8441 (`websocket_h2`) and RFC 9220 (`websocket_h3`) tunnel APIs, but UMP must not assume the ChatGPT Codex endpoint supports them without live ALPN/settings evidence.
- I updated the dependency section from stale `specters` v2.1.3 language to the merged Specter mainline: local package version currently `2.3.0`, with merged Specter commit `da3ddff`.
- I changed the test dependency wording from generic "local WSS-compatible test server" to "local RFC 6455 WSS-compatible test server."
- I added guidance that H2/H3 Extended CONNECT fixtures should only be added if UMP enables those protocol lanes.
- I kept `reqwest` scoped to Bedrock Mantle and kept Specter as the client for Codex WSS, Codex HTTP fallback, OpenAI OAuth refresh, and Google HTTP/SSE.
- I added a hard dependency note: if crates.io lacks the merged Specter WebSocket APIs, use a local/path dependency to `/Users/jaredboynton/__devlocal/specter` or publish Specter first.
- I added "Do not add `tokio-tungstenite`" because Specter now owns the WebSocket frame state machine and the RFC 6455 / RFC 8441 / RFC 9220 handshake split.
- I updated the Codex headers to use `originator: codex_cli_rs`.
- I added a warning not to copy v1 Rust's older `originator: opencode` constant unless a live backend smoke test proves `codex_cli_rs` regressed.
- I added a "Default WebSocket protocol lane" section under Codex Responses.
- I stated that UMP should pass headers and messages into Specter, not build Upgrade headers or WebSocket frames itself.
- I documented the H2/H3 rules: Extended CONNECT only, required pseudo headers, `:status = 200` for success, and no HTTP/1.1 WebSocket bootstrap headers on H2/H3.
- I added an explicit v0.1 protocol selection rule: hardcode RFC 6455 for Codex WSS.
- I added a future-proofing note that an internal enum is okay, but `rfc8441` / `rfc9220` config should be rejected until there is endpoint evidence and a dedicated test fixture.
- I changed the fallback latch wording from "non-101 responses" to "non-101 handshake responses or WebSocket handshake errors" because Specter may surface handshakes as typed errors.
- I changed the image section to send future image generation over the existing Codex RFC 6455 WSS path.
- I changed image safety from "set WSS max message >=8 MB" to "keep Specter `max_message_size` at the default 16 MB or higher."
- I added `UT-CODEX-WS-PROTOCOL` to lock the default RFC 6455 protocol decision.
- I revised `IT-CODEX-WSS` to use a mock RFC 6455 WSS and verify the Specter path.
- I added `IT-CODEX-WSS-HANDSHAKE-FAIL` for repeated non-101 Specter handshake failures tripping the HTTP fallback latch.
- I revised the v0.2 gates to include a live Codex WSS capture decision before any RFC 8441/RFC 9220 lane work.
- I added `ws_protocol=rfc6455|rfc8441|rfc9220` to Codex observability fields.
- I added a risk row for Codex WebSocket protocol mismatch.
- I added a follow-up to capture ChatGPT Codex WSS ALPN/HTTP version before enabling Specter RFC 8441 or RFC 9220.
- I added evidence anchors for Specter commits and WebSocket tests.
- I added RFC source links for RFC 6455, RFC 8441, and RFC 9220.
- I did not create or modify implementation code in UMP v2.
- I did not inspect or include `.env`; it exists in the directory and may contain secrets.

## Current state

- `PLANNING/v2-plan.md` has been updated directly on disk.
- `plans/handoffs/HANDOFF_ump-v2-ralplan-ultrawork-plan_2026-05-13.md` is this handoff file.
- The directory has no `.git` repository under `unified-model-proxy-v2` or nearby `amp-research`; `git status` and branch checks return nothing.
- Current visible files before this handoff were:
  - `.env`
  - `PLANNING/bedrock.md`
  - `PLANNING/codex.md`
  - `PLANNING/v2-plan.md`
- The `.env` file was intentionally not read.
- Markdown fence count in `PLANNING/v2-plan.md` was checked after edits and is even.
- I searched `PLANNING/v2-plan.md` for stale `v2.1.3`, `tokio-tungstenite`, WebSocket protocol, originator, and Specter language after editing.
- The plan still references v1 Rust and TS source paths under sibling directories:
  - `../unified-model-proxy-rs`
  - `../unified-model-proxy`
- The next session should assume the plan is a planning artifact, not an implementation repo yet.

## Pending tasks

- Run `$ralplan` from this directory.
- Treat `PLANNING/v2-plan.md` as the source plan to refine.
- Produce an ultrawork-ready execution plan, not implementation code, unless the user explicitly switches to execution.
- Break the UMP v2 build into parallelizable work lanes with clear file ownership.
- Include acceptance criteria and validation commands per lane.
- Include a dependency/bootstrap lane because the directory currently does not contain `Cargo.toml`, `src/`, or `tests/`.
- Include a Codex transport lane that uses Specter RFC 6455 WSS by default.
- Include a Bedrock lane that handles bearer/profile/discovery and Mantle request forwarding.
- Include a Google lane that handles path rewrite, API key auth, SSE passthrough, and Bedrock fallback translation.
- Include auth and file-write safety tasks for `~/.codex/auth.json` and `~/.ump/auth.json`.
- Include SSE filter/splice porting from v1.
- Include route/model map tasks.
- Include observability and failure capture tasks.
- Include E2E Amp smoke tests.
- Do not add RFC 8441 or RFC 9220 Codex support to v0.1 execution unless the plan explicitly makes that a future capture-gated lane.
- Do not introduce `tokio-tungstenite`; Specter is the WebSocket owner.

## Key files (max 10)

- `PLANNING/v2-plan.md` — main UMP v2 implementation plan to refine with `$ralplan`.
- `PLANNING/codex.md` — Codex Responses versus public OpenAI Responses notes; includes endpoint, header, body, event, compaction, and quick checklist details.
- `PLANNING/bedrock.md` — Bedrock Mantle planning notes and likely source for Bedrock auth/model details.
- `.env` — local environment file; do not read or quote unless the user explicitly asks and it is safe to handle secrets.
- `plans/handoffs/HANDOFF_ump-v2-ralplan-ultrawork-plan_2026-05-13.md` — this handoff.
- `../unified-model-proxy-rs/crates/ump-auth/src/lib.rs` — v1 Rust auth constants and OAuth refresh implementation; beware stale `CODEX_ORIGINATOR`.
- `../unified-model-proxy-rs/crates/ump-adapters/ump-adapter-codex-responses-wss/src/lib.rs` — v1 Rust Codex WSS adapter evidence and reusable behavior.
- `../unified-model-proxy-rs/crates/ump-compat/src/lib.rs` — v1 SSE filter/splice logic to port.
- `../unified-model-proxy/src/lib/adapters/codex-responses.ts` — TS adapter evidence for `originator: codex_cli_rs`.
- `/Users/jaredboynton/__devlocal/specter` — Specter repo containing merged RFC 6455, RFC 8441, and RFC 9220 WebSocket support.

## Decisions and rationale

- Default Codex WSS stays RFC 6455 over HTTP/1.1 — this matches the current plan evidence and avoids assuming unsupported H2/H3 behavior.
- Specter owns WebSocket implementation — Specter now has a shared RFC 6455 codec plus RFC 8441 and RFC 9220 tunnel APIs, so UMP should not duplicate frame or handshake state machines.
- RFC 8441 and RFC 9220 are future lanes — they require live endpoint ALPN/settings evidence or a fixture-backed decision before UMP should enable them for Codex.
- `originator: codex_cli_rs` is the planned Codex header — TS adapter and `PLANNING/codex.md` support it; v1 Rust's `opencode` constant is treated as stale unless smoke testing proves otherwise.
- Use a local/path Specter dependency if publishing lags — the local Specter repo is already merged and validated, while crates.io availability may lag.
- Keep v0.1 small — Bedrock, Codex, and Google only; no broad provider framework or legacy v1 sprawl.
- Keep `reqwest` only for Bedrock Mantle — AWS signing fits `reqwest`, while Specter buys browser/TLS/WebSocket behavior for Codex and Google.
- Treat UMP v2 as a greenfield single binary — the current directory contains planning docs only, not a generated project.

## Failed approaches (do not retry without new information)

- Do not plan a `tokio-tungstenite` Codex WSS adapter — Specter now provides the WebSocket APIs and frame handling needed for v0.1.
- Do not assume `SETTINGS_ENABLE_CONNECT_PROTOCOL = 1` is default behavior for H2/H3 — it must be advertised by the server before Extended CONNECT is sent.
- Do not use HTTP/1.1 Upgrade, `Connection`, `Host`, `Sec-WebSocket-Key`, or `Sec-WebSocket-Accept` on H2/H3 WebSocket paths — RFC 8441 and RFC 9220 use Extended CONNECT instead.
- Do not claim Chrome-exact RFC 8441 or RFC 9220 fingerprinting for Specter or UMP without live browser capture or repo evidence.
- Do not copy the v1 Rust `originator: opencode` Codex header into the v2 plan without a live backend regression; the current plan points to `codex_cli_rs`.
- Do not read `.env` just to plan; it may contain secrets and is not needed for the ultrawork-ready plan.

## Explicit constraints

- "with all of this complete, take another look at /Users/jaredboynton/__devlocal/amp-research/unified-model-proxy-v2/PLANNING/v2-plan.md and revise it as applicable. the websocket stuff probably needs to be revised"
- "give me a handoff so i can start a new session of codex in that dir. i'll need it to run a ralplan loop to generate an ultrawork-ready plan there"
- "No broad provider framework. No legacy v1 sprawl. Lift proven v1 algorithms where they matter."
- "Do not ship v0.1:"
- "Codex WebSocket-over-HTTP/2 or WebSocket-over-HTTP/3 selection."
- "Public OpenAI `/v1/images/*` calls with Codex OAuth."
- "Codex WebSocket default: Specter RFC 6455 over HTTP/1.1 via `Client::websocket()`. Specter now also has RFC 8441 (`websocket_h2`) and RFC 9220 (`websocket_h3`) tunnel APIs, but UMP must not assume ChatGPT Codex supports them without live ALPN/settings evidence."
- "If crates.io lacks the merged Specter WebSocket APIs, use a local/path dependency to `/Users/jaredboynton/__devlocal/specter` or publish Specter first. Do not add `tokio-tungstenite`; Specter owns the WebSocket frame state machine and the RFC 6455 / RFC 8441 / RFC 9220 handshake split."
- "Do not copy v1 Rust's `originator: opencode` constant unless a live backend smoke test proves `codex_cli_rs` regressed. The TS adapter and Codex notes use `codex_cli_rs`."
- "v0.1 hardcodes RFC 6455 for Codex WSS."
- "Never log raw base64 image payload."
- "Harness refuses real home paths."

## Open questions

- Should the ultrawork-ready plan include initial project scaffolding as the first execution lane, since this directory currently has planning docs only?
- Should the new session split the plan into separate `.omx/plans/` artifacts, or revise `PLANNING/v2-plan.md` into an implementation checklist in place?
- Should UMP v2 use local/path Specter immediately, or should publishing Specter happen before UMP implementation starts?
- Should v0.1 include real Codex WSS smoke testing against ChatGPT, or keep Codex WSS validation entirely fixture-based until the first integration pass?
- Should the Codex HTTP fallback be built in v0.1 or stay behind the `wss-then-http` latch as a v0.2 gate? The current plan lists Codex HTTP fallback under Specter usage and default transport, but v0.2 gates still mention Codex HTTP fallback.
- Should `count_tokens` remain a local approx/stub in v0.1, or should ralplan move it out of the initial ultrawork lanes unless Amp proves it is required?

## Context for continuation

- The current directory is planning-only; there is no `Cargo.toml`, `src/`, or `tests/` yet.
- The next session should run `$ralplan` and produce an ultrawork-ready plan, not jump straight into implementation.
- The target implementation should be a Rust HTTP proxy listening on `127.0.0.1:18743`.
- v2 keeps only three upstreams: Bedrock Mantle, Codex/ChatGPT OAuth, and Google Gemini direct.
- The plan's verified Amp bundle is `0.0.1778531432-g3bd093`.
- Amp path facts are in `PLANNING/v2-plan.md` and should drive route design.
- The plan explicitly keeps `/chat/completions` as a contingency and suggests dropping it after a 24h soak if unused.
- The model map in `PLANNING/v2-plan.md` should become a concrete `model_alias` task with unit tests.
- Codex auth reads `~/.codex/auth.json` only; `~/.ump/auth.json` is a diagnostic mirror for the Codex section, not an access-token source.
- Codex refresh must avoid double-refreshing the same refresh token after a concurrent file change.
- Bedrock discovery probes real Mantle with `max_tokens=1`; no IAM simulation and no `ListFoundationModels`.
- Google auth is `GOOGLE_API_KEY` env only; no OAuth or file fallback.
- Google fallback translates Gemini request to Anthropic Messages, sends through Bedrock Mantle, then translates Anthropic text response back to Gemini response shape.
- `gpt-image-2` is unsupported in v0.1; future generation support goes through Codex Responses hosted image tool, not public OpenAI Images endpoints.
- Image edit support is explicitly gated on real Codex Desktop trace or fixture proving image-input shape.
- Failure capture must redact auth headers, API keys, cookies, refresh tokens, and image base64 payloads.
- Specter main in `/Users/jaredboynton/__devlocal/specter` is currently ahead of origin with the WebSocket commits merged, but the main worktree also contains unrelated binding work from another Codex session.
- Specter relevant commits:
  - `44c4769` adds RFC 8441 WebSocket-over-HTTP/2.
  - `f24676c` adds RFC 9220 WebSocket-over-HTTP/3.
  - `da3ddff` merges the HTTP/3 WebSocket work into current main.
- Specter validation after merge included `cargo fmt --check`, targeted RFC 9220/RFC 9114/RFC 8441/RFC 6455 tests, and `just test` with 286 passed and 1 skipped in the feature worktree.
- Use Specter tests as reference for how the WebSocket lanes are separated:
  - `tests/rfc6455_websocket.rs`
  - `tests/websocket_handshake.rs`
  - `tests/rfc8441_*`
  - `tests/rfc9220_*`
- The `$ralplan` output should be directly consumable by `$ultrawork`: lanes, ownership, dependencies, acceptance tests, validation commands, and stop conditions.
