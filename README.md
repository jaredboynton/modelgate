# Unified Model Proxy v2

Rust HTTP proxy for Amp listening on `127.0.0.1:18743`.

v0.1 keeps three upstreams only: Bedrock Mantle for Claude/Anthropic-shaped
traffic, Codex/ChatGPT OAuth, and Google Gemini direct. See `PLANNING/v2-plan.md`
for the implementation contract.

The proxy also serves Amp's local control-plane compatibility surface:
thread/task/report storage, thread search/markdown, attachments, telemetry,
GitHub/Bitbucket helper stubs, and startup account/plugin probes.

UMP's OpenAI-shaped routes are an OpenAI-compatible facade, not a claim of
public OpenAI API parity. Each route resolves a model to an explicit provider
disposition before upstream dispatch. Today, GPT Responses and chat-compatible
traffic use Codex OAuth from `~/.codex/auth.json`; public OpenAI API-key
transport is not a fallback.

## Run Locally

Debug run:

```sh
cargo run
```

Release run:

```sh
cargo build --release
./target/release/unified-model-proxy-v2
```

Amp smoke:

```sh
AMP_URL=http://127.0.0.1:18743 amp -x "say hi"
```

Health check:

```sh
curl -fsS http://127.0.0.1:18743/health
```

Config UI:

```sh
open http://127.0.0.1:18743/config
```

Safe config UI smoke:

```sh
tmpdir="$(mktemp -d)"
export UMP_V2_AUTH_HOME="$tmpdir/auth"
export UMP_V2_CODEX_HOME="$tmpdir/codex"
export UMP_V2_CONFIG="$tmpdir/config.json"
export UMP_V2_LISTEN_ADDR="127.0.0.1:0"
printf '{"routes":[]}' > "$UMP_V2_CONFIG"
cargo run
```

Use the ephemeral address printed by the process, wait for `/health`, then open
`/config` with a CDP/browser runner pointed only at that temp process. Capture a
screenshot or DOM dump plus browser console output, then verify the route map,
typed editor, diagnostics, validate, preview, and save controls.

## Runtime Config

- `UMP_V2_LISTEN_ADDR`, default `127.0.0.1:18743`
- `UMP_V2_CODEX_TRANSPORT`, one of `wss`, `http`, `wss-then-http`; default `wss-then-http`
- `UMP_V2_CODEX_RESPONSES_WSS_URL`, default `wss://chatgpt.com/backend-api/codex/responses`
- `UMP_V2_CODEX_RESPONSES_HTTP_URL`, default `https://chatgpt.com/backend-api/codex/responses`
- `UMP_V2_CODEX_WSS_CONNECT_TIMEOUT_MS`, default `5000`
- `UMP_V2_CODEX_MAX_CONCURRENT`, default `20`
- `UMP_V2_CODEX_HANDSHAKES_PER_MIN`, default `55`
- `UMP_V2_BEDROCK_DISCOVERY_TIMEOUT_MS`, default `5000`
- `UMP_V2_CODEX_HOME`, defaults to `~/.codex`
- `UMP_V2_AUTH_HOME`, defaults to `~/.ump`
- `UMP_V2_CONFIG`, defaults to `~/.ump/config.json`; re-read on every request
- `UMP_AMP_THREAD_STORE`, defaults to `~/.unified-model-proxy/amp-threads`
  for compatibility with the earlier Rust proxy's local thread store

Do not put live secrets in committed files. Local `.env` files are for humans only.
Provider secrets live in `~/.ump/auth.json`, except Codex OAuth which stays in
the canonical `~/.codex/auth.json`.

Auth file shape:

```json
{
  "bedrock": {
    "bearer": "ABSK...",
    "profile": "optional-aws-profile"
  },
  "gemini": {
    "api_key": "AIza..."
  }
}
```

Bedrock and Gemini prefer `~/.ump/auth.json`; environment variables remain
fallbacks for manual runs. `google.api_key` is accepted as a compatibility alias
for `gemini.api_key`.

Hot routing config:

```json
{
  "routes": [
    {
      "source": { "model": "gemini-3.1-flash-lite", "format": "responses" },
      "target": { "provider": "codex", "model": "gpt-5.5", "format": "responses" }
    }
  ]
}
```

Changing the JSON file affects the next request without restarting the proxy.

The browser config surface is split by purpose:

- `/config` shows the local route map for static routing state. It should show
  built-in effective routes even when the hot config is `{ "routes": [] }`.
- `/api/config` is the raw hot-config compatibility API. `GET` reads persisted
  JSON and strict `PUT` saves a full valid hot-config document for later
  requests.
- `/api/config/graph` is the Switchyard Atlas graph projection used by the route
  map and typed editor. `GET` projects persisted config; `POST` validates and
  projects a draft without writing it, rejecting unknown fields and
  secret-shaped keys.

These routes are local admin surfaces. They require loopback Host values, reject
cross-site unsafe browser writes, return `Cache-Control: no-store`, and keep the
UI on same-origin CSP-protected assets.

Responses WebSocket passthrough:

- `GET /v1/responses` and `GET /api/provider/openai/v1/responses` accept RFC 6455 upgrades.
- The first client frame can be raw Responses JSON or a `response.create` event.
- The proxy applies hot routing for `format: "responses"` on every `response.create`.
  Codex-backed targets use upstream Responses WSS with Codex OAuth; Bedrock and
  Google targets use provider-isolated HTTP/SSE bridges and stream back over the
  same downstream WebSocket.

OpenAI-compatible facade disposition:

- `/v1/responses` and `/v1/chat/completions` are aggregate facade routes. They
  route by requested model: Codex-backed GPT models use Codex OAuth,
  Claude-shaped models go through Bedrock adapters, and Gemini models go
  through Google adapters where that edge exists.
