#!/usr/bin/env bash
set -euo pipefail

base_url="${UMP_V2_LIVE_BASE_URL:-http://127.0.0.1:18743}"
base_url="${base_url%/}"

tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT

redact() {
  local text="$1"
  local value
  for name in AWS_BEARER_TOKEN_BEDROCK GOOGLE_API_KEY OPENAI_API_KEY ANTHROPIC_API_KEY; do
    value="${!name:-}"
    if [ "${#value}" -ge 8 ]; then
      text="${text//$value/[REDACTED]}"
    fi
  done
  printf '%s' "$text"
}

scan_for_secret_leaks() {
  local file="$1"
  local value
  for name in AWS_BEARER_TOKEN_BEDROCK GOOGLE_API_KEY OPENAI_API_KEY ANTHROPIC_API_KEY; do
    value="${!name:-}"
    if [ "${#value}" -ge 8 ] && grep -Fq "$value" "$file"; then
      printf 'smoke failed: response leaked %s\n' "$name" >&2
      return 1
    fi
  done
}


check_codex_compaction_safety() {
  local config="${CODEX_CONFIG:-$HOME/.codex/config.toml}"
  if [ ! -f "$config" ]; then
    return 0
  fi

  local request_compression remote_compaction
  request_compression="$(awk '
    /^\[features\]$/ { in_features = 1; next }
    /^\[/ { in_features = 0 }
    in_features && $1 == "enable_request_compression" { print $3; exit }
  ' "$config")"
  remote_compaction="$(awk '
    /^\[features\]$/ { in_features = 1; next }
    /^\[/ { in_features = 0 }
    in_features && $1 == "remote_compaction_v2" { print $3; exit }
  ' "$config")"

  if [ "$request_compression" != "true" ]; then
    printf 'smoke failed: Codex config must keep [features].enable_request_compression = true for UMP transport compression\n' >&2
    return 1
  fi
  if [ "$remote_compaction" = "true" ]; then
    printf 'smoke failed: Codex mixed UMP config must keep [features].remote_compaction_v2 = false until provider-aware compaction lands; use proxy-ws for Codex-only compaction\n' >&2
    return 1
  fi
}

request() {
  local method="$1"
  local path="$2"
  local output="$3"
  local status
  status="$(curl -sS --connect-timeout 2 --max-time 10 -X "$method" -H 'content-type: application/json' -o "$output" -w '%{http_code}' "$base_url$path")"
  scan_for_secret_leaks "$output"
  printf '%s' "$status"
}

check_codex_compaction_safety

health_body="$tmpdir/health.json"
health_status="$(request GET /health "$health_body")"
if [ "$health_status" != "200" ]; then
  printf 'smoke failed: GET /health returned %s: %s\n' "$health_status" "$(redact "$(cat "$health_body")")" >&2
  exit 1
fi
grep -Eq '"status"[[:space:]]*:[[:space:]]*"ok"' "$health_body" || {
  printf 'smoke failed: GET /health did not return status ok: %s\n' "$(redact "$(cat "$health_body")")" >&2
  exit 1
}

models_body="$tmpdir/models.json"
models_status="$(request GET /v1/models "$models_body")"
if [ "$models_status" != "200" ]; then
  printf 'smoke failed: GET /v1/models returned %s: %s\n' "$models_status" "$(redact "$(cat "$models_body")")" >&2
  exit 1
fi
grep -Eq '"object"[[:space:]]*:[[:space:]]*"list"' "$models_body" || {
  printf 'smoke failed: GET /v1/models did not return a model list: %s\n' "$(redact "$(cat "$models_body")")" >&2
  exit 1
}

printf 'smoke ok: %s\n' "$base_url"
