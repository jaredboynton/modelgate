# Codex App / Codex Responses Adapter Notes

**Note: For testing purposes, codex auth always resides under ~/.codex/auth.json**

## Purpose

Use this note when building or reviewing an adapter between:

- Codex App task or app-server payloads and the public OpenAI `/v1/responses` API.
- Public Responses-shaped requests and the ChatGPT Codex backend responses surface used by Codex/UMP.

The image-input question was the starting point, but the larger bridge is not just media. State, tools, files, streaming events, permissions, and response identity all need explicit adapter policy.

## Bottom Line

Codex App does not submit image input directly from the renderer to the public Responses endpoint. It sends Codex task or local-conversation payloads through internal Codex surfaces, and those payloads need translation before they match public Responses API request items.

The reverse direction is also not a clean public Responses pass-through. Current UMP Codex routing rewrites a public-looking Responses body, wraps it for the ChatGPT Codex backend, removes or defaults several fields, and normalizes backend events. Treat this as a lossy adapter unless every field and event below is handled deliberately.

Codex OAuth is broader than the Codex backend, but not equivalent to a normal public OpenAI API key. Fresh probes showed it can authenticate several ChatGPT product surfaces and public realtime/STT auth gates; the same bearer still lacks public `/v1/responses`, public `/v1/models`, and public TTS scopes. Embeddings are a separate public API surface: they are not listed in the Codex model catalog and are rejected on Codex backend `/codex/responses`, but `POST https://api.openai.com/v1/embeddings` accepted the Codex OAuth bearer for at least `text-embedding-3-small` and `text-embedding-3-large` in a 2026-05-19 live probe.

## Validation Standard

Treat the Codex-specific behavior in this note as live/API-and-code validated, not documentation-derived. The current grounding is:

- Live `curl`/WebSocket probes with the local `~/.codex/auth.json` ChatGPT OAuth bearer, with tokens redacted from logs.
- Real shipped Codex App bundle inspection under `/Applications/Codex.app`, including extracted JS assets and packed `app.asar` paths.
- Real `openai-codex` and `unified-model-proxy-v2` source inspection under `${AMP_RESEARCH_ROOT}`.

Public OpenAI documentation links remain below as secondary reference breadcrumbs for route names and public API concepts; they are not the primary evidence for Codex App, ChatGPT Codex backend, UMP, or Codex OAuth behavior.

## App Bundle OAuth Client IDs

Local bundle pass on 2026-05-18. Treat these as app-bundle constants only; a client ID appearing in a shipped app does not prove that Auth0 will allow arbitrary public API scopes such as `api.responses.write`. The expanded Codex OAuth request that added only `api.responses.write` still failed before callback.

| App bundle | Bundle ID / version | OAuth client IDs observed | Evidence / context |
|---|---|---|---|
| `/Applications/Codex.app` | `com.openai.codex` / `26.506.31421` | Primary Codex OAuth: `app_EMoamEEZ73f0CkXaXp7hrann` | `/Applications/Codex.app/Contents/Resources/app.asar` contains the ID next to `https://auth.openai.com`, port `1455`, `/auth/callback`, PKCE, and `codex.remote_control.enroll`; `/Applications/Codex.app/Contents/Resources/codex` also contains the same ID next to token refresh constants. |
| `/Applications/ChatGPT.app` | `com.openai.chat` / `1.2026.118` | Likely account-login client: `app_LlGpXReQgckcGGUo2JrYvtJK`; first-party / Sign in with Apple / shared-device clients: `app_WXrF1LSkiTtfYqiL6XtjygvX`, `app_EshkfRrR0legqtFbIYer1PVN` | `/Applications/ChatGPT.app/Contents/Frameworks/ChatGPT.framework/Versions/A/ChatGPT` contains `app_LlGpXReQgckcGGUo2JrYvtJK` next to `https://auth.openai.com/api/accounts/authorize`, `https://auth.openai.com`, `https://api.openai.com/v1`, and `com.openai.chat://auth0.openai.com/ios/com.openai.chat/callback`; the other two IDs appear near `https://auth.openai.com/api/first_party_authorize/next` and Sign in with Apple/shared-device strings. |
| `/Applications/ChatGPT Atlas.app` | `com.openai.atlas` / `1.2026.119.1` | Likely account-login client: `app_EshkfRrR0legqtFbIYer1PVN`; first-party / Sign in with Apple / shared-device clients: `app_WXrF1LSkiTtfYqiL6XtjygvX`, `app_LlGpXReQgckcGGUo2JrYvtJK` | `/Applications/ChatGPT Atlas.app/Contents/Frameworks/Aura.framework/Versions/A/Aura` and `/Applications/ChatGPT Atlas.app/Contents/Library/AtlasUpdateHelper` contain `app_EshkfRrR0legqtFbIYer1PVN` near `https://auth.openai.com/api/accounts/authorize`, `https://auth.openai.com`, and `https://api.openai.com/v1`; `app_WXrF1LSkiTtfYqiL6XtjygvX` and `app_LlGpXReQgckcGGUo2JrYvtJK` appear near `https://auth.openai.com/api/first_party_authorize/next` and Sign in with Apple/shared-device strings. |

Only candidates found in auth-adjacent bundle strings are listed here. Connector IDs, telemetry strings, and Rust/Swift symbol fragments that merely matched `app_` were ignored.

## Proxy Implementation Contract

Use this section as the implementation spec for a Responses-to-Codex proxy. A correct proxy must implement each row explicitly; unsupported rows should fail closed with a useful error instead of silently passing through.

### Required Routes

| Client-facing route | Upstream / internal route | Required behavior |
|---|---|---|
| `POST /v1/responses` | `POST https://chatgpt.com/backend-api/codex/responses` | Rewrite request into the Codex backend create shape described in "Current UMP Codex Responses Behavior"; apply the field and stream maps below; fail closed when no mapping exists. |
| `POST /v1/responses` with `{ "type": "compaction_trigger" }` in `input` | `POST https://chatgpt.com/backend-api/codex/responses` | Preserve the live Codex compaction trigger and returned `{ "type": "compaction", "encrypted_content": "..." }` item. Do not send `context_compaction` upstream; the live backend rejects it. |
| `POST /v1/responses/compact` | `POST https://chatgpt.com/backend-api/codex/responses/compact` | Forward a compact request body with `model`, list-shaped `input`, `instructions`, tools, reasoning, and prompt-cache metadata. The live backend returns preserved user messages plus `{ "type": "compaction_summary", "encrypted_content": "..." }`; canonicalize that alias to `{ "type": "compaction", ... }` before storing or replaying it. |
| Realtime-style Responses WebSocket | `wss://chatgpt.com/backend-api/codex/responses` for Codex responses; `wss://api.openai.com/v1/realtime` for Realtime API | Do not mix these protocols. Codex responses WebSocket uses `response.create` messages; Realtime uses `session.update`, `conversation.item.create`, and `response.create`. |
| `GET /v1/models` for Codex-backed clients | `GET https://chatgpt.com/backend-api/codex/models?client_version=<client_version>` | Translate Codex `models` catalog into an OpenAI-compatible model list or expose a Codex-specific catalog endpoint. `client_version` is required; omitting it returned `400`. |
| Public file/image inputs | Public Files API or Codex/ChatGPT file upload + pointer resolution | Public `file_id` and Codex `image_asset_pointer` / `sediment://` IDs are not interchangeable. Resolve or re-upload bytes. |
| Dictation recording | `/transcribe` in the app bundle, or public speech-to-text/transcriptions | Treat as a separate audio transcription route. Do not fold it into `/v1/responses`. |
| Realtime audio/text | Public `wss://api.openai.com/v1/realtime`, public `POST https://api.openai.com/v1/realtime/calls`, or local app-server `thread/realtime/*` | Keep as a separate realtime control plane. `gpt-realtime-2` uses GA fields/events; old beta fields differ. The `/v1/realtime/calls` route is WebRTC SDP offer/answer setup, not a Responses create call. |
| TTS | Public `/v1/audio/speech` with normal API-key auth | Codex OAuth bearer was rejected on tested TTS paths; do not advertise TTS through Codex bearer unless a separate auth path is implemented. |
| Embeddings | `POST https://api.openai.com/v1/embeddings` | Not a Codex backend or Codex catalog surface. The Codex OAuth bearer accepted `text-embedding-3-small` and `text-embedding-3-large` in a 2026-05-19 live probe, but rejected embedding models on `/backend-api/codex/responses` and did not expose embedding IDs through `GET /v1/models`. Keep embeddings on an explicit public-API route; do not fold them into `/v1/responses`. |
| ChatGPT web chat | `POST https://chatgpt.com/backend-api/f/conversation/prepare`, Sentinel prepare/finalize, then `/backend-api/f/conversation` | Supported by the current Codex OAuth bearer in a basic live probe, but this is a ChatGPT-web adapter surface, not the Codex backend or public Responses. Keep it behind an explicit `chatgpt-web` provider/route. |
| ChatGPT voice WebRTC | `POST https://chatgpt.com/realtime/vp?dcid=0` | Supported by the current Codex OAuth bearer with the local account header and HAR-shaped multipart text fields `sdp` and `session`. Treat as ChatGPT product voice, not public Realtime or Responses. |
| ChatGPT Celsius handoff | `GET https://chatgpt.com/backend-api/celsius/ws/user`, then `wss://ws.chatgpt.com/...` | Supported by the current Codex OAuth bearer for WSS bootstrap metadata. Redact WSS path/query and keep it separate from public Realtime. |
| Local app-server JSON-RPC | `ws://codex-app-server/rpc`, UDS `ws://localhost/rpc`, or configured app-server websocket | This is a local control-plane API, not a public Responses endpoint. Use only for local Codex App/Core integration. |

