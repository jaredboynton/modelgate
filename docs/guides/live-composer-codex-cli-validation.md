# Composer Codex CLI live validation

This runbook defines the opt-in live validation lane for Composer requests that
exercise the Codex CLI path through UMP. It is for local operator confidence and
schema capture only. It is not a CI gate, and it must not write secrets or raw
provider payloads into tracked files.

## Scope

Use this lane to prove that a local Composer profile can reach UMP, route Codex
CLI-shaped requests to the intended Codex-backed model, and produce enough
sanitized evidence to compare request shape, context use, indexing behavior, and
failure modes across models.

Out of scope:

- changing Composer or Codex CLI config permanently;
- validating Bedrock, Google, or public OpenAI parity;
- collecting raw OAuth files, bearer tokens, cookies, prompts, workspace source,
  private file paths, or unredacted provider responses;
- treating live provider success as a replacement for unit and integration tests.

## Config backup expectations

Live validation may require temporary Composer and Codex CLI profile edits. The
operator must preserve the original local config before changing it.

Expected sequence:

1. Record the source config paths in the artifact under `config.inputs`.
2. Copy each edited config to a timestamped backup outside the repo, for example
   under a local scratch directory ignored by git.
3. Record backup paths and content hashes under `config.backups`.
4. Apply only the minimum temporary route/provider edits needed for the matrix
   row under test.
5. Restore the original files after the run and record restore status under
   `config.restore`.

Never commit backup copies, auth files, exported Composer profiles, or local
scratch directories. If a run cannot prove that backup and restore completed,
mark it `live-blocked` with `blocked_reason = "config-backup-missing"` or
`"config-restore-unverified"`.

## Environment gates

Live validation is disabled unless all required gates are explicit. A runbook or
script may use different variable names, but the artifact must record the gates
that were present.

Required gates:

- `UMP_V2_LIVE_HARNESS=1` opts into the generic live harness.
- `UMP_V2_LIVE_COMPOSER_CODEX_CLI=1` opts into this lane.
- `UMP_BASE_URL` points at the specific UMP proxy for the run. Prefer an
  ephemeral `127.0.0.1:0` proxy with the actual bound address copied from its
  logs; use `http://127.0.0.1:18743` only when intentionally validating the
  fixed launchd service.
- Composer is configured to send the selected model through UMP.
- Codex CLI OAuth state exists locally and is valid for the requested Codex
  route.
- Any prompt, workspace, or fixture used by the run is sanitized or synthetic.
- `CODEX_CONFIG_BACKUP` points at a backup copy, or the harness can discover the
  latest `~/.codex/backups/composer-*/config.toml`.

Recommended gates:

- `UMP_V2_LIVE_HARNESS_RUNS_ROOT` points to an ignored local artifact directory.
- `UMP_V2_LIVE_HARNESS_STAMP` supplies a stable run identifier.
- `UMP_V2_LIVE_ALLOW_CONTEXT_PROBE=1` allows context/indexing probes that may
  inspect workspace metadata.
- `UMP_V2_LIVE_REDACTION_STRICT=1` fails the run if a required field cannot be
  redacted safely.

When any required gate is missing, do not attempt a live request. Emit or write a
blocked artifact instead.

## Live-blocked states

Use `status = "live-blocked"` when the row did not make a provider call because
local prerequisites were missing or unsafe.

Common blocked reasons:

| Reason | Meaning | Recovery |
|---|---|---|
| `env-gate-missing` | A required opt-in variable was absent. | Set `UMP_V2_LIVE_HARNESS=1` and `UMP_V2_LIVE_COMPOSER_CODEX_CLI=1`. |
| `ump-unreachable` | UMP was not listening at `UMP_BASE_URL`. | Start a local UMP proxy, preferably on an ephemeral port, and retry. |
| `composer-config-missing` | Composer was not configured for the row. | Back up config, apply the temporary profile, retry. |
| `composer-catalog-unavailable` | The Codex model catalog or UMP `/v1/models` did not expose the Composer slugs. | Add or refresh `composer-2`, `composer-2-fast`, and `composer-1.5`, then retry. |
| `config-backup-missing` | Config would be edited without a recorded backup. | Create and record backups first. |
| `config-restore-unverified` | Original config restore was not proven. | Restore from backup and record hashes. |
| `codex-auth-missing` | Required Codex OAuth state was absent. | Authenticate Codex CLI locally, then retry. |
| `codex-auth-expired` | OAuth refresh failed before the row could run. | Refresh Codex CLI auth outside the artifact path. |
| `fixture-missing` | The selected prompt/workspace fixture was unavailable. | Install or point at a sanitized fixture. |
| `redaction-failed` | Sanitization could not guarantee safe output. | Fix redaction before rerunning. |
| `operator-aborted` | The operator intentionally stopped the live run. | No action unless another run is needed. |

