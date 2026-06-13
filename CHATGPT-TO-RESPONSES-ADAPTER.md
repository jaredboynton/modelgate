# ChatGPT Web / Atlas Responses Adapter Notes

**Note: For testing purposes, codex auth always resides under ~/.codex/auth.json**

## Purpose

Use this note when building or reviewing an adapter between:

- Public OpenAI `/v1/responses`-shaped clients.
- ChatGPT web/backend conversation surfaces reached through ChatGPT web or Atlas-auth cookies.
- ChatGPT voice/realtime and public OpenAI audio routes when a client tries to treat voice as "Responses".

This is the ChatGPT-web companion to `docs/CODEX-TO-RESPONSES-ADAPTER.md`. Codex uses a separate ChatGPT Codex backend Responses surface. ChatGPT web does not. A ChatGPT-to-Responses adapter is therefore a projection layer over `/backend-api/f/conversation`, Sentinel admission, ChatGPT SSE/Celsius streams, and adapter-owned state.

## Bottom Line

The local evidence supports a working ChatGPT-web Responses facade, but it is not a public Responses pass-through.

- `POST /v1/responses` can be backed by ChatGPT web models through UMP's `chatgpt-web` adapter.
- The adapter converts public `input`, `instructions`, function `tools`, `tool_choice`, `function_call_output`, `reasoning.effort`, and `service_tier` into ChatGPT web chat requests.
- It converts ChatGPT chat-completion chunks back into Responses SSE events such as `response.created`, `response.output_text.delta`, `response.function_call_arguments.delta`, and `response.completed`.
- `previous_response_id` works only through adapter-owned continuation cache entries that map public response IDs to ChatGPT `conversation_id` / `message_id` metadata.
- Public file/image/state/storage features are incomplete. Current code drops or ignores several public fields; a strict adapter should fail closed instead.
- Voice/realtime is a separate control plane. The HAR proves `chatgpt.com/realtime/vp` WebRTC offer exchange, not `/v1/responses` audio input.
- The local Codex OAuth token is not a public OpenAI API key, but this pass showed it is accepted by several ChatGPT web backend surfaces and by public STT/realtime auth gates. It still lacks public Responses and TTS scopes.
- Atlas auth appears compatible with the ChatGPT-web auth envelope, but Atlas-only local tool and `AgentEventAPI` protocols remain `blocked-live`.

## Validation Standard

Treat the ChatGPT-specific behavior in this note as live/API-and-code validated where explicitly marked. The current grounding is:

- Live local UMP probes against `http://127.0.0.1:18741/v1/responses`, with text output, tool-call output, and `previous_response_id` IDs redacted.
- Live ChatGPT web auth probes using the local ChatGPT-web envelope and native ChatGPT `binarycookies`, with cookies/bearers held in temp files or memory and not logged.
- Live public API probes with ChatGPT session tokens, recording only status/error classes.
- HAR shape analysis from `${HOME}/Downloads/chatgpt-voice-tts.har`, which was captured from ChatGPT web voice, not Atlas.
- UMP source inspection under `${UNIFIED_MODEL_PROXY_ROOT}`.
- Existing focused unit tests for the ChatGPT-web Responses facade.

Validation artifacts for this pass are archived under `docs/gpt-web-evidence/chatgpt_to_responses_validation_20260513_230751/`.

## Proxy Implementation Contract

A correct ChatGPT-to-Responses proxy must expose explicit policy for every public route and field. Unsupported public features should be rejected with OpenAI-compatible errors instead of silently converting to empty prompts or ignored state.

### Required Routes

| Client-facing route | Upstream / internal route | Required behavior |
|---|---|---|
| `POST /v1/responses` | ChatGPT web `/backend-api/f/conversation/prepare`, Sentinel prepare/finalize, then `/backend-api/f/conversation` | Convert public Responses input into ChatGPT web chat request, run Sentinel admission, stream ChatGPT SSE/Celsius output, and project chunks back to Responses SSE events. |
| `GET /v1/models` | Adapter catalog / route table, not a canonical ChatGPT public model endpoint | List only configured `chatgpt-web/*` IDs. Do not assume every listed model is entitlement-available; validate on use. |
| `GET /v1/responses/{response_id}` | Adapter-owned response store only | Return public stored objects only if the adapter persisted them. Do not call ChatGPT conversation history as if it were public Responses retrieval. |
| `POST /v1/responses/{response_id}/cancel` | Adapter-owned in-flight work only | Map only if the proxy owns the stream and cancellation handle. Otherwise return `501` or `409`. |
| `GET /v1/responses/{response_id}/input_items` | Adapter-owned public input-item store only | Return `404` for volatile `previous_response_id` continuation state. |
| Public file/image inputs | Explicit byte resolution/upload layer | Current facade drops `input_image` and `input_file` content. A production adapter must reject or implement upload/provenance. |
| ChatGPT voice WebRTC | `POST https://chatgpt.com/realtime/vp?dcid=0` | Separate voice route. Multipart text fields `sdp` and `session`; not `/v1/responses`. |
| Celsius websocket streaming | `GET /backend-api/celsius/ws/user`, then `wss://ws.chatgpt.com/...` | Optional stream handoff after ChatGPT emits `stream_handoff`. Redact WSS path/query in logs. |
| Public audio/TTS/STT | `api.openai.com/v1/audio/speech`, `/audio/transcriptions`, `/realtime/calls` | Separate public API routes. ChatGPT session tokens accepted these auth paths in probes, but request bodies still need public API shape. |

### ChatGPT Backend Sequence

The existing UMP adapter sequence is:

1. Resolve ChatGPT-web credentials from `CHATGPT_WEB_ENVELOPE` or `~/.unified-model-proxy/chatgpt-web.json`.
2. Build browser-like headers: bearer, cookie, origin/referer, user-agent, `oai-client-version`, `oai-client-build-number`, `oai-device-id`, `oai-session-id`, `oai-language`, and target headers.
3. `POST /backend-api/f/conversation/prepare` with:
   - `action: "next"`;
   - `parent_message_id`;
   - ChatGPT model slug;
   - `client_prepare_state: "none"`;
   - timezone and `conversation_mode`;
   - `partial_query`;
   - `supports_buffering`;
   - `thinking_effort`.
