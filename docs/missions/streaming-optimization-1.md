# Streaming Optimization 1

## Mission

Find and prioritize issues in how unified-model-proxy-v2 handles network requests, especially streaming behavior, buffering boundaries, retries, timeouts, request forwarding, and provider-specific transport semantics.

## Current Context

- The proxy listens on `127.0.0.1:18743` and exposes OpenAI/Anthropic/Google-compatible routes.
- The current working tree already includes Cursor hardening around protobuf MCP args, terminal stream handling, and batched tool-result continuation validation.
- Prior analysis showed `/v1/responses` is a universal compatibility layer: provider routing, continuation state, and SSE lifecycle synthesis are separate costs from simple wire-format conversion.
- Recent Specter releases changed buffering/streaming contracts; any issue that depends on `.text()`, `.bytes()`, `send()`, `send_streaming()`, or `into_buffered()` should be validated against the actual call site.

## Rules For Agents

- Append findings only under your assigned section.
- Do not edit another agent's section.
- Real Cursor/OpenAI/local proxy calls are allowed, but redact secrets and do not commit logs or raw captures.
- Favor concrete file paths, reproduction commands, failure evidence, and proposed tests over broad speculation.
- If patching, keep changes minimal and list touched files in the assigned section.

## Priority Findings

1. **P1 — Bedrock streaming fallback drops SigV4 auth headers.** Agents A and D independently found the same bug: the primary `send_streaming()` path signs headers, but the non-H2 fallback rebuilds headers from the unsigned request. Fix first because it turns transport fallback into false Bedrock auth failure.
2. **P1 — Cursor stream lifecycle can hang or leak tasks.** Agent C found three related Cursor transport issues: no timeout while waiting for streaming response headers, total-stream rather than idle read timeout, and heartbeat/reader cleanup depending on explicit `close()` instead of terminal/drop semantics.
3. **P1 — Streaming access logs record header completion, not body completion.** Agent F showed live Codex/Cursor streams logging `latency_ms=0` while curl observed ~1s–1.5s full-body durations. Late body failures can be hidden behind successful early 200 logs.
4. **P1/P2 — Stream translators do not consistently flush buffered state on upstream EOF.** Agent B found Google/Anthropic Responses stream adapters map chunks only and do not call translator `finish()` on clean EOF, risking dropped buffered frames or missing terminal events.
5. **P2 — Buffered fallbacks and full-body reads bypass streaming expectations and size limits.** Agent A found Bedrock/Windsurf/Codex fallback paths that convert requested streams into full-body waits, plus Specter `.bytes()`/`.text()`/`collect_to_bytes()` sites without the proxy's existing Axum-side caps.
6. **P2 — Bedrock passthrough terminal detection is fragile.** Agent B found passthrough SSE terminal detection depends on `text.contains("event: message_stop")`, missing valid SSE spellings like `event:message_stop`, CRLF, or JSON-only terminal data.
7. **P2 — WebSocket HTTP bridge needs better malformed-frame and backpressure behavior.** Agent E found malformed in-flight client frames abort active turns, and HTTP-backed bridge queue saturation lacks a timeout/error policy.
8. **P3 — Policy and regression-test gaps remain.** Lower-priority follow-ups include raw Google provider method policy, compression/body-length regression coverage, Codex `[DONE]` terminal policy, and model extraction/logging coverage.

## Implementation Progress

### 2026-05-25 Pass 1

- **Done:** Bedrock non-H2 streaming fallback now preserves signed provider auth headers; regression added in `tests/integration_bedrock_transport.rs`.
- **Done:** Cursor streaming open now times out while waiting for response headers; reader timeout is per-read idle timeout instead of total stream lifetime; `RunStream` drop aborts reader/heartbeat tasks.
- **Done:** Google and Anthropic Responses stream conversion now flushes translator state on upstream EOF, including final SSE frames without a trailing separator.
- **Validation:** `cargo fmt --check`, `cargo nextest run` (`799 passed`, `14 skipped`), and `cargo clippy --tests --no-deps --all-features -- -D warnings` passed.
- **Runtime:** `scripts/install-launchd-release.sh` rebuilt/kickstarted the proxy; `/health` returned `v0.1.11-1-g5946cea-dirty` with build time `2026-05-25T11:26:54Z`.
- **Live smoke:** bounded Cursor forced-tool stream completed with clean `{"q":"ok"}` arguments; artifact directory `.live-harness/runs/20260525T112754Z-cursor-network-fixes-probe`.
- **Remaining:** body-drain observability, bounded Specter body collection, Bedrock terminal parsing hardening, WebSocket bridge saturation/malformed-frame policy, and lower-priority route/header policy tests.

## Agent A: HTTP Client And Specter Contract

### Scope

Audit outbound HTTP client usage for buffering-vs-streaming contract leaks, response-body double reads, incorrect helper selection, and places where Specter behavior may be misused.

### Findings

#### A1 — P1 — Bedrock streaming fallback drops SigV4 auth headers

