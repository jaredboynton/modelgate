# UMP v2 - Amp-only Proxy Plan

Target path: `/Users/jaredboynton/__devlocal/amp-research/unified-model-proxy-v2/`
Listen: `127.0.0.1:18743`
Verified Amp bundle: `0.0.1778531432-g3bd093`

## Goal

Build one Rust HTTP proxy for Amp. Keep only three upstreams:

- Bedrock Mantle for Anthropic-shaped traffic.
- Codex/ChatGPT OAuth for OpenAI Responses traffic.
- Google Gemini direct for Gemini traffic.

No broad provider framework. No legacy v1 sprawl. Lift proven v1 algorithms where they matter.

## Scope

Ship v0.1:

- `amp -x "say hi"` works through Bedrock Mantle.
- Codex Responses WSS works through Specter's RFC 6455 WebSocket client in integration tests.
- Codex token refresh works from `~/.codex/auth.json`.
- Google Gemini passthrough works.
- Google 4xx/5xx fallback routes through Bedrock Mantle.
- Gemini painter default works through Google.
- `gpt-image-2` returns typed unsupported error.

Do not ship v0.1:

- Thread storage.
- Telemetry forwarding.
- GitHub proxy.
- Attachments.
- Web extract.
- Separate first-party Anthropic upstream.
- OpenAI-direct.
- Codex WebSocket-over-HTTP/2 or WebSocket-over-HTTP/3 selection.
- Cursor, Windsurf, xAI, Postman, OpenRouter, MiniMax, ZAI, Kimi, Perplexity, Meta AI, ChatGPT-web, Grok-web.
- macOS Keychain.
- Public OpenAI `/v1/images/*` calls with Codex OAuth.

## Core Decisions

- Architecture: greenfield single binary, not v1 workspace clone.
- Reuse: port v1 Codex filter, output splice, Codex rate limits, Google path rewrite, Codex OAuth refresh, atomic private write.
- New code: router, three upstream modules, Bedrock profile discovery, Google-to-Bedrock fallback translator.
- Auth: fail closed. Missing credential returns structured `401`, never silent provider swap.
- Codex tokens: read from `~/.codex/auth.json`. `~/.ump/auth.json` `codex` section is diagnostic mirror only.
- Codex WebSocket default: Specter RFC 6455 over HTTP/1.1 via `Client::websocket()`. Specter now also has RFC 8441 (`websocket_h2`) and RFC 9220 (`websocket_h3`) tunnel APIs, but UMP must not assume ChatGPT Codex supports them without live ALPN/settings evidence.
- Bedrock auth: env/file bearer first, cached profile next, profile discovery last.
- Google auth: `GOOGLE_API_KEY` env only.
- v1 daemons stay on `:18741` and `:18742`; v2 uses `:18743` until proven.

## Module Layout

```text
unified-model-proxy-v2/
  Cargo.toml
  README.md
  launchd/
    dev.unified-model-proxy-v2.plist
  PLANNING/
    bedrock.md
    v2-plan.md
  src/
    main.rs
    router.rs
    error.rs
    state.rs
    model_alias.rs
    rate_limit.rs
    auth/
      mod.rs
      codex.rs
      bedrock.rs
      google.rs
    upstream/
      mod.rs
      codex.rs
      bedrock.rs
      google.rs
    route/
      mod.rs
      health.rs
      models.rs
      messages.rs
      responses.rs
      chat.rs
      google.rs
      images.rs
    sse/
      mod.rs
      filter.rs
      splice.rs
  tests/
    common/
    fixtures/
    unit_*.rs
    integration_*.rs
```

`route::images` can exist in v0.1 only to return explicit unsupported responses for `gpt-image-2`.

## Dependencies

- `axum`, `tokio`, `serde`, `serde_json`, `bytes`, `futures`.
- `tracing`, `tracing-subscriber`.
- `specters` package at or after merged Specter commit `da3ddff` (local package version currently `2.3.0`), imported as crate `specter`.
- `reqwest` with `rustls-tls` and stream support.
- `aws-config`, `aws-credential-types`, `aws-sigv4`.
- `dirs`, `thiserror`, `anyhow`, `uuid`, `fs2`.
- Dev/test: `tempfile`, mock HTTP server, local RFC 6455 WSS-compatible test server. Add H2/H3 Extended CONNECT fixtures only if UMP enables those protocol lanes.