### Public Route Disposition

Public Responses-compatible route families considered for this proxy include create, retrieve, delete, cancel, list input items, count input tokens, compact, and Conversations item management. A proxy must expose one of these dispositions for each route:

| Public route | Proxy disposition |
|---|---|
| `POST /v1/responses` | Implement through Codex backend response creation with the field policy below. |
| `GET /v1/responses/{response_id}` | Implement only from the adapter's public retrievable response store. Return `404` when the request used `store: false`, even if the adapter kept volatile continuation state. Do not call Codex `turn/read` unless the ID map proves the thread/turn relation. |
| `DELETE /v1/responses/{response_id}` | Delete adapter-owned public retrievable response state and return an OpenAI-shaped deleted object. This does not delete Codex rollout history unless the app-server path is explicitly configured to do so. |
| `POST /v1/responses/{response_id}/cancel` | Support only for adapter-owned in-flight/background work; map app-server sessions to `turn/interrupt` when available. Return `409` for completed responses and `501` for Codex backend requests the proxy cannot cancel. |
| `GET /v1/responses/{response_id}/input_items` | Implement only from the public retrievable input-item store with `after`, `limit`, and `order` pagination. Return `404` for `store: false` responses even when volatile continuation state exists. |
| `POST /v1/responses/input_tokens` | Return `501` unless the proxy has a model-matched tokenizer/counting backend. Do not estimate with a different model silently. |
| `POST /v1/responses/compact` | Implement for Codex-backed models by forwarding to the Codex backend compact route. Return the compacted output as opaque compacted items, canonicalizing `compaction_summary` to `compaction` for adapter storage/replay. |
| `POST /v1/conversations` | Optional adapter-state feature. If implemented, create `conv_*` IDs and store initial items; no Codex backend call is required until a response uses the conversation. |
| `GET/PATCH/DELETE /v1/conversations/{conversation_id}` | Optional adapter-state feature. Return `501` if conversations are disabled, `404` if unknown, and never map directly to a Codex thread without an explicit ID map. |
| `GET/POST /v1/conversations/{conversation_id}/items` | Optional adapter-state feature. Preserve item order and IDs; only project into Codex history on the next response create or via `thread/inject_items` when using app-server integration. |
| `GET/DELETE /v1/conversations/{conversation_id}/items/{item_id}` | Optional adapter-state feature. Deleting a public conversation item must not mutate Codex rollout history unless the proxy owns that history projection. |

### Required State

- Maintain a mapping between public `response.id` / `previous_response_id` and Codex thread or backend response state.
- Separate volatile continuation state from public retrievable storage. Volatile state may support `previous_response_id`, reasoning continuity, stream assembly, or debugging when `store: false`, but it must not make `GET /v1/responses/{id}` or `/input_items` publicly retrievable.
- Preserve raw upstream response items and events when the public shape cannot be represented by Codex UI/core types.
- Track the selected upstream Codex model, reasoning effort, service tier, and model catalog ETag per conversation when clients expect stable continuation.
- Record whether a request used Codex backend responses, public Realtime, app-server JSON-RPC, dictation, or a rejected unsupported path.

### Required Failure Policy

- Reject unsupported public fields with an adapter error unless a loss is explicitly documented in this file.
- Reject unknown or unavailable model IDs instead of silently falling back.
- Reject public hosted tools that have no Codex equivalent unless a route-specific mapping exists.
- Reject public `input_file` unless the proxy can resolve or upload the file and preserve provenance.
- Never let a public JSON payload supply a trusted local filesystem path.

### Create Field Policy

Unknown request fields must be rejected with `unsupported_field`. Known fields use the following policy:

| Public create field | Policy for Codex backend proxy |
|---|---|
| `model` | Required. Resolve aliases, validate against the live Codex catalog, and reject unavailable/hidden models unless explicitly enabled. |
| `input` | Map `input_text` and data-URL / resolvable `input_image`; reject `input_file` unless file resolution/upload is implemented; preserve raw input in adapter state. For Codex compaction, pass live `{ "type": "compaction_trigger" }` triggers and replay opaque `{ "type": "compaction", "encrypted_content": "..." }` items. Do not pass `context_compaction` upstream. |
| `instructions` | Pass or default to `"You are a helpful assistant."` only when the caller omitted it; do not merge public system/developer history into user input. |
| `previous_response_id` | Resolve through adapter state. Reject if unknown or if used together with a public `conversation` value that the proxy cannot reconcile. |
| `conversation` | Use adapter-owned conversation state if implemented; otherwise reject with `unsupported_field`. |
| `store` | Codex backend routing forces or records upstream `store: false`. For caller-visible `store: false`, keep only volatile continuation state and return `404` from public retrieve/input-item routes. For caller-visible `store: true`, implement adapter-owned public storage or reject with `unsupported_field`; do not promise upstream public storage. |
| `metadata` | Store in adapter state; pass upstream only if confirmed harmless for the selected Codex route. |
| `background` | Reject for Codex backend unless the proxy implements a background job queue plus retrieve/cancel semantics. Current UMP strips it, so silent pass-through is forbidden. |
| `max_output_tokens` / `max_tokens` | Reject or strip with a warning only if the caller opted into lossy compatibility. Current UMP strips them; a strict proxy should reject. |
| `max_tool_calls` | Enforce in adapter-owned tool orchestration if implemented; otherwise reject. |
| `parallel_tool_calls` | Pass only when the selected Codex model/tool path supports it; otherwise reject. |
| `service_tier` | Validate against the model catalog `service_tiers`; pass only supported values. |
| `reasoning` | Default to `{ "effort": "medium", "summary": "auto" }` when absent; validate effort against `supported_reasoning_levels`; ensure `reasoning.encrypted_content` is included for continuity. |
| `include` | Preserve supported values and always include `reasoning.encrypted_content`; reject hosted-tool includes the proxy cannot populate. |
| `text` | Map `text.format.type = json_schema` when supported by Codex structured-output paths; reject legacy `json_object` or unsupported verbosity/format options. |
| `tools` | Map supported function/custom/web-search/image-generation tool specs. Validate exact public `tools[]` schema per capability; reject MCP, file search, tool search, code interpreter, computer use, shell/local shell, hosted shell skill bundles, and apply patch unless a route-specific bridge exists. |
| `tool_choice` | Pass `none`, `auto`, and supported named tool choices only when the chosen tool is mapped; reject forced choices for unsupported tools. |
| `stream` | Implement SSE streaming when `true`; otherwise return a complete response. Do not confuse SSE with Realtime or the Codex responses WebSocket protocol. |
| `stream_options` | Support `include_obfuscation` only if the proxy emits obfuscation; otherwise reject or strip with warning under lossy mode. |
| `temperature` / `top_p` | Pass only if the Codex backend accepts the value for the selected model; otherwise reject instead of silently changing sampling. |
| `top_logprobs` | Reject unless the selected Codex backend returns compatible logprob events/items. |
| `truncation` | Validate against the model catalog truncation policy; reject unsupported public values. |
| `prompt` | Reject unless the proxy can resolve public prompt-template IDs and substitute variables before upstream submission. |
| `prompt_cache_key` / `prompt_cache_retention` | Pass only if mapped to Codex request assembly; otherwise reject because cache semantics affect billing/state. |
| `context_management` | Reject unless the proxy implements the exact public context behavior for the selected route. |
| `safety_identifier` | Store or pass only through a documented trust boundary; otherwise reject. |
| `user` | Treat as legacy/client metadata only if needed; otherwise reject to avoid implying public OpenAI usage attribution. |

### Stream Event Mapping

SSE clients must receive deterministic event behavior. The proxy should keep raw upstream events in state even when the public stream projection is narrower.