- **Files:** `src/upstream/bedrock.rs:270`, `src/upstream/bedrock.rs:277`, `src/upstream/bedrock.rs:286`, `src/upstream/bedrock.rs:289`
- **Finding:** `send_runtime_once` applies SigV4 headers to `headers` before the primary `.send_streaming()` path, but the non-H2 fallback rebuilds `headers` from `request.headers.clone()` instead of reusing the signed header map. If Specter returns a non-H2 streaming error and the fallback `.send()` path is used, the fallback request can go upstream without `authorization`, `x-amz-date`, and related signing headers.
- **Evidence:** `nl -ba src/upstream/bedrock.rs | sed -n '260,305p'` shows `apply_runtime_auth_headers(request, &mut headers).await?` at line 271, `.headers(headers)` on the streaming request at line 279, and fallback `let headers = request.headers.clone();` at line 289 before `.send()` at line 295.
- **Impact:** A real Bedrock stream can fail only on the Specter fallback path with provider auth errors, making the fallback unreliable exactly when HTTP/2 streaming setup fails or local/mock transport lacks H2.
- **Proposed fix/test:** Clone the already-signed `headers` before moving it into the streaming request, and use that signed clone for the fallback `.send()` path. Add/extend an integration test that forces `is_non_h2_streaming_error` fallback and asserts the fallback request still contains SigV4 auth/date headers.

#### A2 — P2 — Buffered fallbacks silently turn requested streams into full-body waits

- **Files:** `src/upstream/bedrock.rs:286`, `src/upstream/bedrock.rs:358`, `src/upstream/windsurf.rs:201`, `src/upstream/windsurf.rs:41`, `src/upstream/codex.rs:863`, `src/upstream/codex.rs:886`
- **Finding:** Three streaming providers intentionally fall back from `send_streaming()` to buffered `send()` on non-H2 Specter errors, then consume the entire body with `.bytes()`/`collect_specter_body()` before yielding one synthesized stream item. This is acceptable for local H1 mocks, but it is a contract leak if triggered against a real provider: downstream receives no incremental chunks even though the route stays on a streaming code path.
- **Evidence:** `rg -n "send_streaming|is_non_h2_streaming_error|collect_specter_body|\.bytes\(\)" src/upstream/{bedrock,windsurf,codex}.rs` shows Bedrock fallback at `src/upstream/bedrock.rs:286` and full collection at `src/upstream/bedrock.rs:363`; Windsurf fallback at `src/upstream/windsurf.rs:201` and `.bytes()` at `src/upstream/windsurf.rs:43`; Codex fallback at `src/upstream/codex.rs:863` and `.bytes()` at `src/upstream/codex.rs:888`.
- **Impact:** If Specter reports the fallbackable error class for a real streaming request, TTFT regresses to full response time and large streams are buffered in memory before the first downstream event.
- **Proposed fix/test:** Gate buffered fallback to known local/mock bases or expose a metric/header/log field that marks `stream_transport=buffered_fallback`; add provider-specific tests proving real streaming configs fail fast or preserve streaming instead of silently buffering.

#### A3 — P2 — Unbounded Specter full-body reads remain on several upstream paths

- **Files:** `src/upstream_response.rs:168`, `src/upstream/windsurf.rs:29`, `src/upstream/windsurf.rs:43`, `src/upstream/codex.rs:888`, `src/upstream/bedrock.rs:363`, `src/codex_catalog.rs:184`, `src/route/models.rs:178`, `src/upstream/openai_public.rs:166`
- **Finding:** The Axum-side helpers enforce `MAX_UPSTREAM_BODY_BYTES` / `MAX_UPSTREAM_ERROR_BODY_BYTES`, but Specter response reads use `response.bytes()`, `response.text()`, or `SpecterBody::collect_to_bytes()` directly with no local cap. Some of these are normal bounded metadata paths, but OpenAI passthrough, Codex buffered fallback, Bedrock event-stream fallback, and Windsurf buffered fallback can read provider-controlled bodies into memory.
- **Evidence:** `nl -ba src/upstream_response.rs | sed -n '155,175p'` shows `collect_specter_body` directly calling `body.collect_to_bytes()`; `rg -n "response\.(bytes|text)\(\)|collect_specter_body" src/upstream src/route src/codex_catalog.rs` identifies the full-body sites above.
- **Impact:** Large upstream error bodies or accidental buffered streaming responses can bypass the proxy's existing body-size guardrails and increase memory pressure.
- **Proposed fix/test:** Add `collect_specter_body_limited(context, limit)` or a limit parameter to `collect_specter_body`, use error-body limits for non-success responses and upstream-body limits for buffered success fallbacks, and add mock upstream tests with over-limit bodies to verify deterministic `AppError` instead of unbounded collection.

#### A4 — P3 — Specter helper selection is mostly correct; no response double-read found

- **Files:** `src/upstream_response.rs:110`, `src/upstream_response.rs:177`, `src/upstream/openai_public.rs:164`, `src/upstream/openai_public.rs:183`, `build.rs:99`
- **Finding:** The main passthrough helper uses `UpstreamResponse::from_specter` to preserve the Specter body as an Axum stream, and response-classifying code captures status/headers before a single consuming `.bytes()` read. I did not find a same-response double-read of Specter bodies. The repo also still enforces the no-`reqwest` policy.
- **Evidence:** `rg -n "\breqwest\b" src tests Cargo.toml build.rs` returns only the `build.rs` forbidden-policy message/scan; `src/upstream_response.rs:110` converts `response.into_body()` exactly once; `src/upstream/openai_public.rs:164-166` and `src/upstream/openai_public.rs:183-185` read separate first/second responses once each.
- **Impact:** This lowers migration risk: the critical issues are fallback/header semantics and unbounded full-body reads, not broad helper misuse.
- **Proposed fix/test:** Keep `from_specter` as the default for provider passthrough. Add a regression scan or unit test around no-`reqwest`/single-consumption assumptions only if future transport helpers are introduced.