Use `specter` for:

- Codex WSS via the shared RFC 6455 WebSocket codec.
- Codex HTTP fallback.
- OpenAI OAuth refresh.
- Google HTTP/SSE.
- Future Codex H2/H3 WebSocket experiments through `websocket_h2` / `websocket_h3`, not by duplicating handshake or frame logic.

Use `reqwest` only for Bedrock Mantle. AWS signing path already fits `reqwest`; browser TLS fingerprint buys nothing there.

If crates.io lacks the merged Specter WebSocket APIs, use a local/path dependency to `/Users/jaredboynton/__devlocal/specter` or publish Specter first. Do not add `tokio-tungstenite`; Specter owns the WebSocket frame state machine and the RFC 6455 / RFC 8441 / RFC 9220 handshake split.

## Routes

| Method | Path | Handler | Upstream |
|---|---|---|---|
| GET | `/health` | `route::health` | none |
| GET | `/v1/models` | `route::models` | synthesized |
| GET | `/api/provider/openai/v1/models` | `route::models` | synthesized |
| POST | `/api/provider/anthropic/v1/messages` | `route::messages` | Bedrock Mantle |
| POST | `/api/provider/anthropic/v1/messages/count_tokens` | `route::count_tokens` | local approx/stub |
| POST | `/api/provider/openai/v1/responses` | `route::responses` | Codex WSS, HTTP fallback |
| POST | `/api/provider/openai/v1/chat/completions` | `route::chat` | Codex or Bedrock Mantle |
| ANY | `/api/provider/google/*` | `route::google` | Google, Bedrock fallback |
| POST | `/api/provider/openai/v1/images/generations` | `route::images` | v0.1 unsupported |
| POST | `/api/provider/openai/v1/images/edits` | `route::images` | v0.1 unsupported |
| POST | `/v1/messages` | compat | Bedrock Mantle |
| POST | `/v1/responses` | compat | Codex |
| POST | `/v1/chat/completions` | compat | Codex or Bedrock Mantle |

Everything else: `404`, warn log with method/path/query.

Amp path facts:

- Anthropic SDK base: `${AMP_URL}/api/provider/anthropic`, then `/v1/messages?beta=true`.
- OpenAI SDK base: `${AMP_URL}/api/provider/openai/v1`, then `/responses`, `/chat/completions`, or `/images/*`.
- Google SDK base: `${AMP_URL}/api/provider/google`, then `/v1beta1/publishers/google/models/<model>:generateContent`.

## Model Map

Unknown model: `400 model_not_supported`. No fallback.

| Amp id | Provider | Upstream id | Notes |
|---|---|---|---|
| `anthropic/claude-sonnet-4-6` | Bedrock | `anthropic.claude-sonnet-4-6` | Mantle-only routing |
| `anthropic/claude-haiku-4-5` | Bedrock | `anthropic.claude-haiku-4-5` | accepts dated snapshots |
| `anthropic/claude-opus-4-6` | Bedrock | `anthropic.claude-opus-4-6` | Mantle-only routing |
| `anthropic/claude-opus-4-7` | Bedrock | `anthropic.claude-opus-4-7` | defensive alias |
| `openai:gpt-5.5` | Codex | `gpt-5.5` | strip `openai:` |
| `openai/gpt-5.5` | Codex | `gpt-5.5` | strip prefix |
| `openai/gpt-5.4` | Codex | `gpt-5.4` | oracle default |
| `vertexai/gemini-3-flash-preview` | Google | `gemini-3-flash-preview` | strip prefix |
| `gemini-3-flash-preview` | Google | `gemini-3-flash-preview` | raw Amp form |
| `vertexai/gemini-3.1-pro-preview` | Google | `gemini-3.1-pro-preview` | code review |
| `gemini-3.1-flash-lite` | Google | `gemini-3.1-flash-lite` | GA flash-lite |
| `vertexai/gemini-3-pro-image` | Google | `gemini-3-pro-image-preview` | defensive |
| `gemini-3-pro-image` | Google | `gemini-3-pro-image-preview` | painter selector |
| `gemini-3-pro-image-preview` | Google | `gemini-3-pro-image-preview` | actual SDK model |
| `gpt-image-2` | unsupported v0.1 | n/a | future Codex hosted image tool |