4. `POST /backend-api/sentinel/chat-requirements/prepare`.
5. Solve Sentinel proof-of-work and optional Turnstile path.
6. `POST /backend-api/sentinel/chat-requirements/finalize`.
7. `POST /backend-api/f/conversation` with:
   - `messages`;
   - `client_prepare_state: "success"`;
   - `service_tier` when supplied/defaulted;
   - `thinking_effort`;
   - `tools` and/or `local_function_names` when supported;
   - `conversation_id` and parent message metadata for continuations.
8. Parse ChatGPT SSE patches into text/tool-call chunks, then Responses events.
9. If `stream_handoff` appears and `UMP_CHATGPT_WEB_STREAM=auto|ws`, fetch Celsius WSS URL and continue reading encoded items from `ws.chatgpt.com`.

### Required State

- Map public `response.id` to ChatGPT `conversation_id` and latest assistant `message_id`.
- Keep this continuation state separate from public retrievable response storage.
- Preserve raw upstream events/items when the public projection is incomplete.
- Track requested public model, ChatGPT backend slug, reasoning effort, service tier, and auth source.
- Record whether a request used normal ChatGPT conversation, Celsius handoff, private voice WebRTC, or public audio APIs.

### Required Failure Policy

- Reject unsupported public fields by default.
- Reject unknown or entitlement-blocked models before or immediately after the first upstream failure; do not silently fall back.
- Reject public `input_file` / `input_image` unless byte resolution and provenance tracking exist.
- Reject public hosted tools unless an explicit ChatGPT mapping exists.
- Reject public `store: true` unless adapter-owned storage is implemented.
- Never accept public JSON local filesystem paths as trusted file input.
- Never log bearer tokens, cookies, ChatGPT account/device/session IDs, WSS verify params, raw response IDs, or raw user identifiers.

## Public Route Disposition

| Public route | ChatGPT adapter disposition |
|---|---|
| `POST /v1/responses` | Implemented in current UMP for `chatgpt-web/*` through `runChatGptWebResponsesStream`. |
| `GET /v1/responses/{response_id}` | Not proven. Should be adapter-owned storage only. |
| `DELETE /v1/responses/{response_id}` | Not implemented/proven. Should delete adapter-owned public storage only. |
| `POST /v1/responses/{response_id}/cancel` | Not implemented/proven. Can only cancel adapter-owned in-flight streams. |
| `GET /v1/responses/{response_id}/input_items` | Not implemented/proven. Should return only adapter-owned public items. |
| `POST /v1/responses/input_tokens` | Not implemented/proven. Return `501` unless using a model-matched tokenizer. |
| `POST /v1/responses/compact` | Not a ChatGPT web backend feature. Return `501` unless adapter-owned compaction exists. |
| `/v1/conversations/*` | Optional adapter-state feature; do not map directly to ChatGPT conversation IDs without explicit state ownership. |

## Create Field Policy

| Public create field | Current behavior | Required strict policy |
|---|---|---|
| `model` | Routed through `chatgpt-web/*` table and mapped to ChatGPT slug. | Required; validate against catalog and entitlement. |
| `input` string | Converts to one user message. | Supported. |
| `input` message text parts | `input_text`, `output_text`, and `text` become plain text. | Supported with raw item preservation. |
| `input_image` | Dropped by current conversion. | Reject until image upload/resolution is implemented. |
| `input_file` | Dropped by current conversion. | Reject until file upload/resolution is implemented. |
| `instructions` | Prepended as system message. | Supported. |
| `previous_response_id` | Uses `ChatGptWebContinuationCache`; missing/expired returns invalid request. | Supported only for adapter-owned IDs. |
| `conversation` | Ignored by current conversion. | Reject unless adapter-owned public conversation state exists. |
| `store` | Ignored by current conversion. | Reject `store: true` unless public storage exists; for `store: false`, keep volatile continuation only. |
| `metadata` | Ignored. | Store in adapter state or reject. |
| `background` | Ignored. | Reject unless background queue/retrieve/cancel semantics exist. |
| `max_output_tokens` | Mapped to chat `max_tokens`; upstream support not fully proven. | Validate or reject if ChatGPT ignores it. |
| `max_tool_calls` | Ignored. | Enforce in adapter tool loop or reject. |
| `parallel_tool_calls` | Copied to chat request. | Validate with tool path or reject. |
| `service_tier` | Copied to chat request; adapter defaults priority in chat path. | Validate accepted values; do not imply public tier semantics. |
| `reasoning.effort` | Mapped to `reasoning_effort`, then ChatGPT `thinking_effort`. | Supported for models with known effort map. |
| `include` | Ignored. | Reject unsupported include values. |
| `text.format` | Ignored. | Reject until structured output mapping is implemented. |
| `tools` function | Converted to OpenAI chat `tools[]`; mirrored into ChatGPT tool names/wrapper. | Supported for function tools only. |
| Hosted tools (`web_search`, `file_search`, `mcp`, `computer`, `code_interpreter`, shell) | Non-function tools are dropped. | Reject unless explicit bridge exists. |
| `tool_choice` function | Converted to chat function choice. | Supported for mapped function tools. |
| Hosted `tool_choice` | Ignored. | Reject. |
| `stream` | Facade always streams Responses SSE. | Supported for streaming; non-streaming should be collected deliberately if exposed. |
| `temperature` / `top_p` | Copied to chat request. | Validate model support or reject. |
| `prompt` | Ignored. | Reject unless public prompt templates are resolved before upstream. |
| `prompt_cache_key` / `prompt_cache_retention` | Ignored. | Reject unless semantics are implemented. |
| `safety_identifier` / `user` | `user` is copied; `safety_identifier` ignored. | Treat as metadata through a documented trust boundary or reject. |