#### Commands run

- `rg -n "streaming-optimization|Specter|HTTP Client|Agent A|outbound HTTP|reqwest|bytes_stream|text\(|json\(" /Users/jaredboynton/.codex/memories/MEMORY.md docs/missions/streaming-optimization-1.md src tests Cargo.toml`
- `rg -n "send_streaming|into_buffered|send\(|\.text\(\)|\.bytes\(\)|\.json\(|from_specter|observe_specter_response|SpecterBody|specter::Client|client\." src tests build.rs Cargo.toml`
- `nl -ba src/upstream/bedrock.rs | sed -n '260,305p'`
- `nl -ba src/upstream/windsurf.rs | sed -n '20,70p;145,205p'`
- `nl -ba src/upstream/codex.rs | sed -n '850,895p;898,908p'`
- `nl -ba src/upstream_response.rs | sed -n '110,190p'`
- `nl -ba src/codex_catalog.rs | sed -n '170,188p'`
- `nl -ba src/route/models.rs | sed -n '170,182p'`
- `nl -ba src/upstream/openai_public.rs | sed -n '160,188p'`
- `rg -n "\breqwest\b" src tests Cargo.toml build.rs`

#### Changed files

- `docs/missions/streaming-optimization-1.md`


## Agent B: SSE Framing And Terminal Events

### Scope

Audit SSE production/translation for terminal-event correctness, chunk coalescing, missing flush opportunities, duplicate terminals, and stalled streams across providers.

### Findings

- **B1 - High - Responses stream translators never flush their pending adapter state on upstream EOF.**
  - Evidence: `src/route/responses_executor.rs:1094`-`src/route/responses_executor.rs:1108` and `src/route/responses_executor.rs:1148`-`src/route/responses_executor.rs:1162` map only upstream body chunks through the Anthropic and Google Responses SSE translators; neither stream chains an EOF `finish()` call. `GoogleResponsesSseTranslator::finish()` explicitly drains a partial buffered frame and emits a terminal event when started-but-not-completed at `src/adapter/google_responses.rs:218`, but that method is only used by the text helper path, not by the streaming route. The direct Google route has the same shape at `src/route/google.rs:80`-`src/route/google.rs:95`.
  - Impact: if an upstream/proxy transport closes after a final frame without `\n\n`, or closes after content chunks without an explicit finish marker, buffered bytes can be dropped and downstream clients can see a stream EOF with no `response.completed`/`response.incomplete`. That is exactly the stalled-stream class this mission is looking for.
  - Proposed fix/test: replace the simple `.map(...).filter_map(...)` wrappers with `stream::unfold`/`try_unfold` state machines that call translator `finish()` on `None`, mirroring Codex EOF handling in `src/upstream/codex.rs:976`-`src/upstream/codex.rs:981`. Add route-level tests for Google Responses, direct Google generateContent, and Anthropic Messages streams where the terminal frame lacks a trailing separator and where content is followed by clean EOF.

- **B2 - Medium - Bedrock passthrough SSE terminal detection is string-fragile.**
  - Evidence: Bedrock Runtime marks stream completion in the decoder and raises `premature EOF before message_stop event` when no terminal is seen at `src/upstream/bedrock.rs:338`-`src/upstream/bedrock.rs:342`. JSON chunks set terminal via parsed `type == "message_stop"` at `src/upstream/bedrock.rs:499`-`src/upstream/bedrock.rs:504`, but passthrough SSE chunks only check `text.contains("event: message_stop")` at `src/upstream/bedrock.rs:488`-`src/upstream/bedrock.rs:492`.
  - Impact: valid SSE can express the event name as `event:message_stop`, with CRLF line endings, or as data JSON with no `event:` field. Those frames would be forwarded downstream but not mark `has_seen_terminal_event`, causing a late body error after the visible terminal frame.
  - Proposed fix/test: parse passthrough SSE lines with the same tolerant event-name helper used elsewhere, and fall back to parsing `data:` JSON for `{"type":"message_stop"}`. Extend `tests/integration_bedrock_transport.rs:217`-`tests/integration_bedrock_transport.rs:249` with passthrough variants for `event:message_stop`, CRLF, and JSON-only terminal payloads.

- **B3 - Low - Codex normalizer preserves completed-plus-[DONE] duplicates instead of normalizing terminal policy.**
  - Evidence: Codex REST streaming uses `normalize_sse_stream` and does flush pending bytes on EOF at `src/upstream/codex.rs:956`-`src/upstream/codex.rs:981`; it also strips `codex.*` and splices output items into `response.completed` at `src/upstream/codex.rs:1019`-`src/upstream/codex.rs:1038`. However, terminal detection only covers Responses event types at `src/upstream/codex.rs:1153`-`src/upstream/codex.rs:1168`; the normalizer does not suppress a trailing `data: [DONE]` after `response.completed`. By contrast, the generic Responses parser explicitly ignores `[DONE]` only after a terminal at `tests/integration_responses_sse.rs:28`-`tests/integration_responses_sse.rs:39` and rejects `[DONE]` before terminal at `tests/integration_responses_sse.rs:42`-`tests/integration_responses_sse.rs:50`.
  - Impact: downstream Responses clients may receive both `response.completed` and legacy `[DONE]`, creating duplicate terminal semantics. This is probably compatible with many clients, but it is inconsistent with the internal parser contract and can confuse strict lifecycle accounting.
  - Proposed fix/test: decide the public Responses SSE policy explicitly: either document `[DONE]` passthrough for Codex compatibility or filter `[DONE]` after a terminal in `CodexSseNormalizer`. Add a Codex transport test beside `tests/integration_codex_transport.rs:111`-`tests/integration_codex_transport.rs:147` that includes `data: [DONE]` after `response.completed` and asserts the chosen behavior.