Guidewire frontend clients must consume `/v1/responses` as incremental SSE. Calling `Response.json()` on this route is banned because it blocks browser rendering until the full-envelope POST completes; this was the G4 regression.

| Public stream event / field family | Proxy behavior |
|---|---|
| `response.created`, `response.in_progress` | Pass through if upstream emits them; otherwise synthesize from adapter state before first output delta. |
| `response.output_item.added`, `response.output_item.done` | Map from Codex response items when item IDs are available; otherwise synthesize stable adapter item IDs and preserve raw upstream item JSON. |
| Compaction output items | Preserve streamed `{ "type": "compaction", "encrypted_content": "..." }` items as opaque context packs. They are valid follow-up input only with `encrypted_content` present. |
| `response.content_part.added`, `response.content_part.done` | Map text content parts; preserve unsupported content-part JSON in raw state and reject if the client requested strict public parity. |
| `response.output_text.delta`, `response.output_text.done` | Map from Codex output-text deltas/done events; preserve ordering with adapter-owned `output_index`, `content_index`, and `item_id` maps. |
| `response.function_call_arguments.delta`, `response.function_call_arguments.done` | Map only for supported function tools. Codex custom/freeform deltas are not automatically public function-call deltas; reject unsupported tool streams. |
| `response.custom_tool_call_input.delta` | Preserve or map only to a configured public custom-tool bridge; never relabel as function-call arguments without schema compatibility. |
| `response.refusal.*`, annotations, citations, and logprobs | Pass only when the Codex backend returns compatible public fields; otherwise preserve raw and reject strict clients that requested those `include` values. |
| `response.incomplete`, `response.failed` | Emit public-shaped failure/incomplete events with the upstream error/status preserved in `error` or adapter metadata; do not collapse them into plain text. |
| `response.completed` | Emit exactly once after all mapped items are complete; include the final response object from adapter state. |
| Unknown `response.*` events | If already public-shaped, pass through and store raw; otherwise emit `adapter.unsupported_event` only under a documented extension mode, or terminate strict streams with `502 upstream_unsupported_event`. |
| Ordering IDs | Maintain adapter maps for upstream item IDs, `output_index`, `content_index`, and tool-call IDs. Interleaved streams are invalid unless those maps are present. |

### Adapter Error Contract

Use an OpenAI-compatible error envelope for all proxy-generated errors:

```json
{
  "error": {
    "message": "Human-readable adapter failure.",
    "type": "invalid_request_error",
    "param": "input[0].content[1]",
    "code": "unsupported_input_file"
  }
}
```

Status/code policy:

| Condition | HTTP status | `error.type` | `error.code` |
|---|---:|---|---|
| Unsupported route or feature | `501` | `invalid_request_error` | `unsupported_route` / `unsupported_feature` |
| Unsupported field or tool | `400` | `invalid_request_error` | `unsupported_field` / `unsupported_tool` |
| Unknown or hidden model | `400` | `invalid_request_error` | `model_not_supported` |
| Unknown adapter `response_id` / `conversation_id` | `404` | `invalid_request_error` | `not_found` |
| Invalid state transition, such as canceling a completed response | `409` | `invalid_request_error` | `invalid_state` |
| Missing or invalid inbound proxy auth | `401` | `authentication_error` | `invalid_api_key` |
| Codex bearer lacks entitlement or upstream forbids the path | `403` | `permission_error` | `upstream_forbidden` |
| Upstream Codex/ChatGPT failure | `502` | `api_error` | `upstream_error` |
| Upstream timeout or rate limiting | `503` / `429` | `api_error` / `rate_limit_error` | `upstream_unavailable` / `rate_limit_exceeded` |

### Minimum Acceptance Tests

Use these as the implementation test plan for a proxy. Passing all rows is the minimum bar for calling the proxy compatible with this spec.

| Area | Required check |
|---|---|
| Model catalog | `GET /v1/models` with a Codex-backed provider fetches `https://chatgpt.com/backend-api/codex/models?client_version=<current>`; maps visible Codex models to OpenAI-compatible list entries; hides `codex-auto-review` by default; and returns a clear upstream/configuration error when `client_version` is absent or rejected. |
| Model validation | A request for an unknown model is rejected before upstream submission; a request for a known model preserves the requested model unless an explicit alias resolves it. |
| Model capabilities | `input_modalities`, reasoning levels, verbosity, service tiers, truncation policy, and image-detail support are enforced from the Codex catalog rather than guessed from public OpenAI model naming. |
| Text response | A simple public `POST /v1/responses` text request is rewritten to the Codex backend create shape, forces or records upstream `store: false`, includes `reasoning.encrypted_content`, and streams/returns ordered output text without making `store: false` responses publicly retrievable. |
| Streaming compaction | `POST /v1/responses` with a live `{ "type": "compaction_trigger" }` input item returns SSE containing a `{ "type": "compaction", "encrypted_content": "..." }` item; replaying retained user/developer/system messages plus that compacted item preserves hidden assistant context. |
| Compact endpoint | `POST /v1/responses/compact` forwards to the Codex backend compact route, receives preserved user messages plus `compaction_summary.encrypted_content`, canonicalizes the compacted item to `compaction`, and verifies the canonical item works in a follow-up `/v1/responses` request. |
| Image input | A public `input_image` data URL succeeds; a public local filesystem path fails closed; a Codex `image_asset_pointer` is resolved or re-uploaded before public use. |
| File input | Public `input_file` is rejected unless the proxy has implemented byte resolution/upload and provenance tracking. |
| State continuity | `previous_response_id` uses adapter-owned state and never assumes it is a Codex `turnId` or thread ID. |
| Streaming fidelity | Text deltas preserve ordering; unsupported public event fields are either mapped, preserved in raw event state, or rejected/documented. |
| Tools | Function tools map only when schema/call/result events are supported; hosted public tools without Codex equivalents fail closed. |
| Realtime | Text-only realtime routes to public Realtime or app-server `thread/realtime/*`, never to `/codex/responses`; `gpt-realtime-2` uses GA `response.output_modalities`. |
| Dictation | Completed recordings route through `/transcribe` or public speech-to-text, not normal `/v1/responses`. |
| TTS | `gpt-4o-mini-tts` is not advertised through Codex OAuth unless a separate valid auth/upstream path is implemented. |
| Auth boundaries | Codex OAuth is used only on tested Codex/ChatGPT or Realtime paths. Live probes accepted Codex backend `/codex/responses`, public Realtime WebSocket, public Realtime transcription-session creation, and public `/v1/embeddings` for `text-embedding-3-small` / `text-embedding-3-large`; public `/v1/responses`, `/v1/models`, and `/v1/audio/speech` rejected the same bearer with scope/permission errors. |

## Model Discovery and Selection

There are three distinct model surfaces:

1. **Remote Codex model catalog.** `GET https://chatgpt.com/backend-api/codex/models?client_version=<client_version>` returns a Codex-specific `models` array, not the public `/v1/models` object shape. The query parameter matters: with app version `26.506.31421` or bundled CLI version `0.130.0-alpha.5`, the live endpoint returned `gpt-5.5`, `gpt-5.4`, `gpt-5.4-mini`, `gpt-5.3-codex`, `gpt-5.2`, and hidden `codex-auto-review`; with old test `client_version=0.99.0`, `gpt-5.5` was omitted; without `client_version`, the endpoint returned `400`.
2. **App-server catalog RPC.** `model/list` exposes the supported model picker list from Codex model presets. It supports `cursor`, `limit`, and `include_hidden`; hide entries such as `codex-auto-review` unless an internal caller explicitly requests them.
3. **Provider capability RPC.** `modelProvider/capabilities/read` returns capability booleans with camelCase wire names: `namespaceTools`, `imageGeneration`, and `webSearch`. The Rust struct fields are `namespace_tools`, `image_generation`, and `web_search`, but the app-server protocol uses `#[serde(rename_all = "camelCase")]` and the generated TypeScript schema confirms the camelCase response. Use it to gate UI/features, not as a model list.

Model-bearing but non-catalog surfaces:

- `thread/start` and `thread/resume` carry `model`, `modelProvider`, `serviceTier`, config overrides, and reasoning effort.
- `thread/list` / thread metadata exposes `modelProvider`.
- `model/rerouted` and `model/verification` are runtime notifications, not request endpoints. Preserve them if a client needs to explain model changes.
- `model_catalog_json` is a startup/config catalog override; per-thread config overrides should not be treated as live catalog refresh.

Current conclusion: no additional Codex App/backend endpoint found so far exposes model IDs beyond the remote Codex model catalog, app-server `model/list`, provider-capability RPC, thread request/response fields, reroute/verification notifications, and config catalog override. Re-check this conclusion against the shipped app bundle and `openai-codex` source when upgrading the pinned Codex App version.

