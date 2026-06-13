# WebSocket Responses

Golden rule: `/v1/responses` WebSocket handling must fail closed with JSON error events clients can read before any close frame matters.

## Contract

- Downstream WebSocket Responses is a mixed-provider facade, not a Codex-only tunnel.
- Each `response.create` resolves its own `format: "responses"` route/model fingerprint.
- After a response reaches a terminal event, later `response.create` events may switch provider or model independently on the same connection.
- While a response is in flight, another raw Responses body or `response.create` returns `response_already_in_flight` and must not start another upstream call.
- `previous_response_id` is connection-local. Unknown IDs, non-string IDs, or IDs from another connection return JSON error events.
- `previous_response_id` continuations must exactly match the prior route/model fingerprint. Cross-provider, cross-model, or changed request-field continuations return JSON error events.
- Codex-backed requests use Codex upstream WSS; Bedrock and Google requests use provider-specific HTTP/SSE bridges and stream normalized Responses events back to the downstream socket.
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

## Warpsock Boundary

- `warpsock` owns RFC 6455 WebSocket client behavior for Codex WSS.
- Do not add a second WebSocket stack for Codex.
- H2/H3 WebSocket experiments must use Warpsock APIs and live evidence before changing default transport.
- Local integration tests should use Warpsock for both client and fixture paths where that is the behavior under review.

## Review Checklist

- Does the client see a JSON error event before relying on a close reason?
- Is the error code stable enough for Codex CLI and tests?
- Can independent post-terminal `response.create` events switch provider/model on the same downstream socket?
- Are in-flight `response.create` events rejected with `response_already_in_flight`?
- Are `previous_response_id` continuations connection-local and exact-match for route/model plus stable request fields?
- Are ping, pong, binary compatibility, upstream close, and malformed JSON cases still covered?
