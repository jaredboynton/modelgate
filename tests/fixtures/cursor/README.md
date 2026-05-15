# Cursor protobuf parity fixtures

Pre-generated wire-bytes used by `tests/unit_cursor_proto.rs` and the
adapter integration suites. Each `.bin` is the byte output of either the
v1 TypeScript encoder (`/Users/jaredboynton/__devlocal/unified-model-proxy/src/lib/cursor/proto/agent_pb.ts`)
or a captured Cursor agent server frame.

## Layout

```
inputs/      # Deterministic JSON inputs that drive both encoders.
run/         # AgentClientMessage / AgentRunRequest encoded bodies.
server/      # AgentServerMessage decoded variants.
unary/       # GetUsableModels request + response (raw + Connect-framed).
connect/     # Connect frame samples (split, multi, end-stream, malformed).
scripts/     # Parity script entry points (drives v1 encoder via Bun).
```

Refer to `.omx/research/cursor-phase0/fixtures-extraction.md` for the
full manifest, the parity command pattern, and redaction policy.

## Regenerating fixtures

The Rust unit tests gate the on-disk fixtures behind
`UMP_REGENERATE_CURSOR_FIXTURES=1`. Run with that env to (re)write the
`run/basic_system_user.bin` baseline from the live Rust encoder, then
verify byte parity against the v1 TypeScript encoder via the script in
`scripts/`.

The ralplan binds Phase 0 acceptance to the manifest in
`.omx/plans/adr-cursor-transport-protobuf-20260515T052535Z.md:64-80`.

## Redaction policy

All committed fixtures use synthetic workspace ids, placeholder file
paths, no tokens, no real prompts. Per the live-fixture policy at
`tests/fixtures/live/README.md` and the ADR Droid contract, tokens,
prompt bodies, full file contents, and private workspace paths are
forbidden in this tree.