Use `status = "live-failed"` only after the row made a live request and the
request, transport, adapter, or provider behavior failed.

## Matrix rows

Each row should isolate one Composer/Codex CLI behavior. Keep prompts synthetic,
small, and repeatable.

| Row id | Purpose | Required evidence |
|---|---|---|
| `streaming_chat` | Basic Composer → UMP → Codex streaming chat succeeds. | Profile/model, event sequence or transcript marker, terminal output, latency, redaction pass. |
| `reasoning_metadata` | Reasoning row returns final correctness and exposure classification. | Final answer `391`, reasoning marker or `not_exposed_by_cli`, transcript sidecars, redaction pass. |
| `continuation_resume` | `codex exec resume --last` preserves continuation state. | Two commands, resume output containing the synthetic token, no cross-model/provider fallback. |
| `single_tool_call` | Tool-capable request uses a shell/tool call. | Tool invocation evidence, command result, repo basename, read-only sandbox command. |
| `parallel_tool_calling` | Parallel tool-call prompt inspects two files before synthesis. | Two file-inspection facts; classify serialized execution as warning instead of silent pass. |
| `context_indexing` | Codebase context/indexing recognition is classified. | File/symbol/architecture hits, classification, no hallucinated paths or source snippets. |
| `negative_unsupported_model` | Unsupported model fails closed. | Model error, no fallback to other provider/model/API-key path, redaction pass. |
| `negative_missing_auth` | Missing Codex auth fails closed. | Empty Codex home, auth error class, no real auth path leakage, redaction pass. |

Rows may be skipped, but skipped rows need a `live-blocked` artifact with a
reason instead of disappearing from the matrix.

## Artifact schema

Write one sanitized JSON artifact per matrix row. Use stable field names so
later scripts can compare runs without parsing prose.

```json
{
  "schema_version": 1,
  "run_id": "2026-05-14T120000Z-local",
  "row_id": "streaming_chat",
  "status": "pass",
  "blocked_reason": null,
  "started_at": "2026-05-14T12:00:00Z",
  "finished_at": "2026-05-14T12:00:08Z",
  "operator": {
    "host_hash": "sha256:...",
    "repo_head": "<git sha or dirty>",
    "ump_version": "<binary or crate version>"
  },
  "gates": {
    "UMP_V2_LIVE_HARNESS": "present",
    "UMP_V2_LIVE_COMPOSER_CODEX_CLI": "present",
    "UMP_V2_LIVE_ALLOW_CONTEXT_PROBE": "absent"
  },
  "config": {
    "inputs": [
      { "name": "composer-profile", "path_hash": "sha256:..." }
    ],
    "backups": [
      { "name": "composer-profile", "path_hash": "sha256:...", "content_hash": "sha256:..." }
    ],
    "restore": {
      "attempted": true,
      "verified": true,
      "content_hash_matches_backup": true
    }
  },
  "request": {
    "route": "/v1/responses",
    "composer_model": "gpt-5.5-codex-fixture",
    "resolved_provider": "codex",
    "resolved_model": "<redacted-or-public-model-id>",
    "stream": false,
    "prompt_fixture_id": "synthetic-small-context-v1"
  },
  "context": {
    "probe_enabled": false,
    "estimated_input_tokens": 128,
    "reported_context_window": null,
    "remaining_context_tokens": null,
    "limit_signal": null
  },
  "indexing": {
    "probe_enabled": false,
    "workspace_fixture_id": null,
    "indexed_file_count": null,
    "indexed_path_hashes": [],
    "ignored_path_hashes": []
  },
  "response": {
    "http_status": 200,
    "event_types": [],
    "output_text_present": true,
    "tool_call_names": [],
    "provider_request_id_hash": "sha256:..."
  },
  "redaction": {
    "policy_version": 1,
    "strict": true,
    "findings": [],
    "raw_capture_retained": true,
    "raw_capture_scope": "ignored local run directory only"
  },
  "sidecars": {
    "command": "command.txt",
    "stdout": "stdout.jsonl",
    "stderr": "stderr.log",
    "timing": "timing.txt",
    "proxy_log": "proxy.redacted.log",
    "redacted_stdout": "stdout.redacted.jsonl",
    "redacted_stderr": "stderr.redacted.log"
  },
  "notes": []
}
```

