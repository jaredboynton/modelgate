# Live fixture policy

This directory is reserved for tracked metadata and schema notes for opt-in live
validation. It must not contain raw live captures, credentials, provider payloads,
private prompts, Composer exports, Codex auth files, or local config backups.

## Composer Codex CLI artifacts

Composer Codex CLI live validation writes sanitized JSON artifacts outside the
repo, normally under the ignored directory named by `UMP_V2_LIVE_ARTIFACT_DIR`.
Tracked fixtures may reference only synthetic prompt or workspace ids.

Use `docs/guides/live-composer-codex-cli-validation.md` as the runbook and
schema source. Each artifact should include:

- `schema_version`, `run_id`, `row_id`, `status`, and timestamps;
- explicit env gate presence, never raw env values that contain secrets;
- config input hashes, backup hashes, and restore verification status;
- route/model resolution, response status, event types, and output presence;
- context estimates with clear `accepted`, `truncated`, `rejected`, or `unknown`
  limit signals;
- indexing counts and path hashes for synthetic workspaces only;
- redaction policy version, strict-mode status, findings, and raw-capture
  retention status.

## Live-blocked records

A skipped prerequisite is still a result. Prefer a tiny `live-blocked` artifact
over missing evidence when a matrix row cannot safely run.

Common blocked reasons are:

- `env-gate-missing`
- `ump-unreachable`
- `composer-config-missing`
- `config-backup-missing`
- `config-restore-unverified`
- `codex-auth-missing`
- `codex-auth-expired`
- `fixture-missing`
- `redaction-failed`
- `operator-aborted`

## Redaction rules

Allowed tracked material:

- synthetic fixture ids;
- JSON schema examples with placeholder values;
- hashes, counts, booleans, status codes, event type names, and public route
  names;
- documentation explaining how to reproduce a live run locally.

Forbidden tracked material:

- bearer tokens, refresh tokens, cookies, API keys, account ids, and auth files;
- raw Composer, Codex CLI, or provider config with local personal values;
- raw prompts, private workspace source, absolute local paths, usernames,
  hostnames, and unredacted request/response bodies;
- live artifact directories or config backups.

If a fixture needs real provider behavior, store only a sanitized schema sample
or blocked-state record here and keep the raw capture in ignored local scratch
until it is deleted.