## Stream Event Mapping

| Public stream event / field family | ChatGPT adapter behavior |
|---|---|
| `response.created` | Synthesized before first ChatGPT chunk. |
| `response.output_item.added` | Synthesized for first text item or tool call. |
| `response.content_part.added` | Synthesized for text output. |
| `response.output_text.delta` | Mapped from ChatGPT text deltas. |
| `response.content_part.done` | Synthesized when text item closes. |
| `response.function_call_arguments.delta` | Mapped from ChatGPT tool-call argument deltas. |
| `response.function_call_arguments.done` | Synthesized when tool-call item closes. |
| `response.output_item.done` | Synthesized with completed message/function_call item. |
| `response.completed` | Synthesized once after finish; includes adapter-projected output array. |
| `response.failed` / `response.incomplete` | Not fully proven for ChatGPT web facade. Should map upstream failures into public-shaped error events. |
| `response.refusal.*`, annotations, citations, logprobs | Not represented by current conversion. Reject strict requests or preserve raw upstream. |
| Interleaved item IDs | Current conversion tracks text and tool indexes. Preserve raw events if richer interleaving appears. |

## Model Discovery and Selection

The local UMP `/v1/models` probe returned 13 `chatgpt-web/*` IDs:

- `chatgpt-web/gpt-5.5-instant`
- `chatgpt-web/gpt-5.5-thinking`
- `chatgpt-web/gpt-5.5-pro`
- `chatgpt-web/gpt-5.4-thinking`
- `chatgpt-web/gpt-5.4-pro`
- `chatgpt-web/gpt-5.4-t-mini`
- `chatgpt-web/gpt-5.3-mini`
- `chatgpt-web/gpt-5.3-instant`
- `chatgpt-web/gpt-5.2-thinking`
- `chatgpt-web/gpt-5.2-pro`
- `chatgpt-web/o3`
- `chatgpt-web/o3-pro`
- `chatgpt-web/deep-research`

Do not treat catalog presence as entitlement proof. In the live pass:

- `chatgpt-web/gpt-5.5-pro` completed text Responses SSE.
- `chatgpt-web/gpt-5.5-thinking` completed text Responses SSE.
- `chatgpt-web/gpt-5.5-instant` returned upstream `403` / access denied for a text-only Responses probe, but a forced function-tool probe did produce a completed function-call item. Treat this as a model/path entitlement risk until repeated against fresh auth.

Focused ChatGPT-web Pro validation on 2026-05-14 added:

- `gpt-5-5-pro` and `gpt-5-4-pro` both report the requested Pro slug in ChatGPT stream metadata as `model_slug` / `default_model_slug`.
- Both Pro slugs also include internal `resolved_model_slug: "i-cot"`. Treat that as the serving/runtime implementation id, not as evidence that the requested Pro model was ignored.
- Both Pro slugs invoked web search successfully when prompted. The stream metadata included `tool_name: "SonicTool"`, nonzero `search_result_groups`, and server metadata `turn_use_case: "search"`.
- The standalone `deep-research` model slug also works and reports `default_model_slug: "deep-research"` with internal `resolved_model_slug: "i-5-mini-m"`.
- Asking `gpt-5-5-pro` or `gpt-5-4-pro` to use "deep research mode" produced normal web-search turns, not a proven switch into the separate `deep-research` model/control plane. Treat Pro web search as validated; treat Pro deep-research control-plane access as not proven.

Evidence: `docs/gpt-web-evidence/chatgpt_web_pro_search_research_probe_1778813655463/summary.json`.

## Auth Sources

Validated auth sources:

| Source | Evidence | Result |
|---|---|---|
| `~/.unified-model-proxy/chatgpt-web.json` | UMP flat envelope with cookies + bearer | `/api/auth/session`, backend account check, `/realtime/vp`, public audio, and Celsius bootstrap succeeded. |
| `~/Library/HTTPStorages/com.openai.chat.binarycookies` | Native ChatGPT BinaryCookies parsed and URL-encoded | `/api/auth/session`, backend account check, `/realtime/vp`, public audio, and Celsius bootstrap succeeded. |
| Atlas auth | Not separately extracted in this pass | Likely compatible when materialized as ChatGPT-web cookies/envelope; Atlas-only local tool auth remains unknown. |
| Direct targeted Keychain items | Prior targeted metadata queries | No ChatGPT/Auth0 token item was found under derived service/account candidates. |

The important operational detail: `curl`-style browser headers worked where default language transports received Cloudflare/HTML `403`. Do not treat that `403` as an auth or entitlement failure until the same request has been retried with the transport guidance below.

### Transport Requirements: Avoiding Edge `403`

ChatGPT web endpoints sit behind Cloudflare bot mitigation that inspects the request's **header set**, not its TLS or HTTP/2 fingerprint. A 2026-05-24 live probe sent the same Codex bearer twice through plain `curl` against `GET /backend-api/accounts/check/v4-2023-04-27`:

- Bare `curl` with `Authorization` only -> `HTTP/2 403 text/html` (Cloudflare interstitial).
- Same `curl` + Chrome 146 desktop `User-Agent`, `sec-ch-ua` triplet, `sec-fetch-*`, `Origin: https://chatgpt.com`, `Referer: https://chatgpt.com/`, `Accept-Language` -> `HTTP/2 200 application/json`.

That is the entire admission gate for ChatGPT-product backend routes. No custom TLS stack, no JA3/JA4 fingerprint match, and no HTTP/2 frame ordering tricks are required: any HTTP client that lets you set headers will pass the edge if the headers are coherent. This contradicts earlier guidance in this file that recommended Warpsock/BoringSSL fingerprinting for admission; that recommendation is retracted.

Minimum browser-like header set (apply on every ChatGPT-backend request):

