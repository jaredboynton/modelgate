# WebSocket Responses

Golden rule: `/v1/responses` WebSocket handling must fail closed with JSON error events clients can read before any close frame matters.

## Contract

- WebSocket Responses is Codex-capable only after the first frame routes to `ResponsesRoute::CodexResponses`.
- Non-Codex Responses routes must reject instead of tunneling to Codex.
- One socket owns one route/model fingerprint. Model or provider switches on later frames return an error event.
- Unsupported parseable events return an error event and keep the socket usable when possible.
- Upstream failures are reported downstream as top-level JSON error events before close handling.

## Error Event Shape

Every downstream WebSocket error event uses this shape:

```json
{
  "type": "error",
  "status": 400,
  "error": {
    "code": "model_not_supported",
    "message": "..."
  }
}
```

Required fields:

- top-level `type = "error"`
- numeric top-level `status`
- nested `error.code`
- nested `error.message`

## Specter Boundary

- `specter` owns RFC 6455 WebSocket client behavior for Codex WSS.
- Do not add a second WebSocket stack for Codex.
- H2/H3 WebSocket experiments must use Specter APIs and live evidence before changing default transport.
- Local integration tests should use Specter for both client and fixture paths where that is the behavior under review.

## Review Checklist

- Does the client see a JSON error event before relying on a close reason?
- Is the error code stable enough for Codex CLI and tests?
- Does the first frame lock the route/model, and do follow-up frames preserve it?
- Are ping, pong, binary compatibility, upstream close, and malformed JSON cases still covered?