Model mapping rules:

- Use the remote Codex catalog as the allowlist for Codex backend `/codex/responses`.
- Apply each model's `default_reasoning_level`, `supported_reasoning_levels`, `support_verbosity`, `default_verbosity`, `service_tiers`, `input_modalities`, `supports_image_detail_original`, and `truncation_policy` when validating requests.
- Preserve upgrade metadata (`upgrade.model`, `migration_markdown`) for picker/UI warnings, but do not auto-upgrade a caller's requested model.
- Do not expose hidden models such as `codex-auto-review` to public clients by default.
- Public realtime models (`gpt-realtime-2`, `gpt-realtime`, `gpt-realtime-whisper`) and public TTS models (`gpt-4o-mini-tts`) are not Codex backend `/codex/responses` models. Reject them on the Codex backend route unless a live probe proves support. Public OpenAI-provider routing may still allow non-streaming `/v1/responses` for models such as `gpt-realtime-2`; live audio/text sessions belong on `/v1/realtime`.
- Embedding models (`text-embedding-3-small`, `text-embedding-3-large`, and similar) are not Codex backend `/codex/responses` models and do not appear in the remote Codex catalog. They belong on public `POST /v1/embeddings` when the bearer accepts that route; do not validate them against the Codex catalog allowlist.

## Public Responses Baseline

The public Responses-compatible surface this proxy needs to account for is wider than the Codex App/local protocol surface:

- Message content can include `input_text`, `input_image`, `input_file`, and audio-shaped inputs; file inputs can use public `file_id`, `file_url`, or `file_data`.
- Request state can use `conversation`, `previous_response_id`, `store`, `metadata`, `background`, `truncation`, `context_management`, `reasoning`, `prompt`, `prompt_cache_key`, `prompt_cache_retention`, `max_output_tokens`, `max_tool_calls`, `parallel_tool_calls`, `service_tier`, sampling fields, `top_logprobs`, `safety_identifier`, and `text.format`.
- Public tool-related capabilities include function/custom tools, web search, file search, tool search, remote MCP, hosted shell skill bundles, shell/local shell, computer use, image generation, apply patch, and code interpreter. These are not all identical `tools[{ type: ... }]` shapes; validate each tool's exact public schema before mapping or rejecting it.
- Streaming has item/index-rich `response.*` events, including function-call argument deltas, content-part/refusal/annotation events, incomplete states, and final completion events.

Secondary reference links, not primary evidence:

- Responses create API: `https://platform.openai.com/docs/api-reference/responses/create`
- Responses route index: `https://developers.openai.com/api/reference/overview`
- Responses compact/count/cancel/input-items routes: `https://platform.openai.com/docs/api-reference/responses/retrieve`
- Conversations API routes: `https://platform.openai.com/docs/api-reference/conversations/create`
- Streaming Responses: `https://platform.openai.com/docs/api-reference/responses-streaming/response`
- File inputs: `https://developers.openai.com/api/docs/guides/file-inputs`
- Images and vision: `https://developers.openai.com/api/docs/guides/images-vision`
- Background mode: `https://platform.openai.com/docs/guides/background`
- Tools overview: `https://developers.openai.com/api/docs/guides/tools`
- Shell tool: `https://developers.openai.com/api/docs/guides/tools-shell`
- Computer tool: `https://developers.openai.com/api/docs/guides/tools-computer-use`
- Remote MCP tool: `https://developers.openai.com/api/docs/guides/tools-connectors-mcp`
- Realtime transcription: `https://platform.openai.com/docs/guides/realtime-transcription`
- Realtime WebRTC: `https://platform.openai.com/docs/guides/realtime-webrtc`
- Realtime model catalog: `https://developers.openai.com/api/docs/models/gpt-realtime-2`
- Full model catalog: `https://developers.openai.com/api/docs/models/all`
- Text-to-speech model catalog: `https://developers.openai.com/api/docs/models/gpt-4o-mini-tts`
- Speech to text: `https://platform.openai.com/docs/guides/speech-to-text`
- Embeddings create API: `https://platform.openai.com/docs/api-reference/embeddings/create`

## Observed Codex App Image Input Flow

Research target: `/Applications/Codex.app`, specifically the packed `Contents/Resources/app.asar` bundle.

- Renderer intake lives in `webview/assets/composer-DawxvKsB.js`.
- Paste and drag/drop image files are read with `FileReader.readAsDataURL`.
- The composer stores image attachments with fields like `src`, `localPath`, `filename`, `uploadStatus`, and, for cloud mode, `pointer`.
- Local mode can preserve local filesystem images as `localImage` items when a trusted `localPath` is available.
- Cloud mode uploads image bytes before task creation:
  - `POST /files` creates an upload target.
  - The renderer uploads the base64 image bytes to the returned `upload_url`.
  - `POST /files/{file_id}/uploaded` finalizes the upload.
  - The final task image item is an internal `image_asset_pointer` with `asset_pointer`, `width`, `height`, and `size_bytes`.
- Cloud task creation is then a `POST /wham/tasks` request from `webview/assets/codex-api-B3jrGDqO.js`.
- The `/wham/tasks` body includes `input_items`, starting with a text message item:
  - `{ type: "message", role: "user", content: [{ content_type: "text", text: prompt }] }`
  - Image items are appended directly to the same `input_items` array.

## Public Responses Image Shape

The public Responses API accepts image input inside user message content:

```json
{
  "model": "gpt-4.1-mini",
  "input": [
    {
      "role": "user",
      "content": [
        { "type": "input_text", "text": "what is in this image?" },
        { "type": "input_image", "image_url": "data:image/png;base64,..." }
      ]
    }
  ]
}
```

It also supports file-backed images via `{ "type": "input_image", "file_id": "file_..." }` when the file is created through the public Files API with a vision-compatible purpose.

## Voice, Dictation, and Realtime Audio

There are two audio paths that are not covered by normal `/v1/responses` text/image/file translation:

1. **Global dictation in the app bundle.** `global-dictation-page-Bg1-uDEX.js` asks Electron for microphone permission, records `navigator.mediaDevices.getUserMedia({ audio: { channelCount: 1 } })` with `MediaRecorder`, then passes the recorded `Blob` to `use-recording-waveform-gMqiyEbV.js`.
2. **Realtime voice in Codex app-server/core.** The app-server protocol exposes experimental `thread/realtime/*` methods for starting a thread-scoped realtime session, appending audio/text, stopping, listing voices, and receiving transcript/audio notifications.

The dictation helper sends multipart audio to `/transcribe` with `X-Codex-Base64: 1`; after transcription, it optionally cleans up the transcript by streaming a small `/codex/responses` request with `tool_choice: "none"`, `store: false`, and `gpt-5.4-mini`.

The realtime path is a separate Realtime API bridge, not a Responses create call:

- App-server methods: `thread/realtime/start`, `thread/realtime/appendAudio`, `thread/realtime/appendText`, `thread/realtime/stop`, and `thread/realtime/listVoices`.
- App-server notifications: `thread/realtime/started`, `thread/realtime/transcript/delta`, `thread/realtime/transcript/done`, `thread/realtime/outputAudio/delta`, `thread/realtime/sdp`, `thread/realtime/error`, and `thread/realtime/closed`.
- Transport can be WebSocket or WebRTC SDP. The WebRTC helper creates a macOS peer connection, attaches a local microphone audio track, and applies the remote SDP answer.
- Realtime v2 session config uses `audio/pcm` at 24 kHz, `gpt-4o-mini-transcribe` for input transcription, optional server VAD, optional audio output, and function tools such as `background_agent` / `remain_silent`.

Adapter policy:

- Do not fold realtime audio into `/v1/responses` `input_file` or `input_audio`; preserve it as a separate realtime control plane.
- Document `/transcribe` as a completed-recording transcription path, distinct from realtime streaming transcription.
- If bridging to public OpenAI APIs, map dictation to Speech-to-text or audio transcription, and map live voice to Realtime WebSocket/WebRTC sessions.
- Preserve transcript delta/done events separately from regular Responses `response.output_text.*` deltas; they use different event names and lifecycle semantics.

### Two-track diarization

Speaker-tagged transcripts are a separate opt-in REST track, not a Realtime feature. The validated path is `POST /v1/audio/transcriptions` with `gpt-4o-transcribe-diarize`, `response_format=diarized_json`, `stream=true`, and bounded WAV windows. Realtime live captions keep using the existing `gpt-4o-mini-transcribe` input-transcription path.