- Commands/evidence used: `rg -n "UpstreamResponse::stream|responses_prepared_stream|Event|Sse|event-stream|\\[DONE\\]|response\\.completed|finish_reason|stream_chat|streamGenerateContent|data:" src tests`; `rg -n "stream\\(|BoxStream|Body::from_stream|into_response|event-stream|text/event-stream|keep_alive|flush" src/upstream_response.rs src/route src/upstream src/sse`; targeted `nl -ba` reads of the file paths above.
- Changed files: `docs/missions/streaming-optimization-1.md` only.

## Agent C: Timeout, Retry, And Cancellation Behavior

### Scope

Audit request timeouts, retry loops, idle-stream behavior, cancellation propagation, dropped downstream clients, and upstream task cleanup.

### Findings

- **C1 — High: Cursor RunStream leaks heartbeat/reader tasks on normal EOF/timeout paths.**
  - Evidence: `src/upstream/cursor/transport.rs:191` defines `RunStream` with `reader_handle` and `heartbeat_handle`, but `rg -n "impl Drop for RunStream|struct RunStream"` shows no `Drop` implementation for `RunStream`. `RunStream::close()` aborts both tasks at `src/upstream/cursor/transport.rs:246`, and some explicit error/drop-consumer paths call it (`src/upstream/cursor/run.rs:181`, `src/upstream/cursor/run.rs:270`, `src/upstream/cursor/run.rs:345`). However, the normal EOF path falls out of the loop at `src/upstream/cursor/run.rs:153` and only calls `take_connect_error()` / emits `Done` at `src/upstream/cursor/run.rs:357` and `src/upstream/cursor/run.rs:386`; it never calls `transport_stream.close()`. The reader task sets `closed=true` and sends END_STREAM at `src/upstream/cursor/transport.rs:338`, but the heartbeat task spawned at `src/upstream/cursor/transport.rs:349` is not stopped unless `close()` is called.
  - Impact: completed Cursor runs can leave heartbeat tasks alive until the shared h2 send path errors; repeated successful runs can accumulate background tasks and extra heartbeat writes.
  - Proposed fix/test: add an idempotent `Drop for RunStream` or `shutdown()` used on all terminal paths, and add a unit/integration test with a fake h2 stream that reaches terminal EOF then asserts heartbeat stops without requiring explicit `close()`.

- **C2 — High: Cursor streaming response open has no timeout around `response_fut.await`.**
  - Evidence: `open_streaming_run()` bounds TCP/TLS/h2 connection setup with `CONNECT_DEADLINE` (`src/upstream/cursor/transport.rs:392`) and the reader loop with `READ_DEADLINE` (`src/upstream/cursor/transport.rs:475`), but `finish_open_streaming_run()` awaits `response_fut` directly at `src/upstream/cursor/transport.rs:317`. The unary Cursor path does wrap its response future with `timeout(READ_DEADLINE, response_fut)` at `src/upstream/cursor/transport.rs:618`, so the streaming open path is inconsistent.
  - Impact: if Cursor accepts the request stream but never returns headers, the spawned Cursor run task in `src/upstream/cursor/run.rs:126` can hang before yielding a provider error, with downstream clients waiting indefinitely.
  - Proposed fix/test: wrap streaming `response_fut` in `timeout(READ_DEADLINE, response_fut)` and add a transport test with an h2 server that accepts the request but withholds response headers.

- **C3 — Medium: Cursor read timeout is total-stream, not idle-stream.**
  - Evidence: `run_reader_loop()` wraps the entire loop in one `timeout(READ_DEADLINE, async { while let Some(chunk) = body.data().await { ... } })` at `src/upstream/cursor/transport.rs:475`. Unlike the unary loop at `src/upstream/cursor/transport.rs:626`, it does not apply the deadline to each `body.data()` await.
  - Impact: long but healthy Cursor runs are capped at 90 seconds total even if frames arrive continuously; conversely this does not implement the intended "idle stream" semantics described by the audit scope.
  - Proposed fix/test: move the `timeout(READ_DEADLINE, body.data())` inside the loop and rename/configure it as an idle read deadline; test that periodic frames beyond 90 seconds continue while a silent gap trips timeout.

- **C4 — Medium: HTTP provider streams rely on Specter defaults only; repo config advertises idle/retry knobs that are not wired locally.**
  - Evidence: shared client construction calls `.streaming_timeouts()` at `src/state.rs:471`, but `rg -n "stream_idle_timeout|stream_max_retries"` only finds README examples (`README.md:322`, `README.md:323`) and no runtime parsing/wiring. Provider streaming sends use `send_streaming()` for Codex (`src/upstream/codex.rs:861`), Bedrock (`src/upstream/bedrock.rs:282`), and Windsurf (`src/upstream/windsurf.rs:197`), with fallback only for non-h2 streaming errors, not bounded idle or request-level cancellation policy in this crate.
  - Impact: operators cannot tune advertised stream idle timeout/retry behavior from this repo, and provider stalls may depend on Specter defaults rather than explicit UMP policy.
  - Proposed fix/test: either wire documented `stream_idle_timeout_ms` / `stream_max_retries` into `RuntimeConfig` and Specter builder if Specter exposes those controls, or remove/update docs; add config parsing tests plus a local stalled-SSE test.