Rows where `accepts_dated_snapshots == true` also accept any `<id>-YYYYMMDD` suffix at resolve time; the date is stripped before upstream dispatch.

`gemini-3.1-flash-lite-preview` is intentionally NOT registered (sunset 2026-05-25 per Google's release calendar).

`/chat/completions` rule:

- `gpt-*` -> translate inbound chat body to Responses body -> Codex.
- `claude-*` or `anthropic/*` -> translate to Anthropic Messages body -> Bedrock Mantle.
- Else `400 model_not_supported`.

Keep `/chat/completions` as contingency. If 24h soak shows zero Amp traffic, drop in v2.2.

## Auth

### Codex

Read source:

- `~/.codex/auth.json` only.

Write targets after successful refresh:

- `~/.codex/auth.json`.
- `~/.ump/auth.json` merged `codex` diagnostic section.

Refresh request:

```text
POST https://auth.openai.com/oauth/token
grant_type=refresh_token
client_id=app_EMoamEEZ73f0CkXaXp7hrann
refresh_token=<refresh_token>
```

Rules:

- Accept UMP shape and OpenCode shape.
- Extract `account_id` from auth file or `id_token`.
- `ChatGPT-Account-Id` header uses account id when present.
- `organization_id` is not required for Codex backend.
- `~/.ump/auth.json` `codex` section never supplies access token.
- In-process `tokio::sync::Mutex` collapses UMP refreshes.
- `fs2` lock on `~/.codex/auth.json.lock` protects UMP writers only. Codex CLI does not honor it.
- Before write: re-stat + content hash. If file changed, abort write, re-read, retry request once. Do not call OAuth refresh twice with same refresh token.
- Atomic write: port v1 `atomic_write_private` (`tmp`, write, fsync, rename, parent fsync, mode `0600`).

### Bedrock

Resolution order:

1. `~/.ump/auth.json.bedrock.bearer`.
2. `~/.ump/auth.json.bedrock.profile`.
3. `AWS_BEARER_TOKEN_BEDROCK` env.
4. Discovery.

Discovery:

- Enumerate `~/.aws/config` and `~/.aws/credentials`.
- Skip expired SSO profiles.
- Probe max 4 profiles concurrently.
- Probe by real Mantle ping with `max_tokens=1`.
- First `200` wins.
- Cache profile under `~/.ump/auth.json.bedrock.profile`.
- Wall clock cap: `UMP_V2_BEDROCK_DISCOVERY_TIMEOUT_MS=5000`.
- No IAM simulation.
- No `ListFoundationModels`.

On Bedrock `401/403`: invalidate cache, resolve again, retry once.

### Google

Resolution order:

1. `~/.ump/auth.json.gemini.api_key`.
2. `~/.ump/auth.json.google.api_key` compatibility alias.
3. `GOOGLE_API_KEY` env.

No OAuth.

## Upstream Flows

### Bedrock Messages

Inbound:

```text
POST /api/provider/anthropic/v1/messages?beta=true
```

Proxy:

- Normalize model.
- Drop `?beta=true`.
- Preserve Anthropic Messages body.
- Send to `https://bedrock-mantle.us-east-1.api.aws/anthropic/v1/messages`.
- Stream SSE back byte-for-byte where possible.

### Codex Responses

Inbound:

```text
POST /api/provider/openai/v1/responses
```

Default transport:

```text
wss://chatgpt.com/backend-api/codex/responses
```

Headers:

- `Authorization: Bearer <access_token>`.
- `ChatGPT-Account-Id: <account_id>` when present.
- `originator: codex_cli_rs`.
- `OpenAI-Beta: responses_websockets=2026-02-06` for WSS.

Do not copy v1 Rust's `originator: opencode` constant unless a live backend smoke test proves `codex_cli_rs` regressed. The TS adapter and Codex notes use `codex_cli_rs`.

Default WebSocket protocol lane:

- Use `specter::Client::websocket()` for `wss://chatgpt.com/backend-api/codex/responses`.
- This is the RFC 6455 path: HTTP/1.1 Upgrade, `Sec-WebSocket-Key`, `Sec-WebSocket-Accept`, masked client frames, and shared Specter frame parsing.
- UMP code should pass headers and messages into Specter, not build Upgrade headers or WebSocket frames itself.
- Do not use `websocket_h2()` for Codex unless a live capture or server fixture proves ALPN `h2` plus `SETTINGS_ENABLE_CONNECT_PROTOCOL = 1` (setting `0x8`).
- Do not use `websocket_h3()` for Codex unless a live capture or server fixture proves HTTP/3 plus `SETTINGS_ENABLE_CONNECT_PROTOCOL = 1` (setting `0x8`).
- On H2/H3 paths, the handshake is Extended CONNECT with `:method = CONNECT`, `:protocol = websocket`, `:scheme`, `:path`, `:authority`; success is `:status = 200`, not `101`.
- Never send HTTP/1.1 WebSocket bootstrap headers (`Upgrade`, `Connection`, `Host`, `Sec-WebSocket-Key`, `Sec-WebSocket-Accept`) on H2/H3 paths.

WSS message:

```json
{ "type": "response.create", "response": "<inbound Responses body>" }
```

Stream handling:

- Drop `event: codex.*`.
- Preserve public `response.*` SSE events.
- Accumulate `response.output_item.done`.
- Splice accumulated `output[]` into terminal `response.completed`.
- Avoid cloning large image/base64 payloads in splice path.

Transport env:

```text
UMP_V2_CODEX_TRANSPORT=wss|http|wss-then-http
```

Default: `wss-then-http`.

WebSocket protocol selection:

- v0.1 hardcodes RFC 6455 for Codex WSS.
- Keep an internal enum shape if useful, but reject `rfc8441` / `rfc9220` configuration until there is live endpoint evidence and a dedicated test fixture.

Fallback latch:

- If WSS gets >=3 non-101 handshake responses or WebSocket handshake errors within 10s, set process-global `disable_websockets=true`.
- Use HTTP SSE for rest of process lifetime.
- `SIGHUP` clears latch.
- WSS connect timeout: `UMP_V2_CODEX_WSS_CONNECT_TIMEOUT_MS=5000`.

Rate limits:

- Codex concurrency cap default `20`.
- Handshake bucket default `55/min`.
- Env override: `UMP_V2_CODEX_MAX_CONCURRENT`, `UMP_V2_CODEX_HANDSHAKES_PER_MIN`.

### Google

Inbound:

```text
/api/provider/google/v1beta1/publishers/google/models/<model>:generateContent
/api/provider/google/v1beta1/publishers/google/models/<model>:streamGenerateContent?alt=sse
```

Rewrite:

```text
https://generativelanguage.googleapis.com/v1beta/models/<model>:generateContent
https://generativelanguage.googleapis.com/v1beta/models/<model>:streamGenerateContent?alt=sse
```

Header/body rules:

- Replace inbound auth with `x-goog-api-key: $GOOGLE_API_KEY`.
- Strip Amp-only headers.
- Preserve JSON body.
- Preserve SSE for stream calls.

Fallback:

- On Google `4xx/5xx`, translate Gemini request to Anthropic Messages.
- Send through Bedrock Mantle.
- Translate Anthropic text response back to Gemini `GenerateContentResponse`.
- No separate first-party Anthropic fallback.

## Images

v0.1 behavior:

- Default Amp painter uses Gemini path. Support it through Google.
- `painter.model = "gpt-image-2"` returns:

```json
{
  "error": {
    "type": "model_not_supported",
    "message": "gpt-image-2 not supported by ump-v2 v0.1; use the default Gemini painter path"
  }
}
```

Important: Codex OAuth is not a public OpenAI API key. Do not call:

- `https://api.openai.com/v1/images/generations`
- `https://api.openai.com/v1/images/edits`

with `~/.codex/auth.json` access token.

Observed Codex Desktop image path:

- App generates image through `wss://chatgpt.com/backend-api/codex/responses`.
- Stream contains `image_generation_call`.
- `image_generation_call.result` is base64 PNG.
- Local app saves PNG under `~/.codex/generated_images/<thread_id>/<call_id>.png`.
- Public `/v1/images/*` fallback code exists only for real `OPENAI_API_KEY`.

Amp `gpt-image-2` inbound routes:

- `POST /api/provider/openai/v1/images/generations`
- `POST /api/provider/openai/v1/images/edits`

Amp generate body:

```json
{ "model": "gpt-image-2", "prompt": "...", "output_format": "png" }
```

Amp edit body:

```text
multipart/form-data
model=gpt-image-2
prompt=...
output_format=png
image=<File[]>
```

v2.1 generation path:

1. Accept OpenAI Images generate shape.
2. Translate to Codex Responses hosted tool request:

   ```json
   {
     "model": "gpt-5.5",
     "input": "<image prompt>",
     "tools": [
       { "type": "image_generation", "output_format": "png" }
     ]
   }
   ```

3. Send over existing Codex RFC 6455 WSS path.
4. Collect terminal `image_generation_call`.
5. Return OpenAI Images-compatible JSON:

   ```json
   {
     "output_format": "png",
     "data": [
       {
         "b64_json": "<image_generation_call.result>",
         "revised_prompt": "<image_generation_call.revised_prompt>"
       }
     ]
   }
   ```

v2.1 edit gate:

- Do not implement `/images/edits` from guessing.
- Need real Codex Desktop image-edit trace or fixture proving image-input shape.
- Until then return typed `not_implemented`.

Image safety:

- Never log raw base64 image payload.
- Redact as `<redacted_base64_png len=N sha256=...>`.
- Keep Specter `max_message_size` at the default 16 MB or higher before image support.
- Keep splice path streaming/owned enough to avoid duplicate 2 MB+ payload copies.

## Tests

Day-1 v0.1 gates:

- `UT-ALIAS`: all model aliases above.
- `UT-AUTH-CODEX`: UMP + OpenCode auth shapes parse.
- `UT-GOOGLE-PATH`: vertex path rewrites to Gemini path.
- `UT-SSE-FILTER`: `event: codex.*` drops, `event: response.*` stays.
- `UT-CODEX-WS-PROTOCOL`: Codex WSS defaults to RFC 6455; RFC 8441/RFC 9220 config is rejected unless explicitly enabled by a future fixture-backed gate.
- `IT-CODEX-WSS`: mock RFC 6455 WSS emits codex event, output item, completed event; client sees no codex event and terminal output is spliced through Specter.
- `IT-CODEX-WSS-HANDSHAKE-FAIL`: repeated non-101 Specter handshake failures trip the HTTP fallback latch.
- `IT-CODEX-REFRESH`: expired fixture refreshes; both auth files update in temp homes.
- `IT-BEDROCK-SIGV4`: outbound `Authorization` starts `AWS4-HMAC-SHA256`.
- `IT-BEDROCK-BEARER`: outbound `Authorization: Bearer test`.
- `IT-BEDROCK-DISCOVERY`: bad/expired/good profile set; good profile cached and used.
- `IT-GOOGLE-PASS`: rewritten path, `x-goog-api-key`, body byte-equal.
- `IT-GOOGLE-FALLBACK`: Google 503 becomes Bedrock call, then Gemini response.
- `IT-HEALTH`: no upstream contacted, returns `{"status":"ok"}`.
- `E2E-AMP-1`: `AMP_URL=http://127.0.0.1:18743 amp -x "say hi"` exits 0 within 120s with non-empty assistant text.

v0.2 gates:

- Codex HTTP fallback.
- Codex WSS latch after non-101 / 403 burst.
- Live Codex WSS capture decision: stay RFC 6455 or add a separately tested RFC 8441/RFC 9220 lane.
- Codex auth file race.
- Bedrock cache invalidation on 401.
- Google stream passthrough.
- `/v1/models` stable ordering.
- Real Amp Codex mode E2E.
- Real Amp `look_at` E2E.
- Chat completions soak/drop decision.

Test isolation:

- Production path defaults:
  - `~/.codex`
  - `~/.ump`
- Test overrides:
  - `UMP_V2_CODEX_HOME`
  - `UMP_V2_AUTH_HOME`
- Tests use `tempfile::TempDir`.
- Harness refuses real home paths.
- Only `tests/common/mod.rs` builds `AppState`.

## Observability

Default `RUST_LOG=info`.

Per-request span fields:

- `request_id`
- `route`
- `provider`
- `model`
- `upstream_status`
- `upstream_latency_ms`
- `bytes_in`
- `bytes_out`

Codex fields:

- `transport`
- `ws_protocol=rfc6455|rfc8441|rfc9220`
- `handshake_wait_ms`
- `concurrent_in_flight`
- `wss_latch`

Bedrock fields:

- `auth_path=bearer_env|bearer_file|profile_cached|profile_discovered`

Failure capture:

- Path: `~/.ump/v2-failures/<request_id>.json`.
- Trigger: upstream `5xx` or OAuth refresh non-200.
- Cap: 100 files FIFO.
- Redact headers: `Authorization`, `x-api-key`, `x-goog-api-key`, `cookie`.
- Redact fields: `refresh_token`, image base64 payloads.

No metrics endpoint in v0.1.

## Risks

| Risk | Signal | Mitigation |
|---|---|---|
| Codex WSS handshake ceiling | 403 burst, long handshake wait | bucket + concurrency cap + HTTP latch |
| Codex WebSocket protocol mismatch | non-101, missing `SETTINGS_ENABLE_CONNECT_PROTOCOL`, H3 Extended CONNECT failure | default to RFC 6455; enable RFC 8441/RFC 9220 only after live capture and fixture tests |
| Bedrock bearer/profile revoked | 401/403 | invalidate cache, resolve chain, retry once |
| Codex refresh race with Codex CLI | file mtime/hash changes, `invalid_grant` | content-hash recheck, no second OAuth refresh |
| `specters` BoringSSL build break | `boring-sys` compile/link error | document `unset OPENSSL_*`, require `cmake` + `ninja`, use local Specter lock/path while publishing catches up |
| AWS SSO discovery hang | expired SSO profile | skip expired SSO cache, per-probe connect timeout |
| Amp route drift after update | 404 under `/api/provider/*` | warn unmatched paths, route-grep script, pin verified Amp version |
| Image payload memory/log blowup | multi-MB base64 in stream/log | redact, no raw logging, avoid clone-heavy splice |

## Follow-ups

- Keychain-backed auth at rest.
- Promote richer `failure_capture` from v1 if needed.
- Optional offline body-shape parity fixtures only.
- Thread storage, telemetry, GitHub proxy only if Amp boot or workflows force it.
- Uninstall v1 launchd plists after v2 soak.
- Codex hosted image generation for `gpt-image-2` through Responses hosted tool.
- Live capture of ChatGPT Codex WSS ALPN/HTTP version; only then consider enabling Specter RFC 8441 or RFC 9220 for Codex.
- Route removal after soak: `/chat/completions`, `count_tokens` stub if unused.

## Evidence Anchors

- Amp deobfuscated bundle: `amp-ref/bundle.pretty.js`.
- Amp tool model notes: `AMP-TOOL-MODELS.md`.
- Bedrock active model notes: `unified-model-proxy-v2/PLANNING/bedrock.md`.
- v1 Codex constants and WSS adapter: `unified-model-proxy-rs/crates/ump-auth/src/lib.rs`, `unified-model-proxy-rs/crates/ump-adapters/ump-adapter-codex-responses-wss/src/lib.rs`.
- TS Codex adapter originator evidence: `unified-model-proxy/src/lib/adapters/codex-responses.ts`, `unified-model-proxy/src/lib/responses-compat/handler.ts`.
- v1 SSE filter/splice: `unified-model-proxy-rs/crates/ump-compat/src/lib.rs`.
- Specter merged WebSocket support: `/Users/jaredboynton/__devlocal/specter` commits `44c4769` (RFC 8441), `f24676c` (RFC 9220), `da3ddff` (merged into main).
- Specter WebSocket tests: `tests/rfc6455_websocket.rs`, `tests/websocket_handshake.rs`, `tests/rfc8441_*`, `tests/rfc9220_*`.
- Protocol references: RFC 6455 WebSocket (`https://www.rfc-editor.org/rfc/rfc6455.html`), RFC 8441 WebSocket over HTTP/2 (`https://www.rfc-editor.org/rfc/rfc8441.html`), RFC 9220 WebSocket over HTTP/3 (`https://www.rfc-editor.org/rfc/rfc9220.html`).