Guidewire exposes this as `POST /diarize/stream`. The route captures system-audio windows for REST diarization, normalizes upstream `transcript.text.segment` / `transcript.text.done` SSE into local `DiarizeStreamEvent` records, and persists those records as transcript daemon events. Known-speaker matching is opt-in with one local `KnownSpeakerReference`; the sidecar validates `data:audio/wav;base64,...` input, sends the corresponding `known_speaker_names[]` / `known_speaker_references[]` multipart fields only to the REST route, and redacts data URLs from reports and errors.

Default behavior keeps microphone audio out of the diarize model. The mixed mono Realtime stream remains the assistant/live-caption input, while the speaker-tagged route only captures microphone windows when a caller explicitly enables the separate local microphone transcription worker. That worker uses `gpt-4o-mini-transcribe` and tags its segments with the fixed local speaker label; microphone audio is never sent to `gpt-4o-transcribe-diarize`. Do not claim Realtime diarization support unless a new live probe proves that Realtime accepts and emits speaker labels for a diarize model.

Text-only realtime probe:

- Public Realtime WebSocket accepts text input without microphone audio. A `websocat` probe against the current `gpt-realtime-2` model, using the local `~/.codex/auth.json` ChatGPT access token, opened a session, accepted `conversation.item.create` with `input_text`, accepted `response.create`, and streamed `response.output_text.delta` / `response.output_text.done` with `PONG`.
- Public Realtime WebRTC call setup also accepted the Codex OAuth bearer at `POST /v1/realtime/calls?model=gpt-realtime`: an intentionally minimal SDP offer returned `400 invalid_offer`, which proves auth passed and the body failed SDP validation. A complete browser/WebRTC SDP offer still needs a separate end-to-end call test.
- `gpt-realtime-2` is GA-only: sending the old `OpenAI-Beta: realtime=v1` header returned `invalid_model` with "only available on the GA API"; using the old beta `response.modalities` field returned `unknown_parameter`. For GA, use `response.output_modalities`.
- Older `gpt-realtime` / `gpt-realtime-2025-08-28` probes worked with the beta-shaped `response.modalities` field and emitted `response.text.delta` / `response.text.done`; do not reuse that event/field mapping blindly for `gpt-realtime-2`.
- `curl` with WebSocket upload support can open the same session, but it is fragile for this workflow because uploaded bytes are framed in a way that produced server errors in testing. Prefer a real WebSocket client (`websocat`, browser WebSocket, or a websocket library) for adapter tests.
- The direct Codex backend websocket URLs `wss://chatgpt.com/backend-api/codex/realtime?...` and `...?intent=quicksilver...` rejected the same OAuth bearer with `403 Forbidden`. Do not assume ChatGPT OAuth can directly drive the Codex backend realtime websocket.
- Codex core's websocket transport preparation still requires API-key auth for `Op::RealtimeConversation` WebSocket startup. The app-server `thread/realtime/appendText` method is therefore a local/app-server control-plane method, not proof that `/codex/responses` accepts realtime text.

Realtime transcription and TTS auth probes:

- `gpt-realtime-whisper` cannot be used as the top-level `/v1/realtime?model=...` session model. The websocket returned `invalid_model` and told the caller to pass it as `audio.input.transcription.model`.
- The Codex OAuth bearer did work for that websocket path when `gpt-realtime-whisper` was nested under a `gpt-realtime-2` `session.update`: the server returned `session.updated` with `audio.input.transcription.model: "gpt-realtime-whisper"`.
- A fresh live `curl` probe on 2026-05-19 showed `POST /v1/realtime/client_secrets` accepted the Codex OAuth bearer and returned `200` with a `realtime.transcription_session` payload when using GA `session.type: "transcription"` plus `audio.input.transcription.model: "gpt-realtime-whisper"`. The older `POST /v1/realtime/transcription_sessions` beta route now returns `400 beta_api_shape_disabled`.
- A fresh live `curl` probe on 2026-05-14 showed `POST /v1/audio/transcriptions` also accepted the Codex OAuth bearer: an intentionally empty upload returned `400 unsupported_value` for the file format, not an auth/scope error. A valid audio body still needs a separate positive STT-content test.
- `gpt-4o-mini-tts` is not accepted through this bearer on tested paths: `/v1/audio/speech` returned `401` with missing scope `api.model.audio.request`, `/v1/responses` returned `401` with missing scope `api.responses.write`, and `/v1/realtime?model=gpt-4o-mini-tts` returned `invalid_model` because TTS is not a realtime session model.

Public embeddings auth probes (2026-05-19):

- The remote Codex catalog at `GET https://chatgpt.com/backend-api/codex/models?client_version=0.130.0-alpha.5` returned six chat/codex IDs (`gpt-5.5`, `gpt-5.4`, `gpt-5.4-mini`, `gpt-5.3-codex`, `gpt-5.2`, `codex-auto-review`) and no embedding model IDs in a 2026-05-19 live probe.
- `POST https://chatgpt.com/backend-api/codex/responses` with `model: text-embedding-3-small` returned `400` with `The 'text-embedding-3-small' model is not supported when using Codex with a ChatGPT account.` Treat embedding models as out of scope for the Codex backend Responses adapter.
- Guessed ChatGPT backend embedding paths such as `/backend-api/codex/embeddings` and `/backend-api/embeddings` returned `403` in the same probe pass; do not assume a ChatGPT-product embedding route exists behind Codex OAuth.
- `GET https://api.openai.com/v1/models` with the Codex OAuth bearer returned `403` missing `api.model.read`, so embedding models cannot be discovered through the public models list with this token even when `/v1/embeddings` works.
- `POST https://api.openai.com/v1/embeddings` with the Codex OAuth bearer returned:
  - `200` for `text-embedding-3-small` (1536-dim vector in the probe response).
  - `200` for `text-embedding-3-large` (3072-dim vector in the probe response).
  - `401` for `text-embedding-ada-002` with missing scope `model.request`.
  - `403` for `gpt-5.4-mini` with `You are not allowed to generate embeddings from this model`.
- The decoded Codex OAuth token scopes on the probed account were `openid`, `profile`, `email`, `offline_access`, `api.connectors.read`, and `api.connectors.invoke`. There was no `model.request` scope, which likely explains why older embedding models such as `text-embedding-ada-002` failed while `text-embedding-3-*` succeeded.
- Adapter policy: keep embeddings on an explicit public `/v1/embeddings` route or provider. Do not expose them through Codex-backed `/v1/responses`, do not validate them against the Codex catalog, and do not assume every public embedding model ID works with Codex OAuth without a live model-by-model probe.

### ChatGPT Product Surfaces Authenticated by Codex OAuth

These are supported by the current local Codex OAuth bearer, but they are not Codex backend `/codex/responses` and should not be folded into public `/v1/responses`:

| Surface | Live result | Adapter policy |
|---|---|---|
| ChatGPT web conversation chain | End-to-end `/backend-api/f/conversation/prepare` plus Sentinel prepare/finalize plus `/backend-api/f/conversation` returned `200` SSE and the requested marker text using a temporary envelope with the Codex bearer and no real ChatGPT session cookie. | Use only for an explicit ChatGPT-web provider path. Validate model entitlements per ChatGPT slug. |
| ChatGPT web direct prepare shape probe | A deliberately malformed prepare body returned `422 Invalid conversation body`. | Interpret as auth accepted and schema rejected; do not treat it as a usable request shape. |
| ChatGPT Celsius bootstrap | `GET /backend-api/celsius/ws/user` returned `200` JSON containing a `websocket_url`. | Use for ChatGPT stream handoff only; redact WSS path/query and do not mix with public Realtime events. |
| ChatGPT voice WebRTC | `POST /realtime/vp?dcid=0` with HAR SDP/session multipart text fields returned `201`. | Treat as ChatGPT voice offer/answer setup; keep the local account header in memory and never log it. |

Focused follow-up on 2026-05-14 proved the Codex bearer can also drive an actual agentic ChatGPT-web Pro tool loop, not just a single text turn:

- `gpt-5-5-pro` accepted the full `/backend-api/f/conversation/prepare` -> Sentinel prepare/finalize -> `/backend-api/f/conversation` chain with no ChatGPT session cookie.
- For Pro, `conversation/prepare` can return `status: "ok"` with `conduit_token: null`; the subsequent conversation call still succeeds when `x-conduit-token` is omitted.
- A wrapper-prompted first turn produced a real assistant message with `recipient: "api_tool.call_tool"` and JSON content shaped like `{"path":"get_marker","args":{"name":"alpha"}}`.
- Posting a follow-up message with `author.role: "tool"`, `author.name: "api_tool.call_tool"`, `content.content_type: "execution_output"`, the same `conversation_id`, and `parent_message_id` set to the tool-call assistant message id returned `200 text/event-stream` and the assistant used the tool result marker.
- The ChatGPT-web adapter therefore needs to preserve the `{ conversation_id, tool_call_message_id }` pair in the client-visible tool-call ID. UMP does this with `call_cgw_*` IDs; Responses `function_call_output.call_id` must feed that same ID back on the next turn.