```
User-Agent: Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/146.0.0.0 Safari/537.36
Accept: */*
Accept-Language: en-US,en;q=0.9
Accept-Encoding: gzip, deflate, br
sec-ch-ua: "Chromium";v="146", "Not.A/Brand";v="99", "Google Chrome";v="146"
sec-ch-ua-mobile: ?0
sec-ch-ua-platform: "macOS"
sec-fetch-dest: empty
sec-fetch-mode: cors
sec-fetch-site: same-origin
Origin: https://chatgpt.com
Referer: https://chatgpt.com/
oai-language: en-US
oai-device-id: <uuid>
```

Add the route-specific identity headers on top: `Authorization: Bearer <codex-oauth>`, `chatgpt-account-id`, and any flow-specific `oai-client-version` / `oai-client-build-number` / `oai-session-id` the path expects.

Operational rules:

- If the response is HTML / Cloudflare-shaped, treat it as `transport_rejected`. Add the missing browser headers and retry; do not classify it as `auth_failed` or `entitlement_blocked` until you've seen a JSON-shaped response from origin.
- If the JSON response is `401`, `403`, `422`, or a model-access error, that is upstream OpenAI/ChatGPT speaking. Classify by body shape: scope failure, invalid request body, Sentinel failure, or model entitlement failure.
- The same client/connection pool can drive prepare, Sentinel, and conversation calls. Cookies and redirects must be handled explicitly because ChatGPT's admission depends on cookie chaining; clients that auto-follow redirects without persisting cookies can break Sentinel.
- For Pro models, a successful `conversation/prepare` may return `{"status":"ok","conduit_token":null}`. Continue the Sentinel and conversation sequence and omit `x-conduit-token`.

Focused validation results (browser-headered `curl` unless noted):

| Surface | Bare-headers symptom | With Chrome-like browser headers | Interpretation |
|---|---|---|---|
| `GET /backend-api/accounts/check/v4-2023-04-27` | HTML `403` Cloudflare | `200 application/json` (Brotli-encoded body) | Codex bearer is accepted on general ChatGPT backend account routes; gate is browser headers only. |
| `GET /backend-api/celsius/ws/user` | HTML `403` Cloudflare | `200` over HTTP/2 with signed `wss://ws.chatgpt.com/p4/ws/user/...?...` URL | Same. |
| Celsius `wss://ws.chatgpt.com/p4/ws/user/...` | n/a | RFC 6455 handshake succeeds, idle channel | Celsius is a passive stream-handoff channel; not a standalone transcript feed. |
| `POST /realtime/vp?dcid=0` (trivial SDP) | HTML `403` Cloudflare | JSON `400 invalid_offer` | Edge passes; SDP body validates separately. |
| `POST /realtime/vp?dcid=0` (Atlas-shaped offer) | STUN timeouts on offers with gathered local candidates or two audio tracks | `201` SDP answer; ICE connects after `setRemoteDescription` | VP works for any client that can build an Atlas-shaped offer; admission is browser-headers, not Warpsock-only. |

Embeddings note: this admission story applies only to ChatGPT-product routes (`chatgpt.com/backend-api/*` and the Cloudflare-fronted ChatGPT voice surfaces). The public OpenAI API origin (`api.openai.com/v1/*`) does **not** care about browser headers; it gates by token scope and project billing. Browser headers will not turn an OpenAI `429 insufficient_quota` on `/v1/embeddings` into a `200`. See "Codex OAuth Compatibility" below.

Do not classify a Cloudflare HTML `403` as a Codex-bearer or entitlement failure. Add the browser headers above and retry before concluding anything.

### Codex OAuth Compatibility

A follow-up pass tested the current local token from `~/.codex/auth.json` without printing it and without sending real ChatGPT web cookies. Observed token scopes were `openid`, `profile`, `email`, `offline_access`, `api.connectors.read`, and `api.connectors.invoke`.

| Surface | Result with Codex OAuth | Interpretation |
|---|---|---|
| ChatGPT web `/backend-api/f/conversation/prepare`, Sentinel prepare/finalize, `/backend-api/f/conversation` | `200` end-to-end baseline SSE; assistant returned the requested marker. | Codex bearer can authenticate a basic ChatGPT-web conversation chain in this environment, even without a real web session cookie. Model entitlements still need validation per slug. |
| Direct malformed `/backend-api/f/conversation/prepare` probe | `422 Invalid conversation body`. | Auth was accepted; body shape was the failure. |
| `GET /backend-api/celsius/ws/user` | `200` JSON with a `websocket_url`. | Codex bearer can bootstrap Celsius WSS metadata; WSS path/query must stay redacted. |
| `POST /realtime/vp?dcid=0` with HAR SDP/session text fields | `201`. | Codex bearer plus the local account header was accepted for the ChatGPT voice WebRTC offer exchange. |
| `POST /realtime/vp?dcid=0` with a live `@roamhq/wrtc` offer | `201` SDP answer; ICE connected after `setRemoteDescription`. | Codex bearer plus an Atlas-shaped pre-gathered offer is enough for VP WebRTC connectivity, posted from any client that sends the Chrome-like browser-header set above. |
| Public `POST /v1/responses` | `401`, missing `api.responses.write`. | Codex bearer is not a public Responses API credential. |
| Public `POST /v1/audio/speech` | `401`, missing `api.model.audio.request`. | Codex bearer is not sufficient for public TTS. |
| Public `POST /v1/audio/transcriptions` with an intentionally empty file | `400 unsupported_value`. | Auth was accepted; the file body was invalid. |
| Public `POST /v1/realtime/calls` with intentionally minimal SDP | `400 invalid_offer`. | Auth was accepted; the SDP body was invalid. |

Validation artifacts for the Codex OAuth pass are archived under `docs/gpt-web-evidence/codex_token_surface_validation_20260513_232411/`. The full ChatGPT-web baseline probe used a temporary `chatgpt-web` envelope containing the Codex bearer and only an inert `oai-did` cookie value.

## Voice, Realtime, TTS, and Whisper

These are separate from the ChatGPT-to-Responses text/tool adapter.

### ChatGPT Web Voice HAR

The HAR at `${HOME}/Downloads/chatgpt-voice-tts.har` contains a successful:

```text
POST https://chatgpt.com/realtime/vp?dcid=0
```

Shape:

- Request status: `201`.
- Auth: bearer plus ChatGPT account/device/session/client headers.
- Body: multipart text fields `sdp` and `session`.
- SDP: audio, video, and datachannel sections; 187 lines in the captured offer.
- Session JSON fields include model slug, requested default model, thinking effort, voice, language, timezone, voice mode, conversation mode, history/training flag, and message-streaming flag.

Safe replay validation with both envelope and native binarycookies auth returned `201` when multipart fields were sent as text fields. Sending them as file upload parts returned `400 invalid_form_data`, which is a request-encoding error, not an auth failure.

A May 15 2026 RE pass (`docs/gpt-web-evidence/atlas_voice_re_20260515/`) found that VP is not Atlas-only. Plain `curl --http2` with a Chrome desktop `User-Agent` and the `sec-ch-ua` / `sec-fetch-*` / Origin / Referer headers passes the admission edge and reaches the SFU. The accepted offer is Atlas-like: one audio m-line, one `oai-events` datachannel, no local candidates needed in the offer. Two-track and gathered-candidate offers still need separate live proof before use. Live production-path proof: start capture in `chatgpt-webrtc` mode, observe `/realtime/vp/start ok`, then apply the SDP answer back to the sidecar WRTC peer.

### Celsius Websocket

Validated:

```text
GET https://chatgpt.com/backend-api/celsius/ws/user
```

Result: `200` JSON with `websocket_url`, scheme `wss`, host `ws.chatgpt.com`, path shape `/p4/ws/user/user-[REDACTED]`, query key `verify`.

Warpsock validation opened the returned WSS URL successfully, but the socket stayed silent during an idle observation window. This confirms Celsius is a passive stream-handoff channel after a ChatGPT conversation emits `stream_handoff`; it is not a standalone transcript feed, speaker-label feed, or diarization surface.

This is ChatGPT conversation stream handoff, not public Realtime API.

### Public Audio and Realtime APIs

With ChatGPT session access tokens from both envelope and native binarycookies:

- `POST https://api.openai.com/v1/audio/speech` returned `200 audio/mpeg`.
- `POST https://api.openai.com/v1/realtime/calls?model=gpt-realtime` with intentionally minimal SDP returned `400 invalid_offer`, proving auth was accepted and the body was invalid.
- `POST https://api.openai.com/v1/audio/transcriptions` with an empty file returned `400 unsupported_value`, proving auth was accepted and the file was invalid.

Public TTS/STT/realtime should remain separate routes. Do not advertise them as `/v1/responses` features.

### WebRTC via api.openai.com/v1/realtime/calls — PROVEN WORKING (May 2026)

Live validation (May 15 2026) proved end-to-end WebRTC connectivity using Codex OAuth:

**Endpoint:** `POST https://api.openai.com/v1/realtime/calls?model=gpt-4o-realtime-preview-2024-12-17`

**Auth:** Codex OAuth bearer token (same token used for Codex CLI, no additional scopes required beyond standard Codex scopes).

**Request:** `Content-Type: application/sdp`, body = raw SDP offer from `RTCPeerConnection.createOffer()`.

**Response:** `201` with SDP answer body (plain text, not JSON).

**ICE:** Connects in < 1 second. Uses the same Azure ICE servers as `chatgpt.com/realtime/vp` (Azure pool rotates; recent IPs include 4.151.200.38, 40.118.236.137, 20.168.48.117). Earlier VP probes saw STUN timeouts; later VP proof connected after the Tauri Warpsock POST returned an answer for an Atlas-shaped pre-gathered offer.

**DataChannel:** `oai-events` datachannel opens after ICE connects. `session.created` event arrives with session object: `{ type: "realtime", model: "gpt-4o-realtime-preview-2024-12-17", output_modalities: ["audio"], ... }`.

**Session update format:** `session.update` on this endpoint requires `session.type = "realtime"`. Parameters `session.modalities`, `session.input_audio`, and the beta-shape top-level `session.input_audio_transcription` are not accepted (return `unknown_parameter` error). The GA shape nests transcription and turn detection under `session.audio.input`:

```json
{
  "type": "session.update",
  "session": {
    "type": "realtime",
    "audio": {
      "input": {
        "transcription": { "model": "gpt-4o-mini-transcribe" },
        "turn_detection": { "type": "server_vad" }
      }
    }
  }
}
```

The accepted shape is mirrored back in the inbound `session.created` payload as `session.audio.input.{format,transcription,turn_detection}` and `session.audio.output.{format,voice,speed}`.

**Contrast with chatgpt.com/realtime/vp:**
- `chatgpt.com/realtime/vp` is a ChatGPT-web voice route: multipart text fields `sdp` and `session`, plus the Chrome-like browser headers documented in "Transport Requirements: Avoiding Edge `403`".
- VP currently connects when the offer is Atlas-shaped: one audio m-line plus one `oai-events` datachannel m-line, with no local ICE candidates in the offer. Two-track, video, or gathered-candidate offers still need separate live proof.
- `api.openai.com/v1/realtime/calls` accepts Codex OAuth directly with raw SDP. It is still a different product route and must not be substituted for VP when validating ChatGPT-web voice behavior.

**For Guidewire:** The sidecar builds the WRTC peer and pre-gathered offer, Tauri calls `chatgpt.com/realtime/vp?dcid=0` with the standard browser-header set, and posts the SDP answer back to the sidecar. Any HTTP client that lets you set the headers in "Transport Requirements" works; no custom TLS stack is required. Detailed RE evidence: `docs/gpt-web-evidence/atlas_voice_re_20260515/`.

**Diarization: not supported.** Live Guidewire sessions against `POST /v1/realtime/calls?model=gpt-realtime-2` with the GA `audio.input.transcription` shape consistently exhibit:

- All user-audio items carry `role: "user"` with `content: [{ "type": "input_audio", "transcript": ... }]`. Items are segmented by server VAD silence boundaries, not by speaker change.
- Two acoustically distinct voices captured in the same turn produce two separate VAD-segmented items, both tagged `role: "user"`, with no per-speaker fields.
- Zero occurrences of `speaker_id`, `participant_id`, `participant`, `diarization`, or any acoustic-fingerprint key on `oai-events` across observed event types (`session.created`, `session.updated`, `conversation.item.added`, `conversation.item.input_audio_transcription.delta/.completed`, `response.*`, `input_audio_buffer.*`, `output_audio_buffer.*`).

`chatgpt.com/realtime/vp` shares the GA realtime session schema family and therefore cannot expose any per-speaker attribution that `/v1/realtime/calls` does not; VP additionally cannot be observed end-to-end from non-Atlas clients because ICE admission is gated at the HTTP layer (`docs/gpt-web-evidence/atlas_voice_re_20260515/network.md` §3, §7).

The verdict for both surfaces is therefore: single-author transcription only. Apps requiring per-speaker labels for mic + system audio must run client-side diarization (e.g. a separate speaker-embedding model on the local capture) before forwarding text to OpenAI realtime, or move to a STT path that exposes speaker labels.

**Validating your tool surface.** Real plugin validation runs through `guidewire-sidecar plugin --plugin-config <path> smoke`. Use `--dry-run` first to enumerate every tool returned by `PluginRegistry::tools_dynamic_with_diagnostics()` and validate local encode/schema/synthesis/session-update inclusion without calling external services. Without `--dry-run`, the command invokes only tools classified as read-only unless `--allow-destructive` is explicitly passed, and writes a `PluginSmokeReport` containing plugin diagnostics plus per-tool verdicts (`Pass | LocalPathFail | UpstreamFail | Skipped | SchemaRejected | Timeout | OversizeOutput | NameCollision`) to `.validation/plugins-smoke-<ts>.json`. Use the realtime console example only to verify the live wire path: `cargo run -p guidewire-openai --example realtime_console -- --fixture` proves function_call -> PluginRegistry -> function_call_output -> response.create against the echo fixture.

### chatgpt.com/realtime/vp?dcid=0 — Atlas RE pass (May 15 2026)

Parallel reverse-engineering passes on `/Applications/ChatGPT Atlas.app` (native binaries, MV3 extensions, `.pak` resources, live HAR analysis, STUN/ICE protocol bytes) tested whether VP ICE admission is gated by an Atlas-only client identity. It is not.

**Current production interpretation:** the VP edge is gated by browser headers (Chrome-desktop `User-Agent`, `sec-ch-ua`, `sec-fetch-*`, `Origin: https://chatgpt.com`, `Referer: https://chatgpt.com/`), not by a TLS or HTTP/2 fingerprint. Any HTTP client that can set those headers passes admission; the production path therefore uses the standard system HTTP client with the header set documented in "Transport Requirements: Avoiding Edge `403`" and keeps the accepted SDP/session shape pinned by tests.

**Confirmed NOT required for the successful production proof:**
- ChatGPT-web cookies (`__Secure-next-auth.session-token`, `__cf_bm`, `_cfuvid`, `oai-adb`)
- `OAI-Device-Id`, `OAI-Session-Id`, `OAI-Client-Build-Number`, `OAI-Client-Version`
- `traceparent`, `tracestate`, `sec-ch-ua-*` triplet
- `X-OpenAI-Target-Path`, `X-OpenAI-Target-Route`
- Non-empty `chatgpt_account_id` in the bearer JWT (empty `ChatGPT-Account-ID` header is accepted)
- Local ICE candidates in the offer (server uses trickle ICE)

**Confirmed required:**
- `Authorization: Bearer <Codex OAuth access_token>` from `~/.codex/auth.json`
- `Content-Type: multipart/form-data; boundary=...`
- Chrome-like browser headers (`User-Agent`, `sec-ch-ua`, `sec-fetch-*`, `Origin`, `Referer`, `Accept-Language`) — see "Transport Requirements: Avoiding Edge `403`". No custom TLS or HTTP/2 fingerprint required.
- Multipart text fields `sdp` (raw SDP offer string) and `session` (JSON envelope)
- Atlas-shaped offer: 1 audio m-line + 1 `oai-events` datachannel m-line

**Side findings (correct, not the gate):**
- Atlas's native voice path uses `https://realtime.chatgpt.com` (LiveKit SFU, `GET-TOKEN`-then-SDP-exchange) via `Aura.framework`'s Swift `VoiceModeTransceiverSignalingAPI.exchange(offerSDP:sessionPayload:)`. Different surface than `/realtime/vp`. Both work.
- Atlas Chromium is launched with `--owl-scoped-user-agent-prefix=` and `--force-fieldtrials=WebRTC-ForceDtls13/Enabled/...`. Neither is required to connect VP from non-Atlas.
- Working HAR `voice_session_id` was uppercase-hex (`0153CF35-1883-...`) and `model_slug=gpt-5-5-pro`, but the server also accepts our existing `gpt-4o-realtime-preview` slug.

**Evidence index:** `docs/gpt-web-evidence/atlas_voice_re_20260515/INDEX.md`. Full RE reports: `native.md`, `extension.md`, `resources.md`, `network.md`, `protocol.md`.

**Live smoke:** `node sidecar/test/_smoke-vp-live.mjs` — exits 0 with `PASS: vp ICE connected` when `~/.codex/auth.json` is present.

### Whisper / Dictation

Static app evidence shows `ChatGPTWhisper`, `API+SpeechToText.swift`, `WhisperRequest`, and `DictationStreamingWebSocketClientProtocol`. This pass did not recover a private ChatGPT Whisper endpoint or accepted private transcription body. Public transcription auth acceptance is proven; private Whisper remains unknown.

## Current UMP ChatGPT Responses Behavior

Source-backed behavior:

- `src/lib/responses-compat/handler.ts` routes `chatgpt-web/*` models to `runChatGptWebResponsesStream`.
- `src/lib/responses-compat/chatgpt-web.ts` converts Responses requests into OpenAI chat-compatible requests for `ChatGptWebAdapter`.
- `src/lib/adapters/chatgpt-web/adapter.ts` owns ChatGPT backend admission, Sentinel, SSE parsing, tool wrapper, and continuation mechanics.
- `src/lib/adapters/chatgpt-web/websocket.ts` owns Celsius URL fetch and WebSocket handoff.

Live UMP validation:

| Probe | Result |
|---|---|
| `GET /v1/models` | `200`, 13 `chatgpt-web/*` IDs. |
| `POST /v1/responses` text, `gpt-5.5-pro` | `200 text/event-stream`; emitted `response.created`, output item/content events, `response.output_text.delta`, and `response.completed`. |
| `POST /v1/responses` text, `gpt-5.5-thinking` | `200 text/event-stream`; same completed event family. |
| `POST /v1/responses` text, `gpt-5.5-instant` | `200 text/event-stream` carrying adapter error after `response.created`; upstream error class reported access denied. |
| `POST /v1/responses` forced function tool | `200 text/event-stream`; emitted function-call argument delta/done and completed with `output_types: ["function_call"]`. |
| `previous_response_id` after completed Pro parent | `200 text/event-stream`; child completed using adapter continuation cache. |
| `previous_response_id` after non-completed/errored parent | `400 invalid_request_error`; previous response ID not found or expired. |
| Focused unit suite | `bun test tests/unit/chatgpt-web-responses.test.ts` passed 9/9. |

### Agentic Tool Loop Over `/backend-api/f/conversation`

Live Pro tool-loop validation on 2026-05-14 used only the local Codex OAuth bearer from `~/.codex/auth.json`, an inert `oai-did` cookie, the `chatgpt-account-id` claim/header, and curl-style browser headers. No ChatGPT session cookie was used. Artifacts:

- `docs/gpt-web-evidence/chatgpt_web_pro_tool_loop_probe_1778811795965/turn1.sse`
- `docs/gpt-web-evidence/chatgpt_web_pro_tool_loop_probe_1778811795965/turn2.sse`
- `docs/gpt-web-evidence/chatgpt_web_pro_tool_loop_probe_1778811795965/summary.json`
- `docs/gpt-web-evidence/chatgpt_web_pro_tool_parent_probe.ts` output showing parented replay against the actual tool-call message.

The verified loop for `gpt-5-5-pro` is:

1. Start a normal ChatGPT-web turn:
   - `POST /backend-api/f/conversation/prepare`.
   - `POST /backend-api/sentinel/chat-requirements/prepare`.
   - Solve Sentinel proof-of-work and optional Turnstile with the UMP Sentinel helpers.
   - `POST /backend-api/sentinel/chat-requirements/finalize`.
   - `POST /backend-api/f/conversation`.
2. For Pro models, `conversation/prepare` may return `{"status":"ok","conduit_token":null}`. This is not a failure. Continue the turn and omit `x-conduit-token` when it is null. A literal `"null"` header also worked in the probe, but omitting it is cleaner.
3. The first user turn must carry the wrapper contract, either directly or through UMP's `chatGptWebToolInstructionPrompt`:
   - Tell the model to call `api_tool.call_tool`.
   - Require JSON content shaped exactly as `{"path":"tool_name","args":{}}`.
   - Send `local_function_names` including `api_tool.call_tool` and the real tool names.
   - Send the OpenAI `tools[]` schema when it is small enough for the ChatGPT body budget; otherwise send the bounded prompt catalog plus `local_function_names`.
4. A real tool call appears as an assistant message whose `recipient` is `api_tool.call_tool` and whose content is the wrapper JSON. The live Pro stream produced:
   - `conversation_id: "6a068395-f6c4-832b-b868-94b418003f4a"` in the stream envelope.
   - tool-call message id `4ce3bd1f-f6d7-4bdb-97ad-fb3d355dc4e0`.
   - assistant `recipient: "api_tool.call_tool"`.
5. The adapter must stop the client-facing stream after the completed tool call, encode `{ conversationId, messageId }` into a `call_cgw_*` id, and surface that as a normal Responses/OpenAI function call. Do not forward later ChatGPT-native connector fallback text to the client.
6. The tool-result turn is another `POST /backend-api/f/conversation`, parented to the tool-call assistant message:

```json
{
  "action": "next",
  "conversation_id": "6a068395-f6c4-832b-b868-94b418003f4a",
  "parent_message_id": "4ce3bd1f-f6d7-4bdb-97ad-fb3d355dc4e0",
  "model": "gpt-5-5-pro",
  "history_and_training_disabled": true,
  "messages": [
    {
      "id": "<new-client-message-id>",
      "author": { "role": "tool", "name": "api_tool.call_tool" },
      "content": {
        "content_type": "execution_output",
        "text": "TOOL_RESULT marker=CHARLIE-991. Final answer must include CHARLIE-991."
      },
      "metadata": {}
    }
  ]
}
```

The focused replay against the actual Pro tool-call message returned `200 text/event-stream`, stayed in the same `conversation_id`, and the assistant answered with the tool marker `CHARLIE-991`. This proves the maintained-history agent loop is not a prompt-only simulation; the tool output is accepted as a native parented ChatGPT conversation turn.

Responses facade mapping:

- The first response should expose `response.output_item.added` / `response.function_call_arguments.*` with `call_id` set to the `call_cgw_*` value.
- A client can continue the tool loop by sending `function_call_output.call_id` equal to that `call_cgw_*` value. If the client sends the Responses item id prefixed as `fc_call_cgw_*`, the current converter unwraps it back to `call_cgw_*`.
- `previous_response_id` is useful for normal text continuations, but same-adapter tool continuations should rely on the encoded `call_cgw_*` id because it already contains the ChatGPT `conversation_id` and parent tool-call `message_id`.
- Foreign tool-call IDs cannot be natively continued. The adapter must flatten them into a fresh prompt or reject in strict mode.

Implementation notes from UMP:

- `encodeChatGptWebToolCallId()` / `decodeChatGptWebToolCallId()` encode and recover `{ conversationId, messageId }`.
- `classifyChatGptWebBranch()` routes same-adapter tool output to the native continuation branch when the final message is a `role: "tool"` message with a decodable `tool_call_id`.
- `buildToolResultConversationBody()` posts the `execution_output` message shown above.
- `ChatGptConversationDeltaParser` must parse `recipient: "api_tool.call_tool"` assistant messages, not just text patches.
- Direct raw callers may see ChatGPT append a native connector "resource not found" hidden tool result after a fake local tool call. A correct adapter intercepts the first completed `api_tool.call_tool` assistant message and returns to the client for real local execution before that fallback text becomes user-visible.

## Responses to ChatGPT Backend Gaps

| Gap | Adapter policy |
|---|---|
| **Public storage** | Current continuation cache is not a public response store. Implement storage or return `404`/`501` for retrieve/list routes. |
| **`store: true`** | Currently ignored. Reject unless storing full public response/input items. |
| **Image input** | Currently dropped. Add upload/vision path or reject. |
| **File input** | Currently dropped. Add file resolver/upload provenance or reject. |
| **Hosted public tools** | Current conversion drops non-function tools. Reject hosted tools unless a bridge exists. |
| **Structured output** | `text.format` ignored. Reject until ChatGPT structured-output behavior is implemented. |
| **Metadata/include/prompt/cache/safety fields** | Mostly ignored. Store or reject explicitly. |
| **Model entitlements** | Catalog can overstate availability. Validate each model/slug and cache failures carefully. |
| **Error stream shape** | Error after `response.created` was observed for an unavailable model. Ensure clients see a public-shaped `response.failed` or clear error frame, not an ambiguous partial stream. |
| **Celsius stream parity** | Bootstrap is proven; full websocket frame replay/open/subscribe beyond URL fetch should still be fixture-captured with redacted path/query. |
| **Atlas local tools** | Names are symbol-confirmed, but request/result JSON and `AgentEventAPI` auth/frame schema are `blocked-live`. |
| **Private Whisper** | Static app modules are known; accepted private endpoint/body is unknown. |

## Evidence Anchors

Primary source files:

- `docs/CODEX-TO-RESPONSES-ADAPTER.md` — structure and Codex comparison baseline.
- `${UNIFIED_MODEL_PROXY_ROOT}/src/lib/responses-compat/handler.ts` — ChatGPT-web route planning.
- `${UNIFIED_MODEL_PROXY_ROOT}/src/lib/responses-compat/chatgpt-web.ts` — Responses-to-chat conversion and event projection.
- `${UNIFIED_MODEL_PROXY_ROOT}/src/lib/adapters/chatgpt-web/adapter.ts` — ChatGPT backend request sequence, body builders, Sentinel, tool mapping, SSE parser.
- `${UNIFIED_MODEL_PROXY_ROOT}/src/lib/adapters/chatgpt-web/websocket.ts` — Celsius WebSocket URL and handoff.
- `${UNIFIED_MODEL_PROXY_ROOT}/src/lib/auth/chatgpt-web.ts` — credential resolver.
- `${UNIFIED_MODEL_PROXY_ROOT}/docs/chatgpt-web-gpt-5.5.md` — model/tool/Responses facade note.
- `${UNIFIED_MODEL_PROXY_ROOT}/docs/chatgpt-atlas-agent-tools.md` — Atlas local-tool and AgentEventAPI status.
- `${UNIFIED_MODEL_PROXY_ROOT}/scripts/chatgpt-cookie-probe.ts` — native BinaryCookies bootstrap flow.
- `${HOME}/Downloads/chatgpt-voice-tts.har` — ChatGPT web voice WebRTC offer exchange.

Validation artifacts:

- `docs/gpt-web-evidence/INDEX.md` — index of all archived GPT-web evidence moved from `/tmp`.
- `docs/gpt-web-evidence/chatgpt_to_responses_validation_20260513_230751/live_ump_chatgpt_responses_validation.json`
- `docs/gpt-web-evidence/chatgpt_to_responses_validation_20260513_230751/text_anomaly_validation.json`
- `docs/gpt-web-evidence/chatgpt_to_responses_validation_20260513_230751/continuation_validation.json`
- `docs/gpt-web-evidence/chatgpt_to_responses_validation_20260513_230751/conversion_loss_validation.json`
- `docs/gpt-web-evidence/chatgpt_to_responses_validation_20260513_230751/har_voice_shape_validation.json`
- `docs/gpt-web-evidence/chatgpt_to_responses_validation_20260513_230751/prior_session_parse_and_vp_textfield_validation.json`
- `docs/gpt-web-evidence/chatgpt_to_responses_validation_20260513_230751/prior_public_api_session_tokens_validation.json`
- `docs/gpt-web-evidence/chatgpt_to_responses_validation_20260513_230751/prior_celsius_ws_probe_validation.json`

## Validation Passes

This note reflects four passes:

1. Read `CODEX-TO-RESPONSES-ADAPTER.md` and mirrored its route/field/event/state framing for ChatGPT web.
2. Inspected UMP ChatGPT-web auth, adapter, Responses facade, tests, and Atlas docs.
3. Ran live sanitized probes for `/v1/models`, ChatGPT-web `/v1/responses` text/tool/continuation, HAR voice shape, prior voice replay, public TTS/STT/realtime auth acceptance, and Celsius WSS bootstrap.
4. Ran code-level conversion probes proving current silent drops for image/file/hosted tools/state fields, plus the focused `chatgpt-web-responses` unit suite.

Open issues that still need live capture:

- Full Atlas-native local-tool call/result frames.
- Atlas `AgentEventAPI` websocket endpoint/auth/frame schema.
- Full Celsius websocket subscribe/open transcript with query redacted.
- Private ChatGPT Whisper endpoint/body, if distinct from public `/audio/transcriptions`.
- Strict error mapping for partial streams where upstream fails after `response.created`.
