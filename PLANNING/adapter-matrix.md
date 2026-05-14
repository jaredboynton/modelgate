# Adapter Matrix Architecture

Goal: make provider/model routing config-driven while keeping every wire-format
conversion explicit, tested, and reversible.

## Canonical Hub

Use OpenAI Responses as the hub format.

Reasons:

- Codex CLI only supports `wire_api = "responses"`.
- Amp already reaches OpenAI-compatible facade routes for `/responses` and
  `/chat/completions`.
- Responses can represent text, multimodal inputs, tool calls, tool outputs,
  reasoning controls, and hosted-tool metadata better than chat-only formats.

Provider-native formats stay at the edge:

- Anthropic Messages
- OpenAI Responses
- OpenAI Chat Completions
- Google Gemini `generateContent`
- OpenAI Images
- Future provider-specific formats

## Current Edges

| Direction | Module | Status |
|---|---|---|
| Anthropic Messages request -> Responses request | `adapter::anthropic_responses::anthropic_messages_to_responses` | implemented |
| Responses request -> Anthropic Messages request | `adapter::anthropic_responses::responses_to_anthropic_messages` | implemented |
| Chat Completions request -> Responses request | `adapter::anthropic_responses::chat_completions_to_responses` | implemented |
| Anthropic Messages response -> Responses response | `adapter::anthropic_responses::anthropic_message_to_responses_json` | implemented |
| Responses response -> Anthropic Messages response | `adapter::anthropic_responses::responses_json_to_anthropic_message` | implemented |
| Anthropic SSE -> Responses SSE | `adapter::anthropic_responses::AnthropicSseStreamTranslator` | implemented |
| Responses SSE -> Anthropic SSE | `adapter::anthropic_responses::responses_sse_to_anthropic_sse_text` | implemented |
| Responses request -> Google `generateContent` request | `adapter::google_responses::responses_to_google_generate_content` | implemented |
| Google `generateContent` response -> Responses response | `adapter::google_responses::google_generate_content_to_responses` | implemented |
| Google `generateContent` SSE -> Responses SSE | `adapter::google_responses::GoogleResponsesSseTranslator` | implemented |
| Google direct response shaping for Gemini vs Vertex callers | `adapter::google_generate_content` | implemented |

## Known Gaps

These are intentional gaps, not implicit fallbacks:

- Google `generateContent` request -> Responses request. Needed before a
  Gemini URL/model can be routed to Codex by config alone.
- OpenAI Images request/response <-> Responses hosted image tool. Needed before
  `gpt-image-2` can replace Gemini image generation.
- Public OpenAI Responses auth/transport. Current OpenAI-shaped traffic routes
  to explicit provider dispositions only; Codex-backed GPT Responses/chat flows
  use Codex/ChatGPT OAuth, not public OpenAI API keys.
- Provider-specific hosted tools. Unsupported tools must fail closed until each
  semantic mapping is explicit.
- Non-text multimodal output normalization across providers. Do not route image,
  audio, or binary outputs through text-only adapters.
- `public-openai` API-key provider. This is a disabled-by-default follow-up
  provider, not a fallback when Codex auth, route mapping, or model disposition
  fails.

## OpenAI-Compatible Facade Contract

OpenAI-shaped routes are compatibility facades. They are not blanket public
OpenAI API parity and they are not blanket Codex backend routes.

Route/model disposition:

- `/v1/responses` and `/v1/chat/completions` resolve the requested model
  through the aggregate route table. Codex GPT models go to Codex Responses,
  Claude models go to Bedrock Messages through adapters, and Gemini models go
  to Google `generateContent` through adapters where implemented.
- `/api/provider/openai/v1/responses` and
  `/api/provider/openai/v1/chat/completions` are OpenAI-shaped facade
  entrypoints. Codex-backed GPT dispositions use Codex OAuth; other model
  dispositions still follow explicit route/provider rules. They validate
  model/provider fit before upstream dispatch.
- `/api/provider/openai/v1/models` reads the live Codex catalog from the
  ChatGPT Codex backend using Codex OAuth headers.
- `/v1/models` remains aggregate and local: static allowlist plus configured
  hot-route models.

Unsupported public-only OpenAI features fail closed until each semantic mapping,
credential source, and test path is explicit.

## Routing Contract

Hot config routes select a target provider/model. The selected target must have
both request and response edges for the inbound route format:

```json
{
  "routes": [
    {
      "source": { "model": "gemini-3-pro-image", "format": "google_generate_content" },
      "target": { "provider": "codex", "model": "gpt-image-2", "format": "responses" }
    }
  ]
}
```

That example is a desired route, not an implemented route. It remains blocked
until `google_generate_content -> responses` request translation and
Responses-hosted image generation response shaping exist.

## Adding an Adapter

1. Add request conversion and response conversion together.
2. Add streaming conversion if either side can stream.
3. Reject unsupported semantics before credential lookup.
4. Preserve model identity visible to the caller; only target payloads use the
   upstream model.
5. Add unit tests for request conversion, response conversion, streaming, and
   unsupported semantics.
6. Add one route-level test proving the selected config path reaches the
   intended provider gate without touching real credentials.
7. Update this matrix and the README config example only after tests pass.

## Failure Rule

Never silently fall back across providers for configured routes. If an adapter
edge is missing, return `model_not_supported` or `invalid_request` with the
missing edge named in the message. Provider fallback is only allowed where the
route owns a documented compatibility behavior and tests cover both paths.
