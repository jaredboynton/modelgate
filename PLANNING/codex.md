# Codex Responses vs. Public OpenAI Responses

Reference notes for v2 adapter work. Captures every observed wire-level
difference between the public `/v1/responses` endpoint and the
ChatGPT-internal `codex/responses` endpoint, based on the v1 codebase at
`unified-model-proxy/src/lib/adapters/{openai-responses.ts,codex-responses.ts}`
and verified against the published OpenAI docs + `codex-rs/core/client.rs`.

In v2, the OpenAI-shaped routes are an OpenAI-compatible facade. They do not
promise full public OpenAI API parity, and they do not send every OpenAI-shaped
request to Codex. Route handlers first resolve the requested model and route
format to an explicit provider disposition. Codex-backed GPT Responses/chat
flows use Codex OAuth from `~/.codex/auth.json`; public OpenAI API-key
transport is a future `public-openai` provider and must stay
disabled-by-default rather than acting as a fallback.

## Endpoint URL

| | Public OpenAI | Codex |
|---|---|---|
| WebSocket | `wss://api.openai.com/v1/responses` | `wss://chatgpt.com/backend-api/codex/responses` |
| HTTP | `https://api.openai.com/v1/responses` | `https://chatgpt.com/backend-api/codex/responses` |

Codex talks to the ChatGPT backend host directly. There is no `v1` prefix
on the Codex path. The Codex compaction sibling is
`chatgpt.com/backend-api/codex/responses/compact`, mirroring the public
`/v1/responses/compact` endpoint.

## Authentication

### Public OpenAI
- Bearer token from `OPENAI_API_KEY` env (or platform key resolver).
- Single header: `Authorization: Bearer sk-...`.

### Codex
- OAuth tokens persisted at `~/.codex/auth.json` with three fields:
  `access_token`, `refresh_token`, `account_id`.
- Adapter reads via `readCodexAuthWithRefresh()`, which transparently
  refreshes if `last_refresh` is stale.
- Mid-stream `401` triggers refresh + retry once. After two failures the
  adapter surfaces the auth error to the caller.
- Refresh hits the standard OAuth token endpoint with
  `client_id=app_EMoamEEZ73f0CkXaXp7hrann` and
  `grant_type=refresh_token`.

## Request Headers

Public OpenAI adapter:

```
Authorization: Bearer <key>
OpenAI-Beta: responses_websockets=2026-02-06
```

Codex adapter:

```
Authorization: Bearer <access_token>
ChatGPT-Account-Id: <account_id>
originator: codex_cli_rs
OpenAI-Beta: responses_websockets=2026-02-06
```

`ChatGPT-Account-Id` and `originator` are mandatory. The backend uses
`originator` for telemetry / quota bucketing; missing or wrong values can
quietly route the request into a slower or capped lane. Real Codex CLI
sends `codex_cli_rs`, which is the value we copy.

## Body Differences

Both adapters reuse the same `convertToResponsesAPI()` transform that maps
OpenAI chat-compat to Responses-API input items. The Codex adapter then
applies a second pass of mutations on top.

### Shared transform (`convertToResponsesAPI`)

- System messages get pulled out of the `messages` array and joined into a
  single top-level `instructions` string.
- Remaining messages are emitted as canonical input items:

  ```json
  { "type": "message",
    "role": "user|assistant",
    "content": [{ "type": "input_text" | "output_text", "text": "..." }] }
  ```

  The plain `{role, content: "string"}` shape can be accepted over HTTP
  but the WebSocket endpoint silently stalls on it (no
  `response.completed` ever arrives).

- Tool definitions are flattened from Chat Completions shape to Responses
  shape:

  ```
  chat:    { type: "function", function: { name, description, parameters } }
  resp:    { type: "function", name, description, parameters }
  ```

  Forwarding the nested chat shape causes
  `[tools[0].name] [missing_required_parameter]`.

- `tool_choice` is similarly flattened: `{type: "function", name: "..."}`
  with no nested `.function`.

- `reasoning_effort` (chat-compat field) maps to
  `reasoning: { effort, summary: "auto" }`.