- **C5 — Low/Medium: Responses WebSocket HTTP-bridge task is abort-on-client-drop but not graceful upstream cancel.**
  - Evidence: the mixed-provider Responses WS bridge spawns a provider task at `src/route/websocket.rs:1345`, aborts it on downstream send/read failures at `src/route/websocket.rs:1376` and `src/route/websocket.rs:1395`, but the abort drops `forward_responses_response_to_ws()` and its response body rather than sending a provider-specific cancel/close. Codex WSS is better on dropped downstream clients because it closes the upstream socket at `src/route/websocket.rs:1516`.
  - Impact: HTTP-backed providers may see abrupt dropped response bodies instead of an intentional cancellation signal; this is probably acceptable for HTTP streaming but should be covered by tests so it does not regress into leaked tasks.
  - Proposed fix/test: add an integration test with a local stalled SSE upstream and a downstream WS client that disconnects, then assert the bridge task exits and the upstream body is dropped promptly; consider tracing a bounded cancellation reason.

- Commands/evidence used: `rg -n "READ_DEADLINE|HEARTBEAT_INTERVAL|stream_max_retries|stream_idle_timeout|timeout\\(|tokio::spawn|send_streaming|next_frame|Drop for RunStream|struct RunStream" src tests docs README.md`, `nl -ba src/upstream/cursor/transport.rs`, `nl -ba src/upstream/cursor/run.rs`, `nl -ba src/route/websocket.rs`, `nl -ba src/state.rs`, and `nl -ba src/upstream/codex.rs`. No live provider calls were needed; no secrets or captures collected.
- Changed files: `docs/missions/streaming-optimization-1.md` only.

## Agent D: Header, Method, And Body Forwarding

### Scope

Audit inbound-to-upstream header forwarding, method/path preservation, content-length/transfer-encoding behavior, compression, redaction, and provider-specific forbidden headers.

### Findings

#### D1 — High — Bedrock HTTP/1 streaming fallback drops auth headers

- Evidence: `src/upstream/bedrock.rs:270` clones request headers, `src/upstream/bedrock.rs:271` applies auth, and the first HTTP/2 `send_streaming()` uses those authenticated headers at `src/upstream/bedrock.rs:278`. On non-H2 fallback, `src/upstream/bedrock.rs:289` re-clones `request.headers` and `src/upstream/bedrock.rs:291` sends without re-running `apply_runtime_auth_headers`.
- Impact: if Specter/provider rejects or cannot establish HTTP/2 streaming and `is_non_h2_streaming_error()` triggers, the fallback request loses Bedrock `authorization`. That turns a transport fallback into a false upstream auth failure and can make streaming Bedrock unusable on HTTP/1-only paths.
- Proposed fix: reuse the already-authenticated `headers` in the fallback branch or call `apply_runtime_auth_headers(request, &mut headers).await?` after `let mut headers = request.headers.clone()`.
- Proposed test: add a Bedrock transport unit/integration test that forces the non-H2 fallback path and asserts the fallback request includes `authorization` plus the Bedrock streaming `accept` headers.
- Changed files: none.

#### D2 — Medium — Google direct proxy preserves arbitrary methods for provider-prefixed routes

- Evidence: `src/router.rs:255` wires `/api/provider/google/*path` through `any(route::google::google)`, `src/upstream/google.rs:226` accepts the inbound `Method`, and `src/upstream/google.rs:257` forwards that method unchanged. The generated Google-compatible routes are safer: `src/router.rs:246`-`src/router.rs:253` are `post(...)`, and `src/route/google.rs:27` rejects non-POST for generated routes.
- Impact: method/path preservation is intentional for the generic provider path, but it means DELETE/PUT/PATCH can be proxied to Google if a caller has local proxy access. Header filtering strips caller auth/API key, but method allowlisting is inconsistent between generic Google proxy and generated content routes.
- Proposed fix: document `/api/provider/google/*path` as raw passthrough or restrict to the known Google methods/routes required by clients. Add a route test proving either behavior so future changes do not accidentally broaden the raw proxy.
- Proposed test: call `/api/provider/google/...` with a non-POST method against a mock upstream and assert the expected policy: forwarded unchanged if raw passthrough is accepted, or rejected before upstream if not.
- Changed files: none.

#### D3 — Low — Compression/body-length handling is mostly safe, but needs regression coverage

- Evidence: inbound encoded bodies are decoded by `content_encoding_middleware` at `src/router.rs:264`; `src/request_body.rs:59` removes `content-encoding`, and `src/request_body.rs:61` removes stale `content-length` after decoding. Provider header filters also strip hop-by-hop/length headers: Google strips `content-length` and `transfer-encoding` at `src/upstream/google.rs:348`, while OpenAI public strips sensitive and hop-by-hop headers including `content-length` at `src/upstream/openai_public.rs:35`.
- Impact: the current design avoids forwarding stale lengths and transfer encodings after body replacement. Risk is regression: a future provider-specific helper could bypass the shared middleware/filtering and forward decoded body bytes with stale compression headers.
- Proposed fix: add focused route tests for gzipped JSON through at least Google direct and OpenAI-public/body-preserving surfaces, asserting upstream sees decoded bytes, no `content-encoding`, no inbound `content-length`, and provider-required `content-type`.
- Proposed test: use local mock upstreams only; no real credentials or home auth.
- Changed files: none.