Allowed statuses:

- `pass`: row completed and met its expected live behavior;
- `fail`: row made a live request but returned incorrect behavior;
- `warn`: row completed with caveats such as serialized tool use or reasoning
  metadata not exposed by the CLI;
- `live-blocked`: row did not run because prerequisites or safety gates failed;
- `skipped`: row was intentionally omitted from a larger run matrix.

The artifact may add fields, but existing fields should keep their meaning.
Prefer hashes and counts over raw values.
Raw `stdout.jsonl` and `stderr.log` sidecars are local ignored debugging
captures. Only summaries, redacted sidecars, hashes, counts, and sanitized
error classes are eligible for durable docs or issue attachments.

## Redaction policy

Artifacts must be safe to attach to an issue or planning note after review.
Default to strict redaction.

Do not write:

- bearer tokens, refresh tokens, cookies, API keys, account ids, or auth file
  contents;
- raw prompts from a private workspace;
- raw provider responses that may echo prompts or indexed source;
- absolute local file paths, home directories, usernames, hostnames, or IPs other
  than `127.0.0.1`;
- source file contents, diffs, or snippets from non-synthetic workspaces.

Allowed after redaction:

- route names, status codes, event type names, public model aliases, durations,
  booleans, counts, hashes, and synthetic fixture identifiers;
- sanitized error codes and stable internal blocked reasons;
- path hashes or basename-only fixture names when the fixture is synthetic and
  tracked.

If redaction cannot prove safety, discard the raw capture and produce a
`live-blocked` artifact with `blocked_reason = "redaction-failed"`.

## Interpreting context results

Context evidence is directional unless the provider returns an explicit token or
window value. Keep the artifact honest about the source of each value.

- `estimated_input_tokens` comes from local estimation. Treat it as approximate.
- `reported_context_window` is trustworthy only when surfaced by the provider,
  route metadata, or a documented catalog entry.
- `remaining_context_tokens` is useful for comparing rows in one run, not for
  claiming an exact provider limit.
- `limit_signal = "accepted"` means the request completed without context-limit
  symptoms.
- `limit_signal = "truncated"` means Composer, Codex CLI, UMP, or the provider
  indicated truncation or compaction before final output.
- `limit_signal = "rejected"` means a context-length error or equivalent failure
  was returned.
- `limit_signal = "unknown"` means the row did not collect enough evidence.

Do not infer that a model has a larger or smaller context window from one pass.
Use repeated synthetic probes and catalog evidence before updating model docs.

## Interpreting indexing results

Indexing checks should answer whether Composer/Codex CLI saw the intended
synthetic workspace inputs, not whether the model reasoned correctly about a
private repository.

- `indexed_file_count = 0` is expected for empty fixtures and context-only rows.
- A positive `indexed_file_count` should match synthetic fixture files only.
- Store `indexed_path_hashes`, not absolute paths, unless the basename is a
  tracked synthetic fixture name.
- `ignored_path_hashes` should include files intentionally excluded by the
  fixture policy when that evidence is available.
- If the assistant cites a file that was not in the synthetic fixture set, mark
  the row `live-failed` or `live-blocked` and inspect redaction before saving.
- If indexing was disabled or not observable, set `probe_enabled = false` and
  leave counts null instead of writing guessed values.

A passing indexing row proves that the live path can surface the synthetic file
set. It does not prove full IDE indexing parity or correctness for arbitrary
private workspaces.