- `service_tier` defaults to `"priority"` on both adapters (Postman
  ChatGPT Business plan gets Priority Processing per pi-mono#3188).

### Shared transport prep (`prepareWsBody`)

Drops or defaults transport-only fields:

| Field | Behavior |
|---|---|
| `stream` | Dropped (WS is always streaming). |
| `stream_options` | Dropped (WS endpoint doesn't understand it). |
| `background` | Dropped. |
| `instructions` | Defaulted to `"You are a helpful assistant."` if absent. |
| `store` | OAuth path defaults to `false`. Public API stays `true`. |
| `max_output_tokens` | OAuth path strips it. Codex backend rejects it over WS. |
| `max_tokens` | OAuth path strips it (same reason). |

### Codex-only mutations

Applied after `prepareWsBody(body, isOAuth=true)`:

1. **`reasoning` is force-defaulted.**
   ```ts
   if (!responsesBody.reasoning) {
     responsesBody.reasoning = { effort: "medium", summary: "auto" };
   } else if (!responsesBody.reasoning.summary) {
     responsesBody.reasoning.summary = "auto";
   }
   ```
   Matches `codex-rs/core/client.rs` behavior. The backend wants a
   reasoning block whether or not the client cares.

2. **`include` must contain `reasoning.encrypted_content`.**
   ```ts
   if (!existingInclude.includes("reasoning.encrypted_content")) {
     responsesBody.include = [...existingInclude, "reasoning.encrypted_content"];
   }
   ```
   Without this opt-in the Codex backend streams text only — zero
   reasoning frames hit the wire. This is the single biggest non-obvious
   requirement; getting it wrong looks like "model just isn't thinking."

## Event Stream

Same event parser (`convertResponseEventsToOpenAIChunks`) handles both
endpoints. Frame types observed:

- `response.output_text.delta` -> chat `delta.content`
- `response.reasoning_summary_text.delta` -> chat `delta.reasoning_content`
- `response.reasoning_text.delta` -> chat `delta.reasoning_content`
- `response.output_item.added` (with `item.type === "function_call"`) ->
  first tool-call chunk with `id`, `name`, `arguments`.
- `response.function_call_arguments.delta` -> append-only
  `function.arguments` deltas, looked up by `item_id`. Frame carries no
  name/call_id.
- `response.function_call_arguments.done` -> ignored; redundant with the
  item.done state.
- `response.completed` -> final chunk with `finish_reason=stop` or
  `tool_calls`, plus usage rollup.
- `error` / `response.failed` -> raised as `<errorPrefix>: <message>`.

## Compaction

Both endpoints have a sibling compaction route:

- Public: `POST https://api.openai.com/v1/responses/compact`
- Codex: `POST https://chatgpt.com/backend-api/codex/responses/compact`

Behavior is the same: send a full context window of input items, get back
a smaller window containing a `type=compaction` item with opaque
`encrypted_content`. Drop-in replacement for the original transcript on
the next `/responses` call. ZDR-friendly with `store: false`.

Known gotchas to handle in v2:

- Returns `context_length_exceeded` if the input itself doesn't fit. Trim
  trailing tool calls / tool outputs (items the model can reproduce)
  before retrying.
- Not all models supported. `gpt-5.5` shipped without compact support,
  failing Codex auto-compaction. Either keep a model allowlist for
  compact or fall back to local summarization on the unsupported-model
  error.
- The returned `encrypted_content` blob is large. Cap its token estimate
  (the v1 Codex bug capped it at 10% of effective context window, max
  25k tokens) so the UI doesn't show "0% context left" right after
  compaction.

## Catalog and Route Disposition

- `/api/provider/openai/v1/models` uses the live Codex catalog endpoint
  `https://chatgpt.com/backend-api/codex/models` with Codex OAuth headers.
- `/v1/models` is the aggregate catalog: local static allowlist plus
  configured hot-route models.
- `/api/provider/openai/v1/responses` and
  `/api/provider/openai/v1/chat/completions` are OpenAI-shaped facade
  entrypoints. Codex-backed GPT dispositions use Codex OAuth, while other
  models remain governed by explicit route/provider rules.
- `/v1/responses` and `/v1/chat/completions` are aggregate facade routes; they
  may route to Codex, Bedrock, or Google depending on model disposition.
- Public-only OpenAI features, including routes without Codex scopes or adapter
  semantics, fail closed until they have an explicit provider and tests.

## Realtime and Audio Facade Status

`/v1/realtime`, `/v1/realtime/transcription_sessions`, and
`/v1/audio/transcriptions` are the only public OpenAI realtime/audio surfaces
currently allowed to attempt Codex OAuth bearer forwarding. The upstream base is
hardcoded to `https://api.openai.com` for HTTP and converted to
`wss://api.openai.com` for realtime WebSocket. This is a known limitation, not a
general `public-openai` provider: alternate public OpenAI base URLs, API-key
auth, and non-Codex credentials remain out of scope until a separately
configured provider exists.

Live validation for those routes is `live-blocked` by default. The registered
live smoke tests stay ignored unless `UMP_V2_LIVE_CODEX_REALTIME_AUDIO=1`,
local Codex OAuth auth, and any required audio fixture env are present. Normal
CI should rely on the local contract tests and must not claim full public
OpenAI realtime/audio parity.

## Quick V2 Checklist

When wiring the Codex adapter in v2:

- [x] Talk to `wss://chatgpt.com/backend-api/codex/responses`, not the
      public `api.openai.com` endpoint.
- [x] Send `ChatGPT-Account-Id` and `originator: codex_cli_rs` headers.
- [x] Implement OAuth file read + atomic write + 401 refresh-and-retry
      loop. Two attempts max.
- [x] After `convertToResponsesAPI` + `prepareWsBody(_, isOAuth=true)`:
  - inject `reasoning: { effort: "medium", summary: "auto" }` defaults
  - ensure `include` contains `reasoning.encrypted_content`
- [x] Confirm `store=false` and reject `max_output_tokens`/`max_tokens`
      before Codex dispatch.
- [x] Use the shared event parser; do not fork it for Codex.
- [ ] Hook compaction to the Codex sibling route, not the public one,
      when running on Codex auth.