This is separate from the Codex backend `/backend-api/codex/responses` tool model. It should stay behind an explicit `chatgpt-web/*` provider route because it uses ChatGPT web `api_tool.call_tool`, Sentinel admission, and ChatGPT conversation parenting rather than Codex backend response state.

Archived evidence for the GPT-5.4/GPT-5.5 endpoint matrix, ChatGPT-web Pro no-conduit behavior, and Pro tool-loop replay lives under `docs/gpt-web-evidence/`; start with `docs/gpt-web-evidence/INDEX.md`.

Additional Pro capability validation in `docs/gpt-web-evidence/chatgpt_web_pro_search_research_probe_1778813655463/summary.json` showed:

- ChatGPT-web `gpt-5-5-pro` and `gpt-5-4-pro` stream metadata reports the requested Pro slug as `model_slug` / `default_model_slug`, while the internal serving id is `resolved_model_slug: "i-cot"`.
- Both Pro slugs can run ChatGPT web search through `SonicTool`; their search probes returned nonzero `search_result_groups` and server metadata `turn_use_case: "search"`.
- The separate ChatGPT-web `deep-research` model works as its own model slug and reports internal `resolved_model_slug: "i-5-mini-m"`.
- Prompting Pro models to "use deep research mode" produced ordinary web-search turns, not a verified switch into the separate `deep-research` model/control plane.

## Translation Rules: Codex App to Public Responses

Map Codex App input items to public Responses input items as follows:

| Codex App item | Public Responses item | Notes |
|---|---|---|
| `{ type: "text", text }` local input | `{ type: "input_text", text }` | Local conversation input uses app-server item names. |
| `{ type: "message", role: "user", content: [{ content_type: "text", text }] }` | `{ role: "user", content: [{ type: "input_text", text }] }` | Cloud `/wham/tasks` wraps text as `content_type: "text"`. |
| `{ type: "image", url: "data:image/..." }` | `{ type: "input_image", image_url: "data:image/..." }` | Direct base64 data URL path. |
| `{ type: "localImage", path }` | Read file, then `{ type: "input_image", image_url: "data:image/...;base64,..." }` or upload then `file_id` | Public Responses cannot consume arbitrary local paths. |
| `{ type: "image_asset_pointer", asset_pointer }` | Download/resolve asset, then `input_image` via data URL or public `file_id` | `sediment://`, `file-service://`, and `/wham/...` pointers are Codex/ChatGPT-internal, not public Responses IDs. |
| `{ content_type: "image_asset_pointer_citation", asset_pointer }` | Same as `image_asset_pointer`, plus surrounding text/page context | Seen on browser/PDF comment attachments in the app bundle. |
| PDF/document attachment | Prefer `{ type: "input_file", file_id/file_url/file_data }` | If only a screenshot is available, preserve it as `input_image` plus page/comment text. |
| Worktree snapshot or shell snapshot | Do not map directly to `input_file` | Treat as Codex environment/context bootstrap; include source/diff text separately if needed. |

## Translation Rules: Public Responses to Codex App/Core

Codex App and Codex core accept narrower user input than public Responses:

- Core `UserInput` and app-server `UserInput` are `Text`, `Image`, `LocalImage`, `Skill`, and `Mention`; there is no native public `input_file`.
- Codex `ContentItem` is `input_text`, `input_image`, and `output_text`; it does not model public `input_file`, `file_id`, `file_url`, or `file_data`.
- Public `system`/`developer` messages should not be jammed into turn user input. Map them to thread/base/developer instructions when using the app-server path.
- Public prior assistant/history items need adapter-owned state or `thread/inject_items`; `previous_response_id` is not a Codex App turn ID.
- Preserve raw public Responses items in adapter state if exact round-tripping matters; the Codex UI projection is narrower than public output items.

## Current UMP Codex Responses Behavior

The Rust UMP Codex route already performs a Responses-to-Codex-backend adaptation:

- Targets `wss://chatgpt.com/backend-api/codex/responses` or `https://chatgpt.com/backend-api/codex/responses`.
- Resolves `model` through UMP aliases and rejects non-Codex providers.
- Rewrites `model` to the upstream Codex model.
- Removes `stream`, `stream_options`, `background`, `max_output_tokens`, and `max_tokens`.
- Forces `store: false`.
- Defaults `instructions` to `"You are a helpful assistant."` when absent.
- Defaults `reasoning` to `{ "effort": "medium", "summary": "auto" }` and ensures `reasoning.summary`.
- Ensures `include` contains `reasoning.encrypted_content`.
- Sends HTTP create bodies as the prepared flat Responses-shaped body.
- The WebSocket proxy uses `response.create` frames and rewrites the first client frame plus later text/binary frames that parse as `response.create` events or raw model bodies. Later invalid JSON data frames error instead of falling through as raw pass-through.
- Passes live Codex streaming compaction through `/v1/responses`: `{ "type": "compaction_trigger" }` in `input` reaches the backend and returns `{ "type": "compaction", "encrypted_content": "..." }`.
- Does not currently implement `POST /v1/responses/compact`; the local route returned `405` in the live UMP probe.

Live backend proof on 2026-05-14: `POST https://chatgpt.com/backend-api/codex/responses` with the Codex OAuth bearer accepted a flat streamed Responses-shaped body when it included `instructions`, list-shaped `input`, `stream: true`, `store: false`, `tool_choice: "none"`, low reasoning effort, and `include: ["reasoning.encrypted_content"]`. The backend returned SSE events `response.created`, `response.in_progress`, `response.output_item.added`, `response.output_item.done`, `response.content_part.added`, `response.output_text.delta`, `response.output_text.done`, `response.content_part.done`, and `response.completed`; the final response had `model: "gpt-5.4-mini-2026-03-17"`, `store: false`, usage, and encrypted reasoning output.

Live compaction proof on 2026-05-14:

- `POST https://chatgpt.com/backend-api/codex/responses/compact` returned `200` with preserved user messages plus `{ "type": "compaction_summary", "encrypted_content": "..." }`.
- Replaying the compacted item as `{ "type": "compaction", "encrypted_content": "..." }` in a follow-up `/codex/responses` request preserved hidden assistant context.
- `POST https://chatgpt.com/backend-api/codex/responses` with `{ "type": "compaction_trigger" }` in `input` returned `200` SSE with `{ "type": "compaction", "encrypted_content": "..." }`; replaying retained user messages plus that compacted item preserved hidden assistant context.
- The live backend rejected `{ "type": "context_compaction" }` with `400 invalid_value`. The latest pulled `openai-codex` source still contains an under-development, default-off `remote_compaction_v2` path that uses `context_compaction`; treat that as source-only and not the live wire shape for this proxy.

Negative live backend proof from the same endpoint:

- Missing `OpenAI-Account` or malformed account headers returned `401 unauthorized_unknown`.
- `{}` returned `400` because the model was missing.
- A body without `instructions` returned `400` with `Instructions are required`.
- A string-shaped `input` returned `400` with `Input must be a list`.
- Omitting `stream: true` returned `400` with `Stream must be set to true`.

Caveat: the `response.create` envelope is a WebSocket protocol frame shape, not the HTTP Codex backend create body. Live HTTP proof shows the ChatGPT Codex backend accepts a flat streamed body when the required fields are present.

## Responses to Codex Backend Gaps