#### D4 — Positive finding — Provider-specific forbidden header filtering is explicit for major HTTP paths

- Evidence: Bedrock uses an allowlist at `src/upstream/bedrock.rs:714` and defaults provider-required `content-type`/`accept` at `src/upstream/bedrock.rs:696` and `src/upstream/bedrock.rs:701`. Google strips `host`, hop-by-hop, caller `authorization`, `cookie`, and `x-goog-api-key` at `src/upstream/google.rs:348`. OpenAI public strips caller auth/API keys plus `openai-*`, `chatgpt-*`, account/session/user/org/project-like headers at `src/upstream/openai_public.rs:211`.
- Impact: no immediate secret-forwarding issue found in these audited paths. The notable exception is D1, which is dropped required provider auth rather than leaked caller auth.
- Proposed fix: keep provider-specific filters centralized and add regression tests whenever new passthrough routes are added.
- Changed files: none.

## Agent E: WebSocket And Mixed-Provider Sessions

### Scope

Audit WebSocket request handling, provider switching after terminal events, upstream WSS/HTTP fallback, backpressure, and close/error propagation.

### Findings

- **E1 - Medium - Malformed in-flight client frames abort the active upstream turn instead of returning a recoverable JSON error.**
  - Evidence: initial parse errors are handled by the outer dispatcher at `src/route/websocket.rs:2078`, so a malformed first frame closes before any upstream execution. During an active HTTP-backed bridge, however, the in-flight reader aborts the provider task on JSON parse failure at `src/route/websocket.rs:1410`-`src/route/websocket.rs:1415` and `src/route/websocket.rs:1454`-`src/route/websocket.rs:1458`. The Codex WSS path has the same fail-fast shape through `handle_codex_inflight_client_message` at `src/route/websocket.rs:1604`-`src/route/websocket.rs:1627`. Unsupported-but-parseable events are recoverable, but malformed frames during an otherwise valid turn tear down the turn and then fall through outer `websocket_proxy_error` close handling.
  - Impact: a client-side stray frame or partial write during a slow provider response can cancel a valid upstream request and lose the terminal event. That is harsher than the existing in-flight `response_already_in_flight` policy, which keeps the connection alive and lets the first turn complete.
  - Proposed fix/test: in the in-flight readers, convert malformed text/binary frames into a top-level `type:error` frame with an `invalid_request_error`/`invalid_json` code and continue reading the active upstream, unless the frame is a Close. Add Codex-WSS and Google/Bedrock-HTTP bridge tests that send a valid slow `response.create`, then malformed text, then assert the first turn still reaches `response.completed` and no second upstream request is made.

- **E2 - Medium - HTTP-backed WebSocket bridge has bounded-channel backpressure but no saturation timeout.**
  - Evidence: Google/Bedrock WebSocket turns are executed by `run_bridge_provider_task`, which creates a bounded `mpsc::channel(REALTIME_WS_QUEUE_CAPACITY)` at `src/route/websocket.rs:1343` and forwards provider SSE frames through `send_bridge_frame` at `src/route/websocket.rs:1720`. If downstream `send_ws_json` blocks, the receiver loop stops draining, the provider task eventually blocks on `sender.send`, and cancellation depends on downstream close/read behavior. The Realtime bridge has explicit queue saturation constants at `src/route/websocket.rs:39`-`src/route/websocket.rs:40`, but this Responses bridge path does not use a comparable timeout around `send_bridge_frame`.
  - Impact: the bounded queue prevents unbounded memory growth, but a slow or wedged downstream can tie up one provider task and one upstream HTTP stream indefinitely instead of failing with a clear `*_queue_saturated` style error.
  - Proposed fix/test: wrap `send_bridge_frame` calls for the Responses HTTP bridge in a timeout similar to Realtime queue saturation, emit a JSON upstream/proxy saturation error when possible, and abort/close the provider stream. Add a test with a mocked provider emitting more than 128 frames while the downstream is not read, asserting the task exits with a bounded error instead of hanging.

- **E3 - Low - Mixed-provider post-terminal switching and in-flight exclusion are currently well covered.**
  - Evidence: route resolution allows Codex WSS, Bedrock HTTP, and Google HTTP bridge lanes at `src/route/websocket.rs:824`-`src/route/websocket.rs:849`, then dispatches Codex to `execute_codex_ws_response` and Google/Bedrock to `run_bridge_provider_task` at `src/route/websocket.rs:958`-`src/route/websocket.rs:964`. Regression coverage proves same-provider model switches after terminal events in `tests/integration_websocket_passthrough.rs:163`, Codex→Google→Bedrock independent turns in `tests/integration_websocket_passthrough.rs:221`, Google/Bedrock prewarm→Codex switching in `tests/integration_websocket_passthrough.rs:295`, Codex in-flight overlap rejection in `tests/integration_websocket_passthrough.rs:413`, and HTTP-backed in-flight overlap rejection in `tests/integration_websocket_passthrough.rs:474`.
  - Impact: no immediate fix recommended for provider switching/fallback routing. The bridge is intentionally fail-closed rather than falling Codex WSS requests back to HTTP, while non-Codex providers use HTTP/SSE bridging.
  - Proposed fix/test: keep these tests as required coverage for future WebSocket transport work, and add analogous tests if Windsurf/Cursor ever become supported over the Responses WebSocket bridge.

