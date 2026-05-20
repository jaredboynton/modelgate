# Provider Boundaries

Golden rule: provider choice is explicit, catalog-driven, and never inferred from missing credentials or upstream availability.

## Layer Responsibilities

- `src/model_alias.rs` owns the model allowlist and maps accepted IDs to `Provider` plus upstream model ID.
- `src/route/**` owns HTTP shape, request parsing, model validation, and thin delegation.
- `src/auth/**` owns credential loading, refresh, signing helpers, and private writes.
- `src/upstream/**` owns provider-specific request/response forwarding and transport calls.
- `src/adapter/**`, when present, owns format translation between client API shapes and provider API shapes.

## Provider Rules

- Bedrock routes Anthropic-shaped traffic through Bedrock Runtime.
- Codex routes OpenAI Responses and Codex-backed OpenAI facade traffic through Codex/ChatGPT OAuth.
- Google routes Gemini traffic through direct Google API-key auth and only uses documented fallback paths.
- Public OpenAI API-key proxying is not a default provider boundary in this Rust proxy.
- `supports_websockets` in Codex CLI is provider-wide, so UMP owns mixed downstream WebSocket routing instead of requiring Codex-only WebSocket profiles.
- A downstream Responses WebSocket connection may switch provider/model between independent post-terminal `response.create` events. That does not relax upstream provider isolation.
- `previous_response_id` continuations are connection-local and exact-match for the stored route/model fingerprint; they must not bridge provider/auth boundaries.
- Codex WSS, Bedrock HTTP/SSE, and Google HTTP/SSE calls keep separate auth sources, request adapters, and failure handling.

## Failure Rules

- Unknown model means `model_not_supported`; never try the next provider.
- Missing provider credentials means `missing_credential` or Codex `invalid_api_key`; never reroute.
- Unsupported feature means explicit unsupported error; never strip fields silently to make a provider accept a request.
- A second `response.create` during an in-flight WebSocket response means `response_already_in_flight`; never race two upstream providers on one downstream socket.
- Provider-specific quirks stay in the provider or adapter owner, not in `router.rs`.

## Review Checklist

- Did a new model land in `src/model_alias.rs` with provider and upstream ID tests?
- Did route code stay thin, or did it start translating provider internals?
- Did auth code remain independent from route/upstream/adapter decisions?
- Does every provider branch fail closed when its credential or capability is absent?