| Gap | Adapter policy |
|---|---|
| **State and IDs** | Public `response.id`, `conversation`, and `previous_response_id` are not Codex App `threadId`, `sessionId`, or `turnId`. Maintain adapter state from public response IDs to Codex threads/history. |
| **`previous_response_id`** | Codex core only fills it opportunistically inside a turn-scoped WebSocket model session after a cached last response. Do not map it to `turnId`. |
| **Background lifecycle** | Public `background` plus retrieve/cancel polling is not represented by Codex App turn lifecycle. UMP currently removes `background`; app turn interruption is `turn/interrupt`, not public response cancel. |
| **Metadata and truncation** | Public `metadata`/`truncation` are not the same as Codex `client_metadata`, `responsesapiClientMetadata`, `x-codex-turn-metadata`, or rollout truncation policy. |
| **Permissions and environment** | Public Responses has no equivalent for Codex `cwd`, `approvalPolicy`, `approvalsReviewer`, `sandboxPolicy`, permissions profiles, or app environment selection. Inject these from adapter configuration. |
| **Token limits** | Public `max_output_tokens` is a valid create parameter, but UMP Codex currently strips `max_output_tokens` and `max_tokens`. Fail closed or document that caps are ignored. |
| **Store semantics** | Public `store` controls later API retrieval. UMP Codex forces upstream `store: false`, so the adapter must distinguish volatile continuation state from public retrievable storage. For caller-visible `store: true`, own persistence or reject; for `store: false`, retrieve/input-item routes should return `404`. |
| **Compaction wire shapes** | Live Codex backend compaction uses `compaction_trigger` for streaming compaction requests, `compaction` for replayable compacted items, and `compaction_summary` as the compact endpoint output alias. Do not use `context_compaction` upstream unless a later live probe proves the backend accepts it. |
| **Structured output** | Codex App/core maps `output_schema` to `text.format.type = json_schema` with a fixed `name`. UMP does not implement generic `response_format`, `json_object`, or provider-wide structured-output parity. |
| **`include` values** | Codex requires/preserves `reasoning.encrypted_content` for reasoning continuity, but public `include` has more values for hosted tools, file/search outputs, and logprobs. Preserve or reject unsupported values deliberately. |
| **Service tier** | Codex core filters `service_tier` by model support; UMP may pass it through. Do not assume public tier semantics are honored by every Codex path. |
| **Tool surface** | Codex `ToolSpec` covers `function`, `namespace`, `tool_search`, `image_generation`, `web_search`, and `custom`. Public `shell`, `computer`, `mcp`, `skills`, `file_search`, and `code_interpreter` need explicit mapping, rejection, or out-of-band handling. |
| **MCP** | Codex MCP is local/app configured and converted to namespace/function-style tools; public hosted MCP uses `{ type: "mcp", server_label, ... }` and emits `mcp_*` items. These are not equivalent. |
| **Shell/apply patch** | Codex shell/apply_patch are local function/custom/freeform tool flows with sandbox/approval handling. Public shell emits `shell_call`/`shell_call_output`; map explicitly. |
| **Function-call streaming** | Codex SSE parsing handles `custom_tool_call_input.delta`, but public `response.function_call_arguments.delta` is currently not modeled in the same internal event path. |
| **Stream identity** | Codex internal `OutputTextDelta` drops public `item_id`/`output_index`; deltas route through active-item state. Interleaved public streams need an adapter-side ordering/index map. |
| **Incomplete/status events** | `response.incomplete` becomes a generic stream error; `response.completed` does not preserve every public status/incomplete detail. Preserve public status separately if clients rely on it. |
| **Content parts and annotations** | Public content-part/refusal/annotation/logprob events are not fully modeled by Codex `ContentItem`. Unknown events may be traced and dropped. |
| **Voice/realtime audio** | App dictation and `thread/realtime/*` are not plain Responses requests. Treat `/transcribe` and Realtime WebSocket/WebRTC as separate adapter routes. |
| **Embeddings** | Public `/v1/embeddings` is not Codex backend `/codex/responses` and embedding models are absent from the Codex catalog. Route embeddings through public `api.openai.com` only when live probes prove the bearer accepts the model; reject embedding model IDs on Codex backend create/compact paths. |
| **Raw item preservation** | Completed items are deserialized into Codex `ResponseItem`; unsupported public variants become `Other` and extra fields can be lost. Keep raw JSON separately if exact public shape matters. |
| **Files and outputs** | Public `input_file`, file citations, container file citations, and code-interpreter file outputs need a file channel. Codex dynamic tool outputs currently carry text/images, not arbitrary file payloads. |
| **Image generation outputs** | Public image generation returns `image_generation_call` with base64 `result`; Codex core/app saves or projects generated images into artifact/file UI state. Preserve both the raw tool result and artifact metadata. |

## Image Output Difference

The public Responses image-generation path is explicit:

```json
{
  "model": "gpt-5.5",
  "input": "Generate an image...",
  "tools": [{ "type": "image_generation" }]
}
```

Generated images return as `image_generation_call` output items with a base64 `result`.

Codex App UI does not primarily expose that raw output shape. In the inspected bundle, generated images are surfaced as artifacts/files, for example files matching `ig_<hex>.(png|jpg|webp|...)` in `webview/assets/local-conversation-thread-C4DDoT1D.js`. Treat Codex image outputs as an artifact layer over the underlying Responses/tool result, not as the public API object itself.

## Adapter Gotchas

- Do not pass `localImage.path` through to `/v1/responses`; read or upload the bytes first.
- Only trusted Codex App/Core channels may resolve `localImage.path`; public Responses JSON must never be able to spoof trusted local-file provenance.
- Do not treat `image_asset_pointer.asset_pointer`, `sediment://`, `file-service://`, or `/wham/...` IDs as public `file_id`; resolve or re-upload them.
- Preserve image ordering relative to text when converting `input_items`.
- Preserve dimensions and byte size when available for logging or artifact metadata, but public `input_image` does not require those fields.
- For image generation or editing, the public Responses request must include the `image_generation` tool if the adapter wants the Responses API to produce an image tool result.
- Reject or explicitly degrade public Responses fields that Codex ignores; silent success makes debugging state/tool/file bugs painful.
- Keep `Create Field Policy` aligned with the current public Responses create schema whenever OpenAI adds request fields; new fields must default to reject until assigned an explicit policy.
- Keep raw public Responses JSON alongside Codex UI projections when round-trip fidelity matters.

## Evidence Anchors

### Live Validation Evidence

Fresh live/code validation pass: 2026-05-14. All tokens stayed in shell variables or subagent process memory and are redacted from recorded commands.

| Area | Live result |
|---|---|
| Codex model catalog | `GET https://chatgpt.com/backend-api/codex/models?client_version=26.506.31421` returned `200` and six IDs: `gpt-5.5`, `gpt-5.4`, `gpt-5.4-mini`, `gpt-5.3-codex`, `gpt-5.2`, `codex-auto-review`. `client_version=0.99.0` returned five IDs without `gpt-5.5`. Missing `client_version` returned `400 invalid_request_error`. `GET https://api.openai.com/v1/models` with the same bearer returned `403` missing `api.model.read`. |
| Codex backend responses | `POST https://chatgpt.com/backend-api/codex/responses` accepted a flat streamed Responses-shaped body only after `instructions`, list-shaped `input`, and `stream: true` were present. It returned `200` SSE with `response.output_text.delta: "OK"`, `response.completed`, `store: false`, and encrypted reasoning output. |
| Codex backend compaction | `POST https://chatgpt.com/backend-api/codex/responses/compact` returned preserved user messages plus `compaction_summary.encrypted_content`; canonical replay as `compaction.encrypted_content` worked in a follow-up response. `POST /backend-api/codex/responses` with `compaction_trigger` returned streamed `compaction.encrypted_content`; replaying retained user messages plus that item also worked. `context_compaction` returned `400 invalid_value`. |
| Current UMP compaction behavior | Local UMP `POST /v1/responses` already passed `compaction_trigger` through and returned a replayable `compaction.encrypted_content` item. Local UMP `POST /v1/responses/compact` returned `405`, so that route still needs implementation. |
| Public Realtime | `wss://api.openai.com/v1/realtime?model=gpt-realtime-2` with the Codex OAuth bearer upgraded with `101`, accepted GA `output_modalities`, accepted text-only `conversation.item.create`, and emitted `response.output_text.delta`, `response.output_text.done`, and `response.done`. `OpenAI-Beta: realtime=v1` with `gpt-realtime-2` produced `invalid_model`; old `response.modalities` on GA produced `unknown_parameter`. |
| Public Realtime WebRTC call setup | `POST /v1/realtime/calls?model=gpt-realtime` with an intentionally minimal SDP offer returned `400 invalid_offer`, proving auth acceptance and SDP-body rejection. |
| Public audio/auth boundaries | `POST /v1/realtime/client_secrets` returned `200 realtime.transcription_session` with `session.type: transcription` and `gpt-realtime-whisper`; the old `/v1/realtime/transcription_sessions` beta shape returned `400 beta_api_shape_disabled`. `POST /v1/audio/transcriptions` with an intentionally empty file returned `400 unsupported_value`, proving auth acceptance and file-body rejection. `POST /v1/responses` returned `401` missing `api.responses.write`. `POST /v1/audio/speech` with `gpt-4o-mini-tts` returned `401 missing_scope` for `api.model.audio.request`. |
| Public embeddings | `GET https://api.openai.com/v1/models` returned `403` missing `api.model.read`. `POST https://api.openai.com/v1/embeddings` returned `200` for `text-embedding-3-small` and `text-embedding-3-large`, `401` missing `model.request` for `text-embedding-ada-002`, and `403` when asked to embed from `gpt-5.4-mini`. The Codex catalog listed no embedding IDs; `POST /backend-api/codex/responses` with `text-embedding-3-small` returned `400 model not supported for Codex with a ChatGPT account`. |
| ChatGPT product surfaces with Codex OAuth | `/backend-api/f/conversation/prepare` plus Sentinel plus `/backend-api/f/conversation` returned `200` SSE; `/backend-api/celsius/ws/user` returned `200` WSS bootstrap metadata; `/realtime/vp?dcid=0` returned `201` for HAR-shaped SDP/session text fields. |
| App bundle media paths | Shipped JS shows image paste via `FileReader.readAsDataURL`, upload creation through `/files`, blob upload with `X-Codex-Base64: 1`, finalize through `/files/{file_id}/uploaded`, task creation through `/wham/tasks`, remote image fetch through `/files/download/{file_id}`, and dictation through `/transcribe`. |
| App-server model/thread paths | Shipped JS exposes `model/list` and `thread/start`; generated app-server protocol confirms `modelProvider/capabilities/read` uses camelCase wire fields `namespaceTools`, `imageGeneration`, `webSearch`. |
| UMP/Codex source paths | Real source confirms UMP body rewrites, forced `store: false`, reasoning defaults, encrypted reasoning include, WebSocket frame rewrites, Codex SSE parsing, and `response.processed` as a lifecycle ACK rather than a retrieve API. |

