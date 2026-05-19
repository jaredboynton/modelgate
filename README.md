# Unified Model Proxy v2

Unified Model Proxy v2 is a local Rust/Axum proxy for agent clients. It binds
`127.0.0.1:18743` by default and presents OpenAI-compatible, Anthropic-compatible,
Google-compatible, Cursor-backed, and Amp-local compatibility surfaces from one
process.

Routing is explicit and catalog-driven. Requests resolve the submitted model to a
known provider before any credential lookup or upstream call; unknown models and
unsupported public-OpenAI-only features fail closed instead of falling through to a
secret or best-effort provider.

## Current Surface

- OpenAI-shaped aggregate facade: `/v1/responses` (HTTP and WebSocket),
  `/v1/chat/completions`, `/v1/models`, response retrieval/input-items, and
  `/v1/responses/compact`.
- Provider-shaped compatibility routes under `/api/provider/openai/v1/*`,
  `/api/provider/anthropic/v1/messages`, `/api/provider/anthropic/v1/messages/count_tokens`,
  and `/api/provider/google/*`.
- Gemini GenerateContent compatibility paths: `/v1beta/models/*`, `/v1/models/*`,
  `/v1/projects/*`, and `/v1beta1/projects/*`.
- Local admin/config: `/health`, `/config`, `/api/config`, and `/api/config/graph`.
- Amp compatibility: internal probes, telemetry, attachment storage, thread
  lookup/markdown, RSS, GitHub/Bitbucket helper stubs, and audio/realtime/image
  compatibility stubs where implemented.

## Providers

| Provider | Models and surface | Auth and transport |
|---|---|---|
| Bedrock | Claude aliases such as `claude-sonnet-4-6`, `claude-opus-4-6`, `claude-opus-4-7`, and `claude-haiku-4-5` | Bedrock/Mantle auth from `~/.ump/auth.json` or `AWS_BEARER_TOKEN_BEDROCK`; region from `AWS_REGION`/`AWS_DEFAULT_REGION`; Anthropic Messages and Responses bridge |
| Codex | GPT/Codex aliases such as `gpt-5.5`, `gpt-5.4`, `gpt-5.4-mini`, and `gpt-5.3-codex` | ChatGPT Codex OAuth from `~/.codex/auth.json`; Responses over HTTP or WSS |
| Google | Gemini aliases such as `gemini-3.1-flash-lite`, `gemini-3.1-pro-preview`, and image-capable Gemini IDs | `gemini.api_key` in `~/.ump/auth.json` or `GOOGLE_API_KEY`; GenerateContent plus Responses bridge |
| Cursor | `composer-1.5`, `composer-2`, `composer-2-fast`, plus live usable-model discovery when credentials work | Cursor AgentService over h2/Connect; auth from `CURSOR_ACCESS_TOKEN`, system secret store, or `<UMP_V2_AUTH_HOME>/.cursor/auth.json` |

The canonical static allowlist lives in `src/model_alias.rs`; `/v1/models` also
includes hot-route models and any live Cursor models discovered at request time.
`/api/provider/openai/v1/models` fetches the live Codex model catalog through
Codex OAuth.

## Run Locally

```sh
cargo run
```

Release build:

```sh
cargo build --release
./target/release/unified-model-proxy-v2
```

Health check:

```sh
curl -fsS http://127.0.0.1:18743/health
```

Amp smoke:

```sh
AMP_URL=http://127.0.0.1:18743 amp -x "say hi"
```

Config UI:

```sh
open http://127.0.0.1:18743/config
```

Safe config UI smoke with throwaway homes:

```sh
tmpdir="$(mktemp -d)"
export UMP_V2_AUTH_HOME="$tmpdir/auth"
export UMP_V2_CODEX_HOME="$tmpdir/codex"
export UMP_V2_CONFIG="$tmpdir/config.json"
export UMP_V2_LISTEN_ADDR="127.0.0.1:0"
printf '{"routes":[]}' > "$UMP_V2_CONFIG"
cargo run
```

Use the printed ephemeral address, wait for `/health`, then open `/config` and
verify the route map, typed editor, diagnostics, validate, preview, and save
controls.

## Runtime Config

Common environment variables:

- `UMP_V2_LISTEN_ADDR`, default `127.0.0.1:18743`
- `UMP_V2_CODEX_TRANSPORT`, one of `wss`, `http`, `wss-then-http`; default `wss-then-http`
- `UMP_V2_CODEX_RESPONSES_WSS_URL`, default `wss://chatgpt.com/backend-api/codex/responses`
- `UMP_V2_CODEX_RESPONSES_HTTP_URL`, default `https://chatgpt.com/backend-api/codex/responses`
- `UMP_V2_CODEX_WSS_CONNECT_TIMEOUT_MS`, default `5000`
- `UMP_V2_CODEX_MAX_CONCURRENT`, default `20`
- `UMP_V2_CODEX_HANDSHAKES_PER_MIN`, default `55`
- `UMP_V2_CODEX_CLIENT_VERSION`, default from `src/codex_catalog.rs`
- `UMP_V2_CODEX_CATALOG_TTL_SECS`, default `600`
- `UMP_V2_GOOGLE_GENERATE_BASE_URL`, default `https://generativelanguage.googleapis.com`
- `UMP_V2_BEDROCK_DISCOVERY_TIMEOUT_MS`, default `5000`
- `UMP_V2_CODEX_HOME`, default `~/.codex`
- `UMP_V2_AUTH_HOME`, default `~/.ump`
- `UMP_V2_CONFIG`, default `~/.ump/config.json`; re-read on each request
- `UMP_AMP_THREAD_STORE`, default `~/.unified-model-proxy/amp-threads`
- Cursor knobs: `CURSOR_ACCESS_TOKEN`, `CURSOR_REFRESH_URL`,
  `CURSOR_CLIENT_VERSION`, `UMP_CURSOR_CLIENT_PROFILE_OVERRIDE`,
  `UMP_CURSOR_TRUST_CLIENT_HEADERS`, `UMP_CURSOR_WORKSPACE_DIR`,
  `UMP_CURSOR_WORKSPACE_ALLOWLIST`, `UMP_CURSOR_INDEX_CLOUD`,
  `UMP_CURSOR_INDEX_BOOTSTRAP`, `UMP_CURSOR_INDEX_METADATA_JSON`, and
  `UMP_CURSOR_INDEX_METADATA_FILE`
- Compaction knobs: `UMP_COMPACTION_KEYS_JSON` and `UMP_COMPACTION_INSTANCE_ID`

Do not commit live secrets. Local `.env` files are for humans only.

Provider auth file shape under `~/.ump/auth.json`:

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

`google.api_key` is accepted as a compatibility alias for `gemini.api_key`.
Codex OAuth remains in `~/.codex/auth.json`. Cursor auth is resolved separately
from `CURSOR_ACCESS_TOKEN`, system secret stores, or `<UMP_V2_AUTH_HOME>/.cursor/auth.json`.

## Hot Routing

Hot routes live in `UMP_V2_CONFIG` and are applied to the next request without a
restart:

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

The config UI and API are local admin surfaces. They require loopback Host
values, reject cross-site unsafe browser writes, return `Cache-Control: no-store`,
and serve same-origin CSP-protected assets.

## Responses WebSocket Contract

`GET /v1/responses` and `GET /api/provider/openai/v1/responses` accept RFC 6455
upgrades. The first client frame can be raw Responses JSON or a `response.create`
event. Each `response.create` resolves its own `format: "responses"` route; after
a terminal event, the same downstream socket may send another independent
`response.create` for a different provider or model.

While a response is in flight, another `response.create` is rejected with
`response_already_in_flight`. `previous_response_id` is connection-local and must
match the prior route/model fingerprint; it never authorizes a cross-provider
continuation. Codex targets use upstream Responses WSS, while Bedrock, Google,
and Cursor targets use provider-specific bridges and normalized downstream events.

## Codex CLI

Codex only accepts Responses API providers, so point `~/.codex/config.toml` at
the proxy's `/v1` base URL. Use an absolute path for `model_catalog_json`.

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
model_catalog_json = "/Users/<you>/.codex/model-catalog-ump-v2.json"
model_reasoning_effort = "high"
```

Keep `remote_compaction_v2 = false` for mixed-provider profiles until UMP owns
provider-aware compaction end to end. A Codex-only profile may enable native
remote compaction when every routed model stays in the Codex/OpenAI family.

Useful model choices:

- `gpt-5.5` / `openai:gpt-5.5` routes through Codex OAuth.
- `claude-sonnet-4-6` and `claude-sonnet-4-6-max` route through Bedrock.
- `gemini-3.1-flash-lite` routes through Google GenerateContent.
- `composer-2-fast` routes through the Cursor AgentService bridge.

## Development and Validation

Primary checks:

```sh
cargo fmt --check
cargo check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
```

Harness cleanup/contract checks:

```sh
scripts/gc/run-all.sh
```

Current local-development caveat: `Cargo.toml` depends on the local
`/Users/jaredboynton/__devlocal/specter` checkout. A fresh remote clone needs
that dependency made portable or mirrored locally before hosted CI can be treated
as authoritative.

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

For active development, prefer `cargo run`; launchd is for exercising the same
binary shape local agent clients use.