- **E4 - Low - Close/error propagation has useful JSON-before-close coverage for Codex WSS, but HTTP bridge close propagation remains indirect.**
  - Evidence: Codex WSS upstream EOF and close before terminal emit JSON `upstream_error` before close handling at `src/route/websocket.rs:1527`-`src/route/websocket.rs:1566` and `src/route/websocket.rs:1702`-`src/route/websocket.rs:1714`; tests cover abrupt disconnect and upstream close at `tests/integration_websocket_passthrough.rs:756` and `tests/integration_websocket_passthrough.rs:801`. HTTP-backed bridge failures are normalized inside `forward_responses_response_to_ws` and `send_bridge_failure` at `src/route/websocket.rs:1161`-`src/route/websocket.rs:1298` and `src/route/websocket.rs:1727`-`src/route/websocket.rs:1748`, but there is no integration test that a Google/Bedrock body error or premature EOF over the WebSocket bridge preserves the JSON-before-close contract.
  - Impact: Codex WSS close/error behavior is protected; HTTP bridge error propagation relies mostly on unit-level/parser coverage and could regress without the mixed-provider WebSocket tests noticing.
  - Proposed fix/test: add WebSocket integration tests with mocked Google/Bedrock SSE that errors mid-body and ends before `response.completed`, asserting top-level `error` before `response.created` and `response.failed` after `response.created`.

- Commands/evidence used: `rg -n "websocket|WebSocket|wss|stream|provider|close|terminal|fallback|backpressure" src tests docs -g '!target'`; `cargo nextest run --test integration_websocket_passthrough` (16/16 passed); targeted `nl -ba`/`sed` reads of `src/route/websocket.rs`, `src/upstream/codex.rs`, `src/upstream/openai_realtime.rs`, and `tests/integration_websocket_passthrough.rs`.
- Changed files: `docs/missions/streaming-optimization-1.md` only.

## Agent F: Live Network Smoke And Logs

### Scope

Run bounded live proxy calls and inspect current logs for hangs, early 200 logs, late body failures, noisy retries, dropped streams, and provider-specific anomalies.

### Findings

- **Environment checked:** live proxy was listening on `127.0.0.1:18743` as PID `98576`; `GET /health` returned `200 OK` with `x-request-id: 47b5062f-3aa5-4f11-a4a2-604859228597`, body version `0.1.11`, git revision `v0.1.11-1-g5946cea-dirty`, build time `2026-05-25T11:00:25Z`.
- **Artifacts:** bounded smoke outputs are in ignored `/tmp/ump-agent-f/` files: `*.json`, `*.headers`, `*.meta`, and `*.body`. Request bodies contain no secrets; log/body snippets were redacted before reporting.
- **Codex non-stream smoke:** `curl --max-time 45 --connect-timeout 5 -H 'content-type: application/json' http://127.0.0.1:18743/v1/responses --data-binary @/tmp/ump-agent-f/codex_responses_nonstream.json` with `model=gpt-5.5`, `stream=false`, `max_output_tokens=32` returned `http_code=200`, `time_total=4.591678`, `time_starttransfer=0.312027`, `size_download=9201`, `content-type: text/event-stream`, `x-request-id: 3440455c-0734-4e62-90d3-2d6a4678b8f6`, 11 `event:` lines and terminal `response.completed` with output `pong`.
- **Codex stream smoke:** same endpoint/body with `stream=true` returned `http_code=200`, `time_total=1.535730`, `time_starttransfer=0.000942`, `size_download=9201`, `content-type: text/event-stream`, `x-request-id: 823ab01b-34b7-4c88-9a79-c43d997ffafb`, 11 `event:` lines and terminal `response.completed` with output `pong`.
- **Cursor field policy smoke:** `composer-2-fast` Responses requests that included `max_output_tokens=32` were rejected locally with `400 Bad Request` and body `bad request: field max_output_tokens is not mapped for Cursor Composer responses`; request IDs `8efd66d2-ce40-4558-b67c-99d4e965df55` and `b070141b-f252-438c-a0a7-1244f7b866c3`. This is provider-specific and expected if the public Responses field policy intentionally disallows that field for Cursor.
- **Cursor non-stream smoke:** rerunning with a minimal Cursor-compatible body, `{"model":"composer-2-fast","input":"Reply with exactly: pong","stream":false}`, returned `http_code=200`, `time_total=1.358125`, `time_starttransfer=1.357797`, `size_download=470`, `content-type: application/json`, `x-request-id: a3305cab-4eb6-49bf-8743-85520365d803`, completed JSON response with output `pong`.
- **Cursor stream smoke:** same minimal body with `stream=true` returned `http_code=200`, `time_total=0.963509`, `time_starttransfer=0.001374`, `size_download=1644`, `content-type: text/event-stream`, `x-request-id: 65ab2fca-73f0-46dd-a1f6-d841f158f574`, 7 `event:` lines and terminal `response.completed` with output `pong`.
- **Early 200 / misleading latency:** current logs record streamed requests as completed before body delivery completes. Examples: Codex stream request `823ab01b-34b7-4c88-9a79-c43d997ffafb` logged `status=200 upstream_status=200 latency_ms=0` while curl observed `time_total=1.535730`; Cursor stream request `65ab2fca-73f0-46dd-a1f6-d841f158f574` logged `latency_ms=0` while curl observed `time_total=0.963509`. Severity: **P1 observability gap** because late body failures/dropped streams can be hidden behind a successful early access log.
- **Late body failures / dropped streams:** no late body failure or dropped stream reproduced in the four bounded live smokes; both successful SSE bodies had terminal `response.completed` and curl exit `0`.
- **Current log anomalies:** `~/Library/Logs/unified-model-proxy-v2.log` contains recurring Specter h2 errors before these smokes: `H2Driver read error: HttpProtocol("Connection closed")` at `2026-05-25T04:11:46Z`, `04:13:42Z`, `05:13:42Z`, `07:13:43Z`, `09:13:43Z`, plus `Read error: Operation timed out (os error 60)` at `2026-05-25T10:14:07Z`. Severity: **P2 unless correlated with request IDs**; they indicate noisy connection-driver failures but current request-completed logs do not link them to a provider/model/request.
- **Provider/model logging anomaly:** earlier current logs show many `provider=codex` or `provider=cursor` `/v1/responses` completions with `model=unknown`, while the bounded body had a model field. The successful smokes after the current build logged `model=gpt-5.5` and `model=composer-2-fast`, so this may already be improved in the dirty live build or may depend on request body shape. Severity: **P3 follow-up**; add regression coverage for model extraction on streaming and non-streaming Responses bodies.
- **Proposed fixes/tests:** for streamed responses, move or supplement `request completed` logging with an end-of-body/SSE-drain observation that records final status, body result, bytes/events written, terminal-event presence, and elapsed body duration; include request IDs on Specter h2 driver errors or bridge them into provider-specific upstream spans; add tests that simulate a stream yielding headers/early 200 followed by body error and assert the log/metric reports the late failure instead of only the early 200.
- **Changed files:** only `docs/missions/streaming-optimization-1.md` was changed by Agent F.