Primary local evidence:

- `unified-model-proxy-v2/src/upstream/codex.rs` — UMP Codex body rewrites, forced defaults, ChatGPT Codex backend endpoints, flat HTTP forwarding, and WebSocket `response.create` framing.
- `unified-model-proxy-v2/src/route/websocket.rs` — first-frame WebSocket rewrite and pass-through behavior.
- `openai-codex/codex-rs/codex-api/src/common.rs` — Codex `ResponsesApiRequest`, `ResponseCreateWsRequest`, `ResponseEvent`, and structured-output `text.format` creation.
- `openai-codex/codex-rs/codex-api/src/endpoint/models.rs` — provider `GET models?client_version=...` client for remote model catalogs.
- `openai-codex/codex-rs/app-server/src/models.rs` — app-server mapping from Codex model presets to `model/list` responses.
- `openai-codex/codex-rs/app-server-protocol/src/protocol/v2/model.rs` — exact `model/list`, provider-capability, model-reroute, and model-verification payload shapes.
- `openai-codex/codex-rs/app-server/src/request_processors/config_processor.rs` — `modelProvider/capabilities/read` implementation and response fields.
- `openai-codex/codex-rs/codex-api/src/sse/responses.rs` — selected `response.*` event parsing and unhandled-event behavior.
- `openai-codex/codex-rs/protocol/src/user_input.rs` — core `UserInput` variants.
- `openai-codex/codex-rs/protocol/src/models.rs` — `ResponseInputItem`, `ContentItem`, `ResponseItem`, and `Other` fallback.
- `openai-codex/codex-rs/app-server-protocol/src/protocol/v2/turn.rs` — `turn/start`, `turn/steer`, `turn/interrupt`, permissions, sandbox, model, service tier, effort, and output schema fields.
- `openai-codex/codex-rs/app-server-protocol/src/protocol/v2/thread.rs` — `thread/inject_items` for raw Responses items.
- `openai-codex/codex-rs/app-server-protocol/src/protocol/v2/item.rs` — Codex `ThreadItem` projections and dynamic tool output item limits.
- `openai-codex/codex-rs/tools/src/tool_spec.rs` — Codex tool spec variants.
- `openai-codex/codex-rs/core/src/client.rs` — Codex request assembly, `prompt_cache_key`, service-tier filtering, and opportunistic WebSocket `previous_response_id`.
- `openai-codex/codex-rs/codex-api/src/files.rs` — internal `sediment://` file handling.
- `openai-codex/codex-rs/core/src/mcp_openai_file.rs` — Apps SDK file metadata rewriting.
- `openai-codex/codex-rs/app-server-protocol/src/protocol/v2/realtime.rs` — app-server realtime audio chunks, start params, append audio/text params, and transcript/audio notifications.
- `openai-codex/codex-rs/app-server-protocol/src/protocol/common.rs` — experimental `thread/realtime/*` request and notification method names.
- `openai-codex/codex-rs/app-server/src/request_processors/turn_processor.rs` — realtime start/audio/text/stop requests mapped to core `Op::RealtimeConversation*`.
- `openai-codex/codex-rs/app-server/src/bespoke_event_handling.rs` — realtime core events projected to app-server transcript/audio/Sdp/error notifications.
- `openai-codex/codex-rs/core/src/realtime_conversation.rs` — realtime websocket startup, API-key requirement for WebSocket transport, text input forwarding, and `response.create` queueing.
- `openai-codex/codex-rs/codex-api/src/endpoint/realtime_websocket/methods_v2.rs` — realtime v2 session config, `gpt-4o-mini-transcribe`, VAD, output modality, and background-agent tools.
- `openai-codex/codex-rs/codex-api/src/endpoint/realtime_websocket/protocol_v2.rs` — parsing of input transcript deltas/done, output transcript deltas/done, audio deltas, and realtime response events.
- `openai-codex/codex-rs/realtime-webrtc/src/native.rs` — macOS WebRTC peer connection and local microphone track setup.
- `openai-codex/codex-rs/tui/src/voice.rs` — TUI realtime microphone capture, PCM conversion, and audio chunk sending.

Primary app-bundle evidence:

- `main-DnQgBHvi.js` — shipped app callsites for `listModels({ includeHidden: true, cursor: null, limit: 100 })`, automation model fallback, and `startThread` model/model-provider fields.
- `webview/assets/composer-DawxvKsB.js` — image paste/drop intake, upload state, cloud pre-upload.
- `webview/assets/codex-api-B3jrGDqO.js` — `/wham/tasks` request construction and `input_items`.
- `webview/assets/app-server-manager-signals-C1h8B-R-.js` — local `image` / `localImage` item construction and browser/PDF image citation handling.
- `preload.js` / `.vite/build/preload.js` — packaged preload bundle exposing `electronBridge.getPathForFile`; the root copy is the extracted app artifact, while `.vite/build/preload.js` is the bundled source path referenced inside it.
- `webview/assets/local-conversation-thread-C4DDoT1D.js` — generated-image artifact presentation.
- `webview/assets/global-dictation-page-Bg1-uDEX.js` — renderer microphone permission, `getUserMedia`, `MediaRecorder`, and global dictation IPC events.
- `webview/assets/use-recording-waveform-gMqiyEbV.js` — dictation waveform, `/transcribe` multipart upload, and optional transcript cleanup via `/codex/responses`.

## Validation Passes

This note reflects three passes:

1. Six-agent first pass over schema/input, tools, media/files, lifecycle, streaming/output, and public Responses compatibility targets.
2. Six-agent second pass that re-checked the candidate gaps against local source and public API references.
3. Five-agent live-proof pass plus direct probes against real API endpoints with `curl`, WebSocket tooling, the shipped Codex App bundle, `openai-codex`, and UMP source. This pass is now the primary validation basis for Codex-specific claims.

Second-pass refinements applied:

- Use current public `computer` naming; `computer_use_preview` is legacy.
- Keep HTTP flat-body forwarding separate from WebSocket `response.create` framing.
- Treat `max_output_tokens` removal as current UMP policy, not a public Responses requirement.
- Treat `file-service://` and app `image_asset_pointer_citation` as app-bundle evidence, not first-party Rust protocol evidence.

Live-proof refinements applied:

- Corrected `modelProvider/capabilities/read` to camelCase wire fields.
- Split volatile continuation state from public retrievable storage for `store: false`.
- Corrected Realtime transcription-session auth: Codex OAuth accepted the GA `POST /v1/realtime/client_secrets` transcription-client-secret shape in the fresh probe; the old beta `/v1/realtime/transcription_sessions` shape no longer counts as passing evidence.
- Added public Realtime WebRTC call setup and public `/audio/transcriptions` auth-gate findings for Codex OAuth.
- Added ChatGPT product-surface findings: ChatGPT-web conversation, Celsius bootstrap, and ChatGPT voice `/realtime/vp` accepted the current Codex OAuth bearer in sanitized live probes.
- Replaced docs-as-proof wording with live API/code validation wording.
- Added live Codex backend `/codex/responses` requirements: `instructions`, list `input`, and `stream: true`.
- Added live compaction findings: streaming compaction uses `compaction_trigger` -> `compaction`; compact endpoint returns `compaction_summary`; replay uses `compaction`; `context_compaction` is not accepted by the live backend.
- Added public embeddings findings: Codex OAuth accepted `POST /v1/embeddings` for `text-embedding-3-small` and `text-embedding-3-large`, but embedding models are absent from the Codex catalog and rejected on Codex backend `/codex/responses`.
