# WebSocket Provider Switch Fix Spec

## Problem

Codex CLI treats `supports_websockets` as provider-wide. When the user routes
OpenAI, Bedrock, and Google models through the same local provider profile, the
CLI can keep one `/v1/responses` WebSocket open while changing `model` between
turns. The previous proxy bridge rejected that second independent turn with:

```json
{
  "type": "error",
  "status": 400,
  "error": {
    "code": "websocket_route_model_changed",
    "message": "route/model changes on one socket are not supported; open a new WebSocket for gpt-5.5"
  }
}
```

That behavior was not a provider failure. It was a local bridge policy encoded
in `src/route/websocket.rs`: the first recorded response stored a
`bound_fingerprint`, and later no-`previous_response_id` turns called
`ensure_socket_binding_matches` before execution. The fix removes that
socket-wide binding, updates the README/WebSocket golden-principle docs, and
inverts the tests that used to assert the stale contract.

## Target Contract

- One downstream WebSocket may carry multiple independent `response.create`
  turns with different models and providers.
- Independent turns are allowed only when no response is in flight.
- A `response.create` with no `previous_response_id` starts a new independent
  turn and must not be compared to the first route/model seen on the socket.
- A `response.create` with an explicit `previous_response_id` remains
  connection-local and exact-match:
  - absent means independent turn;
  - present non-string, including `null`, is invalid and must not be treated as
    absent;
  - the ID must exist in the socket's bounded response LRU;
  - the provider lane must match;
  - the requested model and upstream model fingerprint must match;
  - changed non-incremental request fields still reject;
  - the bridge strips `previous_response_id` before Google or Bedrock adapters
    can see it.
- Overlapping `response.create` frames remain rejected with
  `response_already_in_flight`, even if the second frame targets a different
  provider.
- `response.processed` stays a Codex-upstream acknowledgement when a Codex turn
  is in flight; otherwise it is accepted as a no-op.
- Error frames keep the top-level JSON shape clients already parse:
  `type`, numeric `status`, `error.code`, and `error.message`.

## Implementation Record

Completed changes:

- Socket binding moved from a socket-wide `bound_fingerprint` to per-response
  state in `BridgeSessionState`.
- Absent `previous_response_id` now starts an independent turn instead of
  comparing against the first route/model seen on the socket.
- Present non-string `previous_response_id` values, including `null`, return
  `invalid_previous_response_id`.
- Unknown IDs, cross-lane continuations, same-lane/different-model
  continuations, and non-incremental continuation changes remain rejected.
- `previous_response_id` is stripped before Google or Bedrock adapters receive
  the request.
- Overlapping `response.create` frames still return
  `response_already_in_flight` and are not queued.
- HTTP-backed bridge tasks stop reading client frames after forwarding a
  terminal event until the provider task result is observed.
- Codex turns still use Codex WSS, while Bedrock and Google turns still use
  HTTP-backed provider execution through `execute_responses_request`.
- README, WebSocket golden principles, Provider Boundaries, and this planning
  note describe the mixed downstream socket plus isolated upstream providers.

## Test Coverage Record

Coverage now locks these behaviors:

- Unit-level request preparation:
  - no-`previous_response_id` request with a different fingerprint is accepted;
  - explicit non-string `previous_response_id`, including `null`, returns
    `invalid_previous_response_id` or the final chosen typed policy code;
  - unknown `previous_response_id` still errors;
  - cross-provider continuation still errors with
    `previous_response_route_mismatch`;
  - same-provider different-model continuation still errors with
    `previous_response_model_mismatch`.
  - after more than `BRIDGE_RESPONSE_STATE_LIMIT` recorded responses, an evicted
    response ID returns `unknown_previous_response_id`.

- `tests/integration_websocket_facade.rs`:
  - `websocket_facade_allows_model_switches_after_terminal_events` proves a
    Google `generate:false` prewarm followed by Bedrock `generate:false`
    returns a second synthetic lifecycle;
  - `websocket_facade_generate_false_prewarm_then_real_cross_model_turn_reaches_provider`
    proves an independent real turn reaches the correct provider path;
  - `websocket_facade_cross_route_previous_response_id_error_keeps_socket_usable`
    stays strict and asserts the socket can still accept a later independent
    turn on a different provider;
  - unknown previous-ID tests remain strict.

- `tests/integration_websocket_passthrough.rs`:
  - Codex real turn -> Google independent turn is accepted after the Codex
    terminal event using non-live provider seams;
  - Codex real turn -> Bedrock independent turn is accepted after the Codex
    terminal event using non-live provider seams;
  - Google/Bedrock independent prewarm -> Codex real turn succeeds and opens
    Codex WSS only for the Codex turn;
  - same-socket same-provider different-model independent turn succeeds after a
    terminal event;
  - flat/raw Responses JSON frames, not only wrapped `response.create` frames,
    can switch models after terminal events;
  - a second `response.create` sent while a Codex turn is in flight returns
    `response_already_in_flight`, not `websocket_route_model_changed`, and sends
    no second upstream Codex frame;
  - a second `response.create` sent while an HTTP-backed provider bridge task is
    in flight returns `response_already_in_flight`, not
    `websocket_route_model_changed`, and starts no second provider execution.

- Docs/contract tests:
  - no test or doc asserts socket-wide first-frame binding;
  - error-shape assertions remain for all policy errors.

## Review Outcomes

Critic wave 1 rejected a naive "just allow switches" change. The accepted
resolution is narrower:

- allow only independent post-terminal turns;
- keep exact continuation matching for `previous_response_id`;
- keep in-flight rejection;
- keep provider/auth isolation;
- keep stable JSON error events.

Critic wave 2 rejected the first draft until it added:

- typed rejection for malformed `previous_response_id`;
- non-live HTTP-backed switch tests;
- bounded-LRU eviction coverage;
- both Codex and HTTP-backed in-flight overlap coverage;
- README, WebSocket golden-principle, and Provider Boundaries doc updates.

This fix does not add cross-provider conversation continuation. It only allows a
single downstream WebSocket to carry multiple independent provider turns, which
matches how Codex CLI behaves when the provider-level WebSocket capability is
enabled.

## Completion Evidence

- The `websocket_route_model_changed` code path is gone for independent turns.
- Old rejection tests are inverted to success tests.
- Continuation mismatch tests still pass.
- Malformed `previous_response_id` and LRU eviction tests pass.
- Codex and HTTP-backed in-flight overlap tests pass.
- README, WebSocket golden-principle docs, Provider Boundaries docs, and this
  note state the new contract.
- `cargo fmt --check`, targeted WebSocket tests, `cargo test`, and
  `cargo clippy --all-targets --all-features -- -D warnings` pass locally.