## Consolidated Plan

### Track 1: Correctness Bugs

1. **Fix Bedrock fallback signing.**
   - Reuse the already-signed header map, or re-run `apply_runtime_auth_headers` inside the non-H2 fallback branch.
   - Add a forced-fallback transport test that asserts `authorization`, `x-amz-date`, and Bedrock streaming `accept` headers reach the fallback request.

2. **Harden Cursor stream lifetime.**
   - Wrap streaming `response_fut` with the same deadline discipline used by unary Cursor requests.
   - Convert the reader timeout from total stream lifetime to per-read idle timeout.
   - Add idempotent `RunStream` shutdown/drop handling so heartbeat and reader tasks stop on all terminal paths.

3. **Flush stream translators on EOF.**
   - Replace chunk-only `.map(...).filter_map(...)` wrappers for Google/Anthropic streaming translations with stateful streams that call translator `finish()` when upstream returns `None`.
   - Add tests for final frames without trailing separators and clean EOF after content.

### Track 2: Streaming Contract And Memory Safety

4. **Make buffered fallback explicit and bounded.**
   - Decide whether non-H2 buffered fallback is local/mock-only or allowed in production.
   - If allowed, log/emit `stream_transport=buffered_fallback` and bound all collected bodies.
   - If not allowed, fail fast with a provider error rather than silently buffering a streaming request.

5. **Add limited Specter body collection.**
   - Add a capped Specter body helper aligned with `MAX_UPSTREAM_BODY_BYTES` / `MAX_UPSTREAM_ERROR_BODY_BYTES`.
   - Replace provider-controlled `.bytes()`, `.text()`, and `collect_to_bytes()` sites where large bodies are possible.

6. **Normalize Bedrock terminal detection.**
   - Parse passthrough SSE event lines tolerantly and inspect `data:` JSON for `message_stop`.
   - Add Bedrock event-stream tests for compact event syntax, CRLF, and JSON-only terminal payloads.

### Track 3: Observability And WebSocket Robustness

7. **Add body-drain observability for streaming routes.**
   - Supplement early request-completed logs with end-of-body metrics: body duration, bytes/events written, terminal event seen, body error, and provider/model/request id.
   - Correlate Specter h2 driver errors to request/provider spans when possible.

8. **Bound WebSocket bridge stalls.**
   - Add queue-send timeout behavior for HTTP-backed Responses WebSocket bridge paths.
   - Convert malformed in-flight frames into recoverable JSON errors when possible instead of aborting the active upstream turn.
   - Add mixed WebSocket tests for provider body error, premature EOF, queue saturation, and malformed in-flight frames.

### Track 4: Policy Cleanup

9. **Lock down route/header/body policy with tests.**
   - Decide whether `/api/provider/google/*path` is intentionally raw passthrough or should be method-restricted.
   - Add compression/body-length regression tests through Google and OpenAI-public surfaces.
   - Decide and test Codex `[DONE]` passthrough vs filtering after `response.completed`.
   - Add model extraction tests for streaming and non-streaming Responses logs.

### Suggested Execution Order

1. Bedrock fallback signing.
2. Cursor stream lifetime.
3. EOF flush for Google/Anthropic translators.
4. Body-drain logging and late-error metrics.
5. Limited Specter body collection and buffered-fallback policy.
6. WebSocket bridge saturation/malformed-frame hardening.
7. Policy/test cleanup.

### Validation Gates

- `cargo fmt --check`
- Targeted tests for each changed provider/route.
- `cargo nextest run` after behavior changes.
- `cargo clippy --tests --no-deps --all-features -- -D warnings`
- Bounded live smoke for Cursor/Codex streams after observability changes.