- `/api/provider/openai/v1/responses` and
  `/api/provider/openai/v1/chat/completions` are OpenAI-shaped facade
  entrypoints. Codex-backed GPT dispositions use Codex OAuth; these routes are
  neither blanket public OpenAI API proxies nor blanket Codex backends for
  every model.
- `/api/provider/openai/v1/models` fetches the live Codex model catalog from
  the ChatGPT Codex backend using Codex OAuth headers.
- `/v1/models` returns the aggregate catalog: built-in static models plus
  configured hot-route models. It does not call the live Codex catalog.
- Public-only OpenAI features that are not mapped to Codex or another explicit
  provider fail closed with `unsupported_route`, `model_not_supported`, or
  `invalid_request` instead of falling through to public OpenAI.
- A future `public-openai` API-key provider may be added as an explicit,
  disabled-by-default provider. It must not become an implicit fallback for
  Codex-backed GPT flows.

## Codex CLI

Codex only accepts Responses API providers, so point `~/.codex/config.toml`
at the proxy's `/v1` base URL. The mixed `proxy` profile presents every listed
text model as Responses WebSocket-capable. Native Codex/OpenAI models stay on
upstream Responses WSS; Bedrock/Claude and Gemini models are bridged inside UMP
and streamed back to Codex over the same downstream WSS contract. Plain HTTP
Responses remains supported for non-Codex clients.

Keep request compression enabled for the proxy, but keep remote compaction off
for mixed-provider profiles until UMP owns provider-aware compaction.
`name = "OpenAI"` is a Codex compatibility shim for OpenAI-shaped transport
behavior such as zstd request compression. It is not proof that every UMP-routed
model accepts OpenAI/Codex opaque compaction items.

```toml
[features]
enable_request_compression = true
remote_compaction_v2 = false

[model_providers.ump-v2]
# Compatibility shim: keep OpenAI-shaped transport behavior for Codex.
name = "OpenAI"
base_url = "http://127.0.0.1:18743/v1"
wire_api = "responses"
requires_openai_auth = false
supports_websockets = true
request_max_retries = 0
stream_max_retries = 0
stream_idle_timeout_ms = 120000
websocket_connect_timeout_ms = 10000

[profiles.proxy]
model = "claude-sonnet-4-6"
model_provider = "ump-v2"
model_catalog_json = "/Users/jaredboynton/.codex/model-catalog-ump-v2.json"
model_reasoning_effort = "high"
```

The Codex-only WSS split profile can stay around as the remote-compaction
exception. Use it only for Codex/OpenAI-native models where opaque native
compaction may safely round-trip to the same provider family:

```toml
[model_providers.ump-v2-codex-ws]
# Compatibility shim for native Codex/OpenAI Responses over UMP.
name = "OpenAI"
base_url = "http://127.0.0.1:18743/v1"
wire_api = "responses"
requires_openai_auth = false
supports_websockets = true
request_max_retries = 0
stream_max_retries = 0
stream_idle_timeout_ms = 120000
websocket_connect_timeout_ms = 10000

[profiles.proxy-ws]
model = "gpt-5.5"
model_provider = "ump-v2-codex-ws"
model_catalog_json = "/Users/jaredboynton/.codex/model-catalog-ump-v2-codex-ws.json"

[profiles.proxy-ws.features]
remote_compaction_v2 = true
```

`supports_websockets` is provider-wide in Codex CLI, so UMP owns the mixed
provider facade. After a response reaches a terminal event, the same downstream
socket may send an independent `response.create` for a different provider or
model. While a response is in flight, another `response.create` is rejected with
`response_already_in_flight`. `previous_response_id` is connection-local and
must exactly match the prior route/model fingerprint; it never authorizes a
cross-provider continuation.

Useful model choices:

- `gpt-5.5` / `openai:gpt-5.5` routes to the Codex/ChatGPT OAuth endpoint
  with credentials from `~/.codex/auth.json`.
- `claude-sonnet-4-6` routes Responses requests through the Anthropic adapter
  to Bedrock Runtime.
- `claude-sonnet-4-6-max` uses the same Bedrock Runtime model with
  proxy-forced Anthropic `max` adaptive thinking.
- `gemini-3.1-flash-lite` routes Responses requests through the Google adapter
  to Gemini `generateContent`.

The long-term adapter matrix and missing cross-format edges live in
`PLANNING/adapter-matrix.md`.

## Signals

- `SIGINT` or `SIGTERM`: graceful server shutdown.
- `SIGHUP`: reset the Codex WebSocket failure latch while keeping the server running.

## launchd

The development plist is `launchd/dev.unified-model-proxy-v2.plist`. It points at
the release binary in this checkout, so build first:

```sh
cargo build --release
launchctl bootstrap gui/$(id -u) "$PWD/launchd/dev.unified-model-proxy-v2.plist"
launchctl kickstart -k gui/$(id -u)/dev.unified-model-proxy-v2
```

For the installed user LaunchAgent, copy the plist to
`~/Library/LaunchAgents/dev.unified-model-proxy-v2.plist` and bootstrap that
path. The agent runs at load, keeps itself alive, and binds
`127.0.0.1:18743`.

Stop and unload:

```sh
launchctl bootout gui/$(id -u)/dev.unified-model-proxy-v2
```

Troubleshooting:

```sh
lsof -nP -iTCP:18743 -sTCP:LISTEN
tail -f ~/Library/Logs/unified-model-proxy-v2.log
```

For active development, prefer `cargo run`; launchd is intended to exercise the
same binary shape Amp will call in a local service setup.
