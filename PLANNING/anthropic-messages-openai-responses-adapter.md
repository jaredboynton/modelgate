# Anthropic Messages to OpenAI Responses Adapter Plan

Date: 2026-05-13
Target repo: `/Users/jaredboynton/__devlocal/amp-research/unified-model-proxy-v2`

## Goal

Implement an adapter for this UMP v2 use case:

```text
Anthropic Messages request
  -> OpenAI Responses request
  -> Codex Responses WSS/HTTP response events
  -> Anthropic Messages JSON or Anthropic SSE
```

This is for Amp clients that speak Anthropic Messages but need to reach a
Codex/OpenAI Responses model such as `openai:gpt-5.5`. Default Anthropic model
traffic must keep using Bedrock Mantle. This adapter only activates when the
model resolves to `Provider::Codex`.

## Local Context Read

- `plans/handoffs/HANDOFF_ump-v2-ralplan-ultrawork-plan_2026-05-13.md`
  existed under `unified-model-proxy-v2/`, not at the path missing that
  directory segment.
- `PLANNING/v2-plan.md` is the companion implementation plan.
- `PLANNING/codex.md` defines the Codex Responses transport contract.
- `PLANNING/bedrock.md` confirms Bedrock Mantle can receive Anthropic Messages
  without body-shape translation.
- Current code is no longer planning-only. `Cargo.toml`, `src/`, `tests/`, and
  `target/` exist. This plan assumes the scaffold/stub state currently on disk.
- `.env` was not read and is not needed for this plan.

## Current Local Gap

Relevant current files:

- `src/route/messages.rs`: accepts only `Provider::Bedrock` models. It rejects
  `openai:gpt-*` before Codex credentials are checked.
- `src/route/chat.rs`: sends GPT chat bodies to Codex, but the converter only
  renames `messages` to `input`. That is not the canonical Responses item
  shape required by `PLANNING/codex.md`.
- `src/upstream/codex.rs`: prepares Codex Responses payloads and headers, but
  currently returns the prepared `response.create` JSON instead of opening the
  upstream transport.
- `src/sse/filter.rs` and `src/sse/splice.rs`: already contain useful Codex SSE
  filtering and `response.completed` output splicing primitives.
- `tests/unit_codex.rs` and `tests/integration_routes.rs`: lock current Codex
  body defaults and routing/auth behavior.

## Core Decision

Add a narrow adapter module, not a new provider framework.

Recommended module:

```text
src/adapter/
  mod.rs
  anthropic_responses.rs
tests/
  unit_anthropic_responses.rs
```

Export `pub mod adapter;` from `src/lib.rs`.

Route behavior:

- `/api/provider/anthropic/v1/messages` and `/v1/messages`
  - `Provider::Bedrock`: current Bedrock Mantle path.
  - `Provider::Codex`: translate Anthropic Messages to Responses and call the
    Codex upstream.
  - everything else: `model_not_supported`.
- `/api/provider/anthropic/v1/messages/count_tokens`
  - `Provider::Bedrock`: current local approx.
  - `Provider::Codex`: same local approx for v0.1 unless Amp proves it needs
    upstream-specific token counting.
- `/api/provider/openai/v1/chat/completions`
  - Replace the naive `messages -> input` conversion with shared canonical
    chat-to-Responses helpers, or route through a common internal message IR.

Do not reroute existing `anthropic/claude-*` aliases to Codex. The adapter
exists for Anthropic-shaped clients using explicit OpenAI/Codex aliases.

## Reference Implementations

### Primary Reference: LiteLLM

Repo: https://github.com/BerriAI/litellm
License: MIT outside `enterprise/`.

Relevant files:

- `litellm/llms/anthropic/experimental_pass_through/responses_adapters/handler.py`
- `litellm/llms/anthropic/experimental_pass_through/responses_adapters/transformation.py`
- `litellm/llms/anthropic/experimental_pass_through/responses_adapters/streaming_iterator.py`

What to borrow:

- Direct Anthropic `/v1/messages` to OpenAI Responses routing.
- Request mapping:
  - `system` -> `instructions`
  - user text/image -> Responses `message` with `input_text` / `input_image`
  - user `tool_result` -> top-level `function_call_output`
  - assistant text -> Responses `message` with `output_text`
  - assistant `tool_use` -> top-level `function_call`
  - Anthropic tools -> Responses `function` tools
  - Anthropic `thinking` -> Responses `reasoning`
  - metadata user id -> Responses `user`
- Response mapping:
  - Responses output text -> Anthropic `text`
  - Responses `function_call` -> Anthropic `tool_use`
  - Responses reasoning summaries -> optional Anthropic `thinking`
  - incomplete status -> `stop_reason=max_tokens`
- Streaming state machine:
  - Responses `output_item.added` creates Anthropic content block indexes.
  - text deltas become `content_block_delta` with `text_delta`.
  - function arguments deltas become `input_json_delta`.
  - completion emits `message_delta` and `message_stop`.

### Codex-Specific Reference: CC-Adapter

Repo: https://github.com/Jakevin/CC-Adapter
License status: no `LICENSE` file found in the root listing. Treat as
reference-only unless permission/license is clarified.

Relevant files:

- `src/convert/request_responses.rs`
- `src/convert/response_responses.rs`
- `src/server.rs`

What to borrow as behavior, not code:

- Codex response recovery when `response.completed.output` is empty but
  `response.output_item.done` events contain the actual items.
- Visible-text fallback extraction from completed payloads and SSE blobs.
- Tool-use recovery from `function_call` / `tool_call` items in either the
  completed response or `output_item.done`.
- Keepalive behavior for Anthropic SSE clients while buffering Codex output.

Conflict to resolve locally:

- CC-Adapter warns against requesting `reasoning.encrypted_content`.
- `PLANNING/codex.md` and current `src/upstream/codex.rs` require
  `reasoning.encrypted_content`.
- Keep the current UMP Codex decision unless a local fixture or live capture
  proves it hides visible output for this adapter. Add tests for both paths
  before changing this.

### Broad Converter Reference: llm-rosetta

Repo: https://github.com/Oaklight/llm-rosetta
License: MIT.

Relevant files:

- `src/llm_rosetta/converters/anthropic/converter.py`
- `src/llm_rosetta/converters/openai_responses/converter.py`
- `tests/converters/anthropic/test_converter.py`

What to borrow:

- Use explicit stream state, not stateless event rewriting.
- Fix or flag orphaned tool calls/results before provider conversion.
- Maintain a provider-independent mental model for text, image, tool call, tool
  result, reasoning, usage, and finish reason, even if UMP v2 does not add a
  full IR layer.

### Mature Client Reference: Vercel AI SDK

Repo: https://github.com/vercel/ai
License: Apache-2.0.

Relevant files:

- `packages/openai/src/responses/convert-to-openai-responses-input.ts`
- `packages/openai/src/responses/openai-responses-language-model.ts`
- `packages/anthropic/src/anthropic-language-model.ts`

What to borrow:

- OpenAI Responses item handling for assistant tool calls, tool results,
  images/files, reasoning items, and provider metadata.
- The policy of including `reasoning.encrypted_content` when `store=false` for
  reasoning models is useful supporting evidence for UMP's current Codex plan.
- Warning-driven handling for unsupported parameters instead of silent lossy
  conversion.

### Official Schemas

- OpenAI Responses create/reference:
  https://developers.openai.com/api/reference/resources/responses/methods/create
- OpenAI Responses streaming events:
  https://developers.openai.com/api/reference/resources/responses
- OpenAI tool guidance:
  https://developers.openai.com/api/docs/guides/function-calling
- Anthropic Messages API:
  https://docs.anthropic.com/en/api/messages
- Anthropic streaming Messages:
  https://platform.claude.com/docs/en/build-with-claude/streaming
- Anthropic tool use:
  https://platform.claude.com/docs/en/agents-and-tools/tool-use/define-tools

## Request Mapping

Implement these as pure functions first.

Suggested public functions:

```rust
pub fn anthropic_messages_to_responses(
    body: serde_json::Value,
) -> AppResult<serde_json::Value>;

pub fn chat_completions_to_responses(
    body: serde_json::Value,
) -> AppResult<serde_json::Value>;
```

Use `serde_json::Value` at module boundaries for v0.1, with small typed helper
enums for content blocks and stream events only where they reduce bugs. Avoid a
large IR until the Google fallback and chat path prove it is worth the surface.

### Top-Level Fields

| Anthropic Messages | Responses | Rule |
|---|---|---|
| `model` | `model` | Preserve requested alias for `upstream::codex::prepare_responses_body`, which maps it to upstream id. |
| `system` string | `instructions` | Use as-is if nonblank. |
| `system` text blocks | `instructions` | Join text blocks with blank lines. |
| no system | `instructions` | Let `prepare_responses_body` default, or set `"You are a helpful assistant."`. |
| `max_tokens` | `max_output_tokens` | Correct public Responses mapping. Codex WSS prep strips it later. |
| `temperature` | `temperature` | Preserve unless a model-specific test says Codex rejects it. |
| `top_p` | `top_p` | Preserve unless a model-specific test says Codex rejects it. |
| `stop_sequences` | `stop` | Map only if current Responses schema accepts it; otherwise return a typed unsupported warning/error. |
| `metadata.user_id` | `user` | Preserve when present and string-like. |
| `stream` | `stream` | Preserve caller intent at route level. Codex WSS prep drops it because WSS is always streaming. |
| `thinking` | `reasoning` | Map enabled/adaptive thinking to effort plus summary. Existing Codex prep defaults missing reasoning. |
| `output_config.format` / `output_format` | `text.format` | Support JSON schema only after a fixture is added. |
| `context_management` | `context_management` | Pass through only known compaction shape; reject unknown shapes for v0.1. |

### Message Content

| Anthropic block | Responses input item |
|---|---|
| user string content | `{type:"message", role:"user", content:[{type:"input_text", text}]}` |
| user `text` block | append `{type:"input_text", text}` to current user message item |
| user `image` base64 | append `{type:"input_image", image_url:"data:<media_type>;base64,<data>"}` |
| user `image` URL | append `{type:"input_image", image_url:<url>}` |
| user `tool_result` | top-level `{type:"function_call_output", call_id:<tool_use_id>, output:<text>}` |
| assistant `text` | `{type:"message", role:"assistant", content:[{type:"output_text", text}]}` |
| assistant `tool_use` | top-level `{type:"function_call", call_id:<id>, name, arguments:<json string>}` |
| assistant `thinking` | drop for v0.1 inbound history unless a fixture proves Codex needs prior visible reasoning items |

Keep ordering. If a user message mixes text/images and tool results, emit the
message item first, then each `function_call_output` item in original order.

Tool-result text extraction:

- string content: use as-is.
- list of text blocks: concatenate text in order.
- image/file blocks inside tool results: reject for v0.1 with a typed error
  unless Amp emits them in a fixture.
- `is_error=true`: preserve text and add a test before deciding whether to
  prefix with `Error: `. CC-Adapter prefixes; LiteLLM preserves more directly.

Image safety:

- Never log raw base64 data.
- Failure capture must redact `data:*;base64,...` values.
- Unit tests should assert redaction on adapter errors and route failures.

### Tools

| Anthropic tool | Responses tool |
|---|---|
| `{name, description, input_schema}` | `{type:"function", name, description, parameters: input_schema}` |
| `web_search` or Anthropic-hosted tools | unsupported in v0.1 unless a mapping to Responses hosted tools is deliberately added |

Tool names:

- Anthropic and OpenAI function tools both cap names at 64 chars in common
  implementations. UMP should reject overlong names with `400 bad_request`
  rather than silently truncating, unless Amp emits such names and requires a
  compatibility mapping.

Tool choice:

| Anthropic `tool_choice` | Responses `tool_choice` |
|---|---|
| omitted | omitted or `auto` |
| `{type:"auto"}` | `auto` / equivalent official Responses shape |
| `{type:"any"}` | `required` / equivalent official Responses shape |
| `{type:"tool", name}` | function choice for that name |
| `{type:"none"}` | `none` if official Responses supports it, otherwise omit tools or reject |

Use the current official Responses schema for exact object/string shape. Add
fixtures before normalizing aliases.

## Response Mapping

Suggested public functions:

```rust
pub fn responses_json_to_anthropic_message(
    response: serde_json::Value,
    requested_model: &str,
) -> AppResult<serde_json::Value>;

pub fn responses_sse_to_anthropic_sse(
    sse: impl Stream<Item = AppResult<Bytes>>,
    requested_model: String,
) -> impl Stream<Item = AppResult<Bytes>>;
```

### Non-Streaming JSON

Map a completed Responses object into Anthropic Messages:

| Responses output | Anthropic content |
|---|---|
| `message.content[].output_text` | `{type:"text", text}` |
| `function_call` | `{type:"tool_use", id: call_id or id, name, input: JSON.parse(arguments)}` |
| `reasoning.summary[].text` | optional `{type:"thinking", thinking, signature:null}` |

Default for v0.1: do not emit thinking blocks unless inbound Anthropic
`thinking` requested them or a config flag enables it. Amp's normal Anthropic
path expects final text and tool calls, not necessarily visible reasoning.

Stop reason:

- any tool call emitted -> `tool_use`
- `status=incomplete` or incomplete reason `max_output_tokens` -> `max_tokens`
- `status=completed` with no tool call -> `end_turn`
- `status=failed` or top-level `error` -> structured proxy error, not a fake
  Anthropic success message

Usage:

- `input_tokens` <- Responses `usage.input_tokens`
- `output_tokens` <- Responses `usage.output_tokens`
- `cache_read_input_tokens` <- Responses
  `usage.input_tokens_details.cached_tokens` when present
- Preserve zero values when absent; do not invent totals as content tokens.

Model field:

- Return the original requested model alias in Anthropic responses. Keep the
  upstream model id in logs/failure capture only. This avoids surprising Amp
  selectors that key on requested model id.

### Streaming SSE

Build a stateful translator. Do not implement this as raw string replacement.

State to maintain:

- Anthropic message id, model, and started flag.
- Responses `item_id` or `output_index` to Anthropic `content_block.index`.
- Function call `item_id` to `call_id`, name, block index, and accumulated
  argument string.
- Text block indexes for message items.
- Usage from the latest `response.completed` / usage event.

Event mapping:

| Responses event | Anthropic SSE event |
|---|---|
| `response.created` | `message_start` |
| `response.output_item.added` message | `content_block_start` text |
| `response.output_item.added` function_call | `content_block_start` tool_use with empty input |
| `response.output_item.added` reasoning | optional `content_block_start` thinking |
| `response.output_text.delta` | `content_block_delta` with `text_delta` |
| `response.function_call_arguments.delta` | `content_block_delta` with `input_json_delta` |
| `response.reasoning_summary_text.delta` | optional `thinking_delta` |
| `response.reasoning_text.delta` | optional `thinking_delta` |
| `response.output_item.done` | `content_block_stop` for the mapped block |
| `response.completed` | `message_delta`, then `message_stop` |
| `response.failed` or `error` | Anthropic error event or HTTP 502 if headers not sent |

If Codex omits `response.created`, synthesize `message_start` before the first
content block. If Codex emits completed output with empty `output`, use existing
`src/sse/splice.rs` behavior and event-held output items before converting.

Send Anthropic `ping` events while buffering if the implementation cannot stream
converted deltas immediately. Prefer true streaming conversion, but keep a
15-second keepalive fallback because Claude-code-like clients can idle-timeout.

## Codex Prep Compatibility

The adapter should emit standard Responses shape. Codex-specific mutations stay
in `src/upstream/codex.rs`:

- endpoint is `wss://chatgpt.com/backend-api/codex/responses`
- headers include `ChatGPT-Account-Id`, `originator: codex_cli_rs`, and
  `OpenAI-Beta: responses_websockets=2026-02-06`
- `store=false`
- drop `stream`, `stream_options`, `background`, `max_output_tokens`,
  `max_tokens`
- default `instructions`
- default/add `reasoning.summary=auto`
- include `reasoning.encrypted_content`

Keep this split so a future public OpenAI `/v1/responses` provider can reuse
the adapter without Codex backend quirks.

## Implementation Lanes

### Lane 1: Adapter Skeleton and Fixtures

Files:

- `src/adapter/mod.rs`
- `src/adapter/anthropic_responses.rs`
- `tests/unit_anthropic_responses.rs`
- `tests/fixtures/anthropic_responses/*.json`

Tasks:

- Add pure request/response translation functions.
- Add fixture helpers that load JSON bodies without real credentials.
- Add redaction fixture with base64 image data.

Acceptance:

- Unit tests run without credentials or network.
- No route code changes yet.

### Lane 2: Request Translation

Tasks:

- Implement system, text, image, tool_use, tool_result, tools, tool_choice,
  thinking, metadata, and generation parameter mapping.
- Reject unsupported Anthropic-hosted tools and ambiguous tool-result content
  with typed errors.
- Share canonical Responses input builder with chat conversion or make chat use
  the same helper for message/tool content.

Acceptance tests:

- `UT-AM2RESP-SIMPLE`: system + one user text message.
- `UT-AM2RESP-IMAGE`: base64 image becomes data URL and logs redact it.
- `UT-AM2RESP-TOOLS`: assistant `tool_use` and user `tool_result` become
  `function_call` and `function_call_output`.
- `UT-AM2RESP-TOOL-CHOICE`: `auto`, `any`, `tool`, `none` mappings.
- `UT-AM2RESP-THINKING`: Anthropic thinking maps to Responses reasoning.
- `UT-CHAT2RESP-CANONICAL`: chat completions no longer produce plain
  `{role, content:"..."}` items for Codex.

### Lane 3: Response JSON Translation

Tasks:

- Convert Responses JSON into Anthropic Messages JSON.
- Preserve requested model alias.
- Parse function call arguments defensively; invalid JSON becomes `{}` and logs
  a warning without leaking raw tool data.
- Implement output recovery from `response.output_item.done` fixtures when
  completed output is empty.

Acceptance tests:

- `UT-RESP2AM-TEXT`: output message becomes Anthropic text content.
- `UT-RESP2AM-TOOL`: function call becomes Anthropic `tool_use`.
- `UT-RESP2AM-MIXED`: text plus tool call preserves both blocks and returns
  `stop_reason=tool_use`.
- `UT-RESP2AM-INCOMPLETE`: incomplete response returns `stop_reason=max_tokens`.
- `UT-RESP2AM-USAGE`: token usage maps correctly, including cached input.

### Lane 4: Streaming Translation

Tasks:

- Add stateful Responses SSE to Anthropic SSE translator.
- Reuse or extend `src/sse/filter.rs` and `src/sse/splice.rs`.
- Handle event blocks with multiple `data:` lines.
- Emit pings only if buffering prevents immediate converted output.

Acceptance tests:

- `UT-RESP-SSE-TEXT`: created, text delta, completed -> valid Anthropic SSE.
- `UT-RESP-SSE-TOOL`: function_call added, argument deltas, done -> Anthropic
  `tool_use` block with `input_json_delta`.
- `UT-RESP-SSE-REASONING`: reasoning deltas are dropped by default or emitted
  only when configured.
- `UT-RESP-SSE-EMPTY-COMPLETED`: output items from events are used when
  completed output is empty.
- `UT-RESP-SSE-ERROR`: failed/error event becomes structured error.

### Lane 5: Route Integration

Files:

- `src/route/messages.rs`
- `src/route/chat.rs`
- `src/upstream/codex.rs`
- `src/router.rs` only if route-level streaming needs a different response type
- `tests/integration_routes.rs`

Tasks:

- Route Anthropic Messages requests with `Provider::Codex` through the adapter.
- Keep Bedrock route unchanged for `anthropic/claude-*`.
- Make `count_tokens` accept Codex aliases with the local approx.
- Preserve query strings such as `?beta=true` only where needed by Bedrock;
  Codex adapter does not need Anthropic beta parameters.
- Ensure missing Codex credentials return `401 missing_credential`, proving the
  request reached the Codex branch.

Acceptance tests:

- `IT-MESSAGES-BEDROCK-STILL-BEDROCK`: current Anthropic alias behavior stays.
- `IT-MESSAGES-CODEX-AUTH-GATE`: `/v1/messages` with `openai:gpt-5.5` reaches
  Codex branch and fails on missing Codex credential.
- `IT-COUNT-TOKENS-CODEX`: `/messages/count_tokens` accepts
  `openai:gpt-5.5` and returns a local estimate.
- `IT-CHAT-CANONICAL-CODEX`: GPT chat route uses canonical Responses input.

### Lane 6: Upstream Transport Hook

This lane may already overlap with the broader Codex transport work in
`PLANNING/v2-plan.md`. Do not duplicate it if another plan owns it.

Tasks:

- Make `upstream::codex::responses` able to return either raw Responses JSON,
  raw Responses SSE, or a stream handle that the adapter can translate.
- Keep Codex WSS implementation in Specter RFC 6455 path.
- Keep HTTP fallback behind the existing fallback latch decision.

Acceptance:

- Adapter tests use fixtures.
- Transport integration tests use mock RFC 6455 WSS, not live ChatGPT, unless a
  live-smoke lane explicitly opts in.

### Lane 7: Documentation and Copy Hygiene

Tasks:

- Document borrowed algorithms and source links in the module header or a
  `NOTICE`/planning note.
- Prefer MIT LiteLLM and MIT llm-rosetta for any copied structure.
- Apache-2.0 Vercel AI code can inform design; if copied, preserve required
  attribution.
- Do not copy CC-Adapter code until license/permission is resolved.

Acceptance:

- Every nontrivial borrowed algorithm has a source comment or plan reference.
- No unlicensed code copied from CC-Adapter.

## Validation Commands

Do not run these while only drafting the plan. Run them during implementation:

```bash
cargo fmt --check
cargo test unit_anthropic_responses
cargo test unit_codex
cargo test unit_sse
cargo test integration_routes
```

If implementation touches shared routing or upstream transport, also run:

```bash
cargo test
```

## Risks

- **Codex Responses drift**: The ChatGPT internal Codex endpoint can differ from
  public OpenAI Responses. Keep Codex mutations in `upstream::codex` and use
  fixtures based on captured events.
- **Reasoning include conflict**: CC-Adapter and UMP notes disagree about
  `reasoning.encrypted_content`. Test both before changing UMP's current
  include behavior.
- **Tool-call pairing**: Anthropic clients are strict about `tool_use` /
  `tool_result` ordering. Add explicit orphan/mismatch tests before accepting
  multi-turn tool histories.
- **Streaming header timing**: Once Anthropic SSE headers are sent, upstream
  errors must become SSE error events, not HTTP status changes.
- **Image payload leakage**: Base64 must be redacted in adapter errors,
  tracing, and failure capture.
- **Scope creep**: Do not add Anthropic-hosted tools, OpenAI direct provider,
  broad IR, thread storage, or public image endpoints as part of this adapter.

## Definition of Done

- Anthropic Messages with `anthropic/claude-*` still routes to Bedrock Mantle.
- Anthropic Messages with `openai:gpt-*` routes through the adapter to Codex.
- Chat Completions GPT fallback uses the same canonical Responses item mapping.
- Non-streaming Responses fixtures convert back to Anthropic Messages JSON.
- Streaming Responses fixtures convert to Anthropic SSE with correct block
  indexes and stop reasons.
- Base64 image data is redacted in logs/failure captures.
- Tests listed in the lanes pass without live credentials.
- No unlicensed CC-Adapter code is copied.
