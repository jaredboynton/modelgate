#!/usr/bin/env bash
set -uo pipefail

SCRIPT_NAME="$(basename "$0")"
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUNS_ROOT="${UMP_V2_LIVE_HARNESS_RUNS_ROOT:-$REPO_ROOT/.live-harness/runs}"
STAMP="${UMP_V2_LIVE_HARNESS_STAMP:-$(date -u +%Y%m%dT%H%M%SZ)}"
RUN_DIR="$RUNS_ROOT/$STAMP"
SUMMARY_JSONL="$RUN_DIR/events.jsonl"
SUMMARY_JSON="$RUN_DIR/summary.json"
BLOCKERS_FILE="$RUN_DIR/live-blockers.txt"
REDACTION_REPORT="$RUN_DIR/redaction-report.json"
CODEX_BIN="${CODEX_BIN:-codex}"
CODEX_CONFIG="${CODEX_CONFIG:-$HOME/.codex/config.toml}"
CODEX_CONFIG_BACKUP="${CODEX_CONFIG_BACKUP:-}"
CODEX_CATALOG="${CODEX_MODEL_CATALOG:-$HOME/.codex/model-catalog-ump-v2.json}"
CODEX_HOME_DIR="${UMP_V2_CODEX_HOME:-${CODEX_HOME:-$HOME/.codex}}"
UMP_PROXY_LOG="${UMP_PROXY_LOG:-}"
PROFILE="${CODEX_PROFILE:-composer-2}"
FAST_PROFILE="${CODEX_FAST_PROFILE:-composer-2-fast}"
LEGACY_PROFILE="${CODEX_LEGACY_PROFILE:-composer-1-5}"
MODEL="${CODEX_MODEL:-composer-2}"
FAST_MODEL="${CODEX_FAST_MODEL:-composer-2-fast}"
LEGACY_MODEL="${CODEX_LEGACY_MODEL:-composer-1.5}"
UMP_BASE_URL="${UMP_BASE_URL:-}"
NEGATIVE_AUTH_BASE_URL="${UMP_V2_NEGATIVE_AUTH_BASE_URL:-}"
CODEX_PROVIDER_BASE_URL="${CODEX_PROVIDER_BASE_URL:-}"
if [[ -z "$CODEX_PROVIDER_BASE_URL" && -n "$UMP_BASE_URL" ]]; then
  CODEX_PROVIDER_BASE_URL="${UMP_BASE_URL%/}"
  if [[ "$CODEX_PROVIDER_BASE_URL" != */v1 ]]; then
    CODEX_PROVIDER_BASE_URL="$CODEX_PROVIDER_BASE_URL/v1"
  fi
fi
CODEX_CONFIG_OVERRIDES=()
if [[ -n "$CODEX_PROVIDER_BASE_URL" ]]; then
  CODEX_CONFIG_OVERRIDES=(-c "model_providers.ump-v2.base_url=\"$CODEX_PROVIDER_BASE_URL\"")
fi
RETRY_DELAYS=(10 30)
OVERALL_STATUS="pass"
ROW_RESULTS=()
BLOCKERS=()
REDACTION_FINDINGS=()

mkdir -p "$RUN_DIR"
: > "$SUMMARY_JSONL"
: > "$BLOCKERS_FILE"

usage() {
  cat <<USAGE
$SCRIPT_NAME runs the local-only Composer/Codex CLI live validation harness.

Required opt-in gates:
  UMP_V2_LIVE_HARNESS=1
  UMP_V2_LIVE_COMPOSER_CODEX_CLI=1
  UMP_V2_ALLOW_LIVE_TESTS_IN_CI=1   required only when CI is set

Useful overrides:
  CODEX_BIN=$CODEX_BIN
  CODEX_CONFIG=$CODEX_CONFIG
  CODEX_CONFIG_BACKUP=${CODEX_CONFIG_BACKUP:-<auto-detect latest ~/.codex/backups/composer-*/config.toml>}
  CODEX_MODEL_CATALOG=$CODEX_CATALOG
  UMP_V2_CODEX_HOME=$CODEX_HOME_DIR
  CODEX_PROFILE=$PROFILE
  UMP_BASE_URL=${UMP_BASE_URL:-<required, prefer ephemeral proxy bound address>}
  CODEX_PROVIDER_BASE_URL=${CODEX_PROVIDER_BASE_URL:-<derived as UMP_BASE_URL/v1>}
  UMP_V2_NEGATIVE_AUTH_BASE_URL=${NEGATIVE_AUTH_BASE_URL:-<optional isolated empty-auth proxy for missing-auth row>}
  UMP_PROXY_LOG=${UMP_PROXY_LOG:-<optional proxy log copied as redacted sidecar>}
  UMP_V2_LIVE_HARNESS_RUNS_ROOT=$RUNS_ROOT

Artifacts are written under:
  $RUN_DIR
USAGE
}

json_escape() {
  local input="${1-}"
  input=${input//\\/\\\\}
  input=${input//"/\\"}
  input=${input//$'\n'/\\n}
  input=${input//$'\r'/\\r}
  input=${input//$'\t'/\\t}
  printf '%s' "$input"
}

sanitize_text() {
  local input="${1-}"
  if [[ "$CODEX_BIN" == */* ]]; then
    input=${input//$CODEX_BIN/\$CODEX_BIN}
  fi
  input=${input//$RUN_DIR/\$RUN_DIR}
  input=${input//$REPO_ROOT/\$REPO_ROOT}
  if [[ -n "$CODEX_CONFIG_BACKUP" ]]; then
    input=${input//$CODEX_CONFIG_BACKUP/\$CODEX_CONFIG_BACKUP}
  fi
  input=${input//$CODEX_CONFIG/\$CODEX_CONFIG}
  input=${input//$CODEX_CATALOG/\$CODEX_MODEL_CATALOG}
  input=${input//$CODEX_HOME_DIR/\$CODEX_HOME}
  if [[ -n "$UMP_PROXY_LOG" ]]; then
    input=${input//$UMP_PROXY_LOG/\$UMP_PROXY_LOG}
  fi
  input=${input//$HOME/\$HOME}
  printf '%s' "$input"
}

path_hash() {
  sha256_text "$1"
}

now_rfc3339() {
  date -u +%Y-%m-%dT%H:%M:%SZ
}

sha256_text() {
  if command -v shasum >/dev/null 2>&1; then
    printf '%s' "$1" | shasum -a 256 | awk '{print "sha256:" $1}'
  elif command -v sha256sum >/dev/null 2>&1; then
    printf '%s' "$1" | sha256sum | awk '{print "sha256:" $1}'
  else
    printf 'sha256:unavailable'
  fi
}

sha256_file() {
  local path="$1"
  if [[ ! -f "$path" ]]; then
    printf 'sha256:missing'
  elif command -v shasum >/dev/null 2>&1; then
    shasum -a 256 "$path" | awk '{print "sha256:" $1}'
  elif command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$path" | awk '{print "sha256:" $1}'
  else
    printf 'sha256:unavailable'
  fi
}

latest_config_backup() {
  local latest=""
  local candidate
  shopt -s nullglob
  for candidate in "$HOME"/.codex/backups/composer-*/config.toml; do
    latest="$candidate"
  done
  shopt -u nullglob
  printf '%s' "$latest"
}

redact_file() {
  local src="$1"
  local dest="$2"
  if [[ ! -f "$src" ]]; then
    : > "$dest"
    return 0
  fi

  local json_scrubbed="$dest.json-scrubbed"
  python3 - "$src" "$json_scrubbed" <<'PY'
import json
import pathlib
import sys

src = pathlib.Path(sys.argv[1])
dest = pathlib.Path(sys.argv[2])
lines = src.read_text(errors="replace").splitlines()

def scrub(value):
    if isinstance(value, dict):
        return {
            key: (
                "[REDACTED-TOOL-OUTPUT]"
                if key == "aggregated_output"
                else scrub(item)
            )
            for key, item in value.items()
        }
    if isinstance(value, list):
        return [scrub(item) for item in value]
    return value

redacted = []
for line in lines:
    try:
        redacted.append(json.dumps(scrub(json.loads(line)), separators=(",", ":")))
    except Exception:
        redacted.append(line)

dest.write_text(("\n".join(redacted) + "\n") if redacted else "")
PY

  perl -pe '
    s/(Bearer )[A-Za-z0-9._~+\/-]+/$1[REDACTED]/g;
    s/((?:api|access|refresh|id|client)[_-]?(?:key|token|secret)[^:=]{0,16}[:=]\s*[" ]?)[^" ,}]+/$1[REDACTED]/ig;
    s/((?:cookie|set-cookie|authorization)[^:=]{0,16}[:=]\s*[" ]?)[^" ,}]+/$1[REDACTED]/ig;
    s#(https?|wss)://\S+\?\S+#$1://[REDACTED-URL-WITH-QUERY]#g;
    my $raw_home = $ENV{HOME} // "";
    my $home = quotemeta($raw_home);
    s#~/\.codex(?:/\S*)?#\$HOME/[REDACTED-CODEX-PATH]#g;
    s#$home/\.codex(?:/\S*)?#\$HOME/[REDACTED-CODEX-PATH]#g if length $home;
    s#(?:/[^/\s]+)+/\.codex(?:/\S*)?#[REDACTED-CODEX-PATH]#g;
    s/$home/\$HOME/g if length $home;
  ' "$json_scrubbed" > "$dest"
  rm -f "$json_scrubbed"
}

record_redaction_scan() {
  local row_dir="$1"
  local row_name="$2"
  local findings_file="$row_dir/redaction-findings.txt"
  : > "$findings_file"

  local scan_files=()
  shopt -s nullglob
  scan_files=("$row_dir"/stdout.jsonl "$row_dir"/stderr.log "$row_dir"/*.redacted.* "$row_dir"/attempt-*/*.redacted.* "$row_dir/command.txt")
  shopt -u nullglob
  if (( ${#scan_files[@]} == 0 )); then
    return 0
  fi

  grep -hRIE \
    'Bearer [A-Za-z0-9._~+/-]+|access[_-]?token[^[]|refresh[_-]?token[^[]|id[_-]?token[^[]|client[_-]?secret[^[]|api[_-]?key[^[]|cookie[^[]|set-cookie[^[]|account[_-]?id[^[]|/\.codex/|Authorization:[^[]|/Users/|/home/' \
    "${scan_files[@]}" > "$findings_file" 2>/dev/null || true

  if [[ -s "$findings_file" ]]; then
    REDACTION_FINDINGS+=("$row_name")
    return 1
  fi
  return 0
}

write_row_event() {
  local row="$1" model="$2" profile="$3" status="$4" latency_ms="$5" error_code="$6" note="$7"
  local request_hash
  request_hash="$(sha256_text "$row|$model|$profile|$STAMP")"
  cat >> "$SUMMARY_JSONL" <<JSON
{"timestamp":"$(now_rfc3339)","row":"$(json_escape "$row")","model":"$(json_escape "$model")","profile":"$(json_escape "$profile")","endpoint_class":"codex_cli_responses","provider":"codex","status":"$(json_escape "$status")","latency_ms":$latency_ms,"request_id_hash":"$request_hash","error_code":$(if [[ -n "$error_code" ]]; then printf '"%s"' "$(json_escape "$error_code")"; else printf 'null'; fi),"flaky":$(if [[ "$error_code" == "passed_after_retry" ]]; then printf 'true'; else printf 'false'; fi),"redaction_version":"v1","note":"$(json_escape "$note")"}
JSON
  ROW_RESULTS+=("$row:$status:$latency_ms:$error_code:$note")
}

mark_blocked() {
  local reason="$1"
  BLOCKERS+=("$reason")
  printf '%s\n' "$reason" >> "$BLOCKERS_FILE"
  OVERALL_STATUS="live-blocked"
}

status_rank_update() {
  local status="$1"
  case "$status" in
    fail)
      OVERALL_STATUS="fail"
      ;;
    warn)
      if [[ "$OVERALL_STATUS" == "pass" ]]; then OVERALL_STATUS="$status"; fi
      ;;
    skipped|live-blocked)
      if [[ "$OVERALL_STATUS" == "pass" || "$OVERALL_STATUS" == "warn" ]]; then OVERALL_STATUS="live-blocked"; fi
      ;;
  esac
}

read_features_value() {
  local config="$1" key="$2"
  awk -v target_key="$key" '
    /^\[features\]$/ { in_features = 1; next }
    /^\[/ { in_features = 0 }
    in_features && $1 == target_key { print $3; exit }
  ' "$config"
}

read_profile_feature_value() {
  local config="$1" profile="$2" key="$3"
  awk -v profile="$profile" -v target_key="$key" '
    BEGIN {
      unquoted = "[profiles." profile ".features]"
      quoted = "[profiles.\"" profile "\".features]"
    }
    $0 == unquoted || $0 == quoted { in_profile_features = 1; next }
    /^\[/ { in_profile_features = 0 }
    in_profile_features && $1 == target_key { print $3; exit }
  ' "$config"
}

check_compaction_safety() {
  local config="$1"
  local request_compression
  request_compression="$(read_features_value "$config" enable_request_compression)"

  if [[ "$request_compression" != "true" ]]; then
    mark_blocked "request-compression-disabled: set [features].enable_request_compression = true for UMP transport compression"
  fi
  local profile remote_compaction
  for profile in "$PROFILE" "$FAST_PROFILE" "$LEGACY_PROFILE"; do
    remote_compaction="$(read_profile_feature_value "$config" "$profile" remote_compaction_v2)"
    if [[ "$remote_compaction" != "false" ]]; then
      mark_blocked "mixed-profile-remote-compaction-not-disabled: set [profiles.$profile.features].remote_compaction_v2 = false; use proxy-ws for Codex-only compaction"
    fi
  done
}

write_command() {
  local row_dir="$1"
  shift
  : > "$row_dir/command.txt"
  local arg_index=0 arg_count="$#"
  local arg
  for arg in "$@"; do
    arg_index=$((arg_index + 1))
    if (( arg_index == arg_count )); then
      printf '%q ' "[REDACTED-PROMPT $(sha256_text "$arg")]" >> "$row_dir/command.txt"
    else
      printf '%q ' "$(sanitize_text "$arg")" >> "$row_dir/command.txt"
    fi
  done
  printf '\n' >> "$row_dir/command.txt"
}

looks_retryable() {
  local combined="$1"
  if grep -Eiq '(^|[^0-9])(408|429|500|502|503|504)([^0-9]|$)|timeout|timed out|connection reset|connection refused|network (error|temporar|unreachable)|temporar(y|ily)|rate limit|overloaded|service unavailable|stream disconnected before completion|error sending request' "$combined"; then
    return 0
  fi
  return 1
}

attach_proxy_log() {
  local row_dir="$1"
  if [[ -n "$UMP_PROXY_LOG" && -f "$UMP_PROXY_LOG" ]]; then
    redact_file "$UMP_PROXY_LOG" "$row_dir/proxy.redacted.log"
  else
    printf 'not provided; set UMP_PROXY_LOG to attach a redacted proxy log sidecar\n' > "$row_dir/proxy.redacted.log"
  fi
}

run_with_retry() {
  local row_dir="$1"
  shift
  local attempt=1
  local max_attempts=3
  local code=0
  local start end latency
  local raw_root raw_stdout raw_stderr
  raw_root="$(mktemp -d "${TMPDIR:-/tmp}/ump-live-harness.XXXXXX")" || return 1
  raw_stdout="$raw_root/stdout.jsonl"
  raw_stderr="$raw_root/stderr.log"
  mkdir -p "$row_dir"
  : > "$raw_stdout"
  : > "$raw_stderr"
  : > "$row_dir/timing.txt"

  while (( attempt <= max_attempts )); do
    local raw_attempt_dir="$raw_root/attempt-$attempt"
    local artifact_attempt_dir="$row_dir/attempt-$attempt"
    mkdir -p "$raw_attempt_dir" "$artifact_attempt_dir"
    start=$(date +%s)
    "$@" > "$raw_attempt_dir/stdout.jsonl" 2> "$raw_attempt_dir/stderr.log"
    code=$?
    end=$(date +%s)
    latency=$(( (end - start) * 1000 ))
    cat "$raw_attempt_dir/stdout.jsonl" >> "$raw_stdout"
    cat "$raw_attempt_dir/stderr.log" >> "$raw_stderr"
    redact_file "$raw_attempt_dir/stdout.jsonl" "$artifact_attempt_dir/stdout.redacted.jsonl"
    redact_file "$raw_attempt_dir/stderr.log" "$artifact_attempt_dir/stderr.redacted.log"
    printf 'attempt=%s exit_code=%s latency_ms=%s\n' "$attempt" "$code" "$latency" >> "$row_dir/timing.txt"

    if [[ $code -eq 0 ]]; then
      printf '%s' "$attempt" > "$row_dir/attempts.txt"
      redact_file "$raw_stdout" "$row_dir/stdout.jsonl"
      redact_file "$raw_stderr" "$row_dir/stderr.log"
      rm -rf "$raw_root"
      return 0
    fi

    cat "$raw_attempt_dir/stdout.jsonl" "$raw_attempt_dir/stderr.log" > "$raw_attempt_dir/combined.log"
    redact_file "$raw_attempt_dir/combined.log" "$artifact_attempt_dir/combined.redacted.log"
    if (( attempt < max_attempts )) && looks_retryable "$raw_attempt_dir/combined.log"; then
      sleep "${RETRY_DELAYS[$((attempt - 1))]}"
      attempt=$((attempt + 1))
      continue
    fi

    printf '%s' "$attempt" > "$row_dir/attempts.txt"
    redact_file "$raw_stdout" "$row_dir/stdout.jsonl"
    redact_file "$raw_stderr" "$row_dir/stderr.log"
    rm -rf "$raw_root"
    return "$code"
  done
  redact_file "$raw_stdout" "$row_dir/stdout.jsonl"
  redact_file "$raw_stderr" "$row_dir/stderr.log"
  rm -rf "$raw_root"
}

classify_cli_output() {
  local row="$1" row_dir="$2" expected_pattern="$3"
  local attempts exit_note status error_code latency_ms
  attempts="$(cat "$row_dir/attempts.txt" 2>/dev/null || printf '1')"
  latency_ms="$(awk -F= '/latency_ms=/ {sum += $4} END {print sum+0}' "$row_dir/timing.txt" 2>/dev/null)"
  status="pass"
  error_code=""
  exit_note="matched expected output"

  if ! grep -Eiq "$expected_pattern" "$row_dir/stdout.jsonl" "$row_dir/stderr.log" 2>/dev/null; then
    status="fail"
    error_code="expected_output_missing"
    exit_note="expected pattern not found"
  elif [[ "$attempts" != "1" ]]; then
    status="pass"
    error_code="passed_after_retry"
    exit_note="passed after retry attempt $attempts"
  fi

  if ! record_redaction_scan "$row_dir" "$row"; then
    status="fail"
    error_code="redaction_scan_failed"
    exit_note="redaction scan found sensitive marker in redacted artifacts"
  fi

  write_row_summary "$row_dir" "$row" "$status" "$latency_ms" "$error_code" "$exit_note"
  write_row_event "$row" "$MODEL" "$PROFILE" "$status" "$latency_ms" "$error_code" "$exit_note"
  status_rank_update "$status"
}

write_row_summary() {
  local row_dir="$1" row="$2" status="$3" latency_ms="$4" error_code="$5" note="$6"
  cat > "$row_dir/summary.json" <<JSON
{
  "row": "$(json_escape "$row")",
  "status": "$(json_escape "$status")",
  "latency_ms": $latency_ms,
  "error_code": $(if [[ -n "$error_code" ]]; then printf '"%s"' "$(json_escape "$error_code")"; else printf 'null'; fi),
  "flaky": $(if [[ "$error_code" == "passed_after_retry" ]]; then printf 'true'; else printf 'false'; fi),
  "note": "$(json_escape "$note")",
  "artifacts": {
    "command": "command.txt",
    "stdout": "stdout.jsonl",
    "stderr": "stderr.log",
    "timing": "timing.txt",
    "proxy_log": "proxy.redacted.log",
    "redacted_stdout": "stdout.redacted.jsonl",
    "redacted_stderr": "stderr.redacted.log"
  }
}
JSON
}

run_codex_row() {
  local row="$1" prompt="$2" expected_pattern="$3"
  local row_dir="$RUN_DIR/$row"
  mkdir -p "$row_dir"
  local command=("$CODEX_BIN" exec "${CODEX_CONFIG_OVERRIDES[@]}" --profile "$PROFILE" --json -C "$REPO_ROOT" --sandbox read-only "$prompt")
  write_command "$row_dir" "${command[@]}"
  run_with_retry "$row_dir" "${command[@]}"
  local code=$?
  redact_file "$row_dir/stdout.jsonl" "$row_dir/stdout.redacted.jsonl"
  redact_file "$row_dir/stderr.log" "$row_dir/stderr.redacted.log"
  attach_proxy_log "$row_dir"
  if [[ $code -ne 0 ]]; then
    local latency_ms
    latency_ms="$(awk -F= '/latency_ms=/ {sum += $4} END {print sum+0}' "$row_dir/timing.txt" 2>/dev/null)"
    local error_code="codex_exit_$code"
    local note="codex command exited non-zero"
    if ! record_redaction_scan "$row_dir" "$row"; then
      error_code="redaction_scan_failed"
      note="redaction scan found sensitive marker in redacted artifacts"
    fi
    write_row_summary "$row_dir" "$row" "fail" "$latency_ms" "$error_code" "$note"
    write_row_event "$row" "$MODEL" "$PROFILE" "fail" "$latency_ms" "$error_code" "$note"
    status_rank_update "fail"
    return 0
  fi
  classify_cli_output "$row" "$row_dir" "$expected_pattern"
}

classify_required_patterns() {
  local row="$1" row_dir="$2" note="$3"
  shift 3
  local status="pass"
  local error_code=""
  local missing=()
  local latency_ms
  latency_ms="$(awk -F= '/latency_ms=/ {sum += $4} END {print sum+0}' "$row_dir/timing.txt" 2>/dev/null)"

  local pattern
  for pattern in "$@"; do
    if ! grep -Eiq "$pattern" "$row_dir/stdout.jsonl" "$row_dir/stderr.log" 2>/dev/null; then
      missing+=("$pattern")
    fi
  done

  if (( ${#missing[@]} > 0 )); then
    status="fail"
    error_code="expected_output_missing"
    note="$note; missing=${missing[*]}"
  fi

  if ! record_redaction_scan "$row_dir" "$row"; then
    status="fail"
    error_code="redaction_scan_failed"
    note="redaction scan found sensitive marker in redacted artifacts"
  fi

  write_row_summary "$row_dir" "$row" "$status" "$latency_ms" "$error_code" "$note"
  write_row_event "$row" "$MODEL" "$PROFILE" "$status" "$latency_ms" "$error_code" "$note"
  status_rank_update "$status"
}

run_reasoning_row() {
  local row="reasoning_metadata"
  local row_dir="$RUN_DIR/$row"
  mkdir -p "$row_dir"
  local command=("$CODEX_BIN" exec "${CODEX_CONFIG_OVERRIDES[@]}" --profile "$PROFILE" --json -C "$REPO_ROOT" --sandbox read-only 'Compute 17*23. Show only the final answer.')
  write_command "$row_dir" "${command[@]}"
  run_with_retry "$row_dir" "${command[@]}"
  local code=$?
  redact_file "$row_dir/stdout.jsonl" "$row_dir/stdout.redacted.jsonl"
  redact_file "$row_dir/stderr.log" "$row_dir/stderr.redacted.log"
  attach_proxy_log "$row_dir"
  local latency_ms status error_code note
  latency_ms="$(awk -F= '/latency_ms=/ {sum += $4} END {print sum+0}' "$row_dir/timing.txt" 2>/dev/null)"
  if [[ $code -ne 0 ]]; then
    status="fail"; error_code="codex_exit_$code"; note="reasoning command exited non-zero"
  elif ! grep -Eiq '391' "$row_dir/stdout.jsonl" "$row_dir/stderr.log"; then
    status="fail"; error_code="expected_output_missing"; note="final answer 391 missing"
  elif grep -Eiq 'reasoning|encrypted_content|summary' "$row_dir/stdout.jsonl" "$row_dir/stderr.log"; then
    status="pass"; error_code=""; note="final answer correct and reasoning marker exposed"
  else
    status="warn"; error_code="not_exposed_by_cli"; note="final answer correct; reasoning metadata not exposed by CLI transcript"
  fi
  record_redaction_scan "$row_dir" "$row" || { status="fail"; error_code="redaction_scan_failed"; note="redaction scan found sensitive marker"; }
  write_row_summary "$row_dir" "$row" "$status" "$latency_ms" "$error_code" "$note"
  write_row_event "$row" "$MODEL" "$PROFILE" "$status" "$latency_ms" "$error_code" "$note"
  status_rank_update "$status"
}

run_continuation_row() {
  local row="continuation_resume"
  local row_dir="$RUN_DIR/$row"
  mkdir -p "$row_dir"
  local cmd1=("$CODEX_BIN" exec "${CODEX_CONFIG_OVERRIDES[@]}" --profile "$PROFILE" --json -C "$REPO_ROOT" --sandbox read-only 'Remember token ALPHA-739 and answer only "ready".')
  local cmd2=("$CODEX_BIN" exec "${CODEX_CONFIG_OVERRIDES[@]}" --profile "$PROFILE" --json -C "$REPO_ROOT" --sandbox read-only resume --last 'What token did I ask you to remember?')
  {
    printf 'step1: '
    local arg
    local arg_index=0 arg_count="${#cmd1[@]}"
    for arg in "${cmd1[@]}"; do
      arg_index=$((arg_index + 1))
      if (( arg_index == arg_count )); then
        printf '%q ' "[REDACTED-PROMPT $(sha256_text "$arg")]"
      else
        printf '%q ' "$(sanitize_text "$arg")"
      fi
    done
    printf '\nstep2: '
    arg_index=0
    arg_count="${#cmd2[@]}"
    for arg in "${cmd2[@]}"; do
      arg_index=$((arg_index + 1))
      if (( arg_index == arg_count )); then
        printf '%q ' "[REDACTED-PROMPT $(sha256_text "$arg")]"
      else
        printf '%q ' "$(sanitize_text "$arg")"
      fi
    done
    printf '\n'
  } > "$row_dir/command.txt"
  run_with_retry "$row_dir/step1" "${cmd1[@]}"
  local code1=$?
  run_with_retry "$row_dir/step2" "${cmd2[@]}"
  local code2=$?
  cat "$row_dir/step1/stdout.jsonl" "$row_dir/step2/stdout.jsonl" > "$row_dir/stdout.jsonl"
  cat "$row_dir/step1/stderr.log" "$row_dir/step2/stderr.log" > "$row_dir/stderr.log"
  cat "$row_dir/step1/timing.txt" "$row_dir/step2/timing.txt" > "$row_dir/timing.txt"
  redact_file "$row_dir/stdout.jsonl" "$row_dir/stdout.redacted.jsonl"
  redact_file "$row_dir/stderr.log" "$row_dir/stderr.redacted.log"
  attach_proxy_log "$row_dir"
  if [[ $code1 -ne 0 || $code2 -ne 0 ]]; then
    local latency_ms
    latency_ms="$(awk -F= '/latency_ms=/ {sum += $4} END {print sum+0}' "$row_dir/timing.txt" 2>/dev/null)"
    local error_code="codex_resume_exit"
    local note="resume command exited non-zero"
    if ! record_redaction_scan "$row_dir" "$row"; then
      error_code="redaction_scan_failed"
      note="redaction scan found sensitive marker in redacted artifacts"
    fi
    write_row_summary "$row_dir" "$row" "fail" "$latency_ms" "$error_code" "$note"
    write_row_event "$row" "$MODEL" "$PROFILE" "fail" "$latency_ms" "$error_code" "$note"
    status_rank_update "fail"
    return 0
  fi
  printf '1' > "$row_dir/attempts.txt"
  classify_cli_output "$row" "$row_dir" 'ALPHA-739'
}

run_context_row() {
  local row="context_indexing"
  local row_dir="$RUN_DIR/$row"
  mkdir -p "$row_dir"
  local prompt='Answer with concise file references and line numbers if available. Where do AppState, AppError, model alias resolution, SSE filtering, WebSocket error wrapping, provider allowlist before credentials, and the 127.0.0.1:18743 safety constraint live in this repo?'
  local command=("$CODEX_BIN" exec "${CODEX_CONFIG_OVERRIDES[@]}" --profile "$PROFILE" --json -C "$REPO_ROOT" --sandbox read-only "$prompt")
  write_command "$row_dir" "${command[@]}"
  run_with_retry "$row_dir" "${command[@]}"
  local code=$?
  redact_file "$row_dir/stdout.jsonl" "$row_dir/stdout.redacted.jsonl"
  redact_file "$row_dir/stderr.log" "$row_dir/stderr.redacted.log"
  attach_proxy_log "$row_dir"
  if [[ $code -ne 0 ]]; then
    local latency_ms
    latency_ms="$(awk -F= '/latency_ms=/ {sum += $4} END {print sum+0}' "$row_dir/timing.txt" 2>/dev/null)"
    local error_code="codex_exit_$code"
    local note="context probe exited non-zero"
    if ! record_redaction_scan "$row_dir" "$row"; then
      error_code="redaction_scan_failed"
      note="redaction scan found sensitive marker in redacted artifacts"
    fi
    write_row_summary "$row_dir" "$row" "fail" "$latency_ms" "$error_code" "$note"
    write_row_event "$row" "$MODEL" "$PROFILE" "fail" "$latency_ms" "$error_code" "$note"
    status_rank_update "fail"
    return 0
  fi

  local hits=0
  for pattern in 'src/state\.rs|AppState' 'src/route|AppError' 'src/model_alias\.rs|model alias' 'src/sse|SSE' 'websocket|WebSocket' '127\.0\.0\.1:18743'; do
    if grep -Eiq "$pattern" "$row_dir/stdout.jsonl"; then hits=$((hits + 1)); fi
  done

  local classification="not-recognized"
  local status="fail"
  local error_code="context_not_recognized"
  if (( hits >= 5 )); then
    classification="recognized"
    status="pass"
    error_code=""
  elif (( hits >= 3 )); then
    classification="partial"
    status="warn"
    error_code="context_partial"
  elif (( hits >= 1 )); then
    classification="grep-only"
    status="warn"
    error_code="context_grep_only"
  fi

  local latency_ms
  latency_ms="$(awk -F= '/latency_ms=/ {sum += $4} END {print sum+0}' "$row_dir/timing.txt" 2>/dev/null)"
  if ! record_redaction_scan "$row_dir" "$row"; then
    status="fail"
    error_code="redaction_scan_failed"
    classification="redaction-failed"
  fi
  write_row_summary "$row_dir" "$row" "$status" "$latency_ms" "$error_code" "classification=$classification hits=$hits"
  write_row_event "$row" "$MODEL" "$PROFILE" "$status" "$latency_ms" "$error_code" "classification=$classification hits=$hits"
  status_rank_update "$status"
}

run_negative_model_row() {
  local row="negative_unsupported_model"
  local row_dir="$RUN_DIR/$row"
  mkdir -p "$row_dir"
  local command=("$CODEX_BIN" exec "${CODEX_CONFIG_OVERRIDES[@]}" --profile "$PROFILE" -m not-a-real-model --json 'hi')
  write_command "$row_dir" "${command[@]}"
  run_with_retry "$row_dir" "${command[@]}"
  local code=$?
  redact_file "$row_dir/stdout.jsonl" "$row_dir/stdout.redacted.jsonl"
  redact_file "$row_dir/stderr.log" "$row_dir/stderr.redacted.log"
  attach_proxy_log "$row_dir"
  local latency_ms status error_code note
  latency_ms="$(awk -F= '/latency_ms=/ {sum += $4} END {print sum+0}' "$row_dir/timing.txt" 2>/dev/null)"
  if [[ $code -ne 0 ]] && grep -Eiq 'model|unsupported|not.*real|not found|unknown|invalid' "$row_dir/stdout.jsonl" "$row_dir/stderr.log"; then
    status="pass"; error_code=""; note="unsupported model failed closed"
  else
    status="fail"; error_code="unsupported_model_did_not_fail_closed"; note="unsupported model did not produce expected closed failure"
  fi
  record_redaction_scan "$row_dir" "$row" || { status="fail"; error_code="redaction_scan_failed"; note="redaction scan found sensitive marker"; }
  write_row_summary "$row_dir" "$row" "$status" "$latency_ms" "$error_code" "$note"
  write_row_event "$row" "not-a-real-model" "$PROFILE" "$status" "$latency_ms" "$error_code" "$note"
  status_rank_update "$status"
}

run_missing_auth_row() {
  local row="negative_missing_auth"
  local row_dir="$RUN_DIR/$row"
  local empty_home="$row_dir/empty-codex-home"
  mkdir -p "$empty_home"
  cp "$CODEX_CONFIG" "$empty_home/config.toml"
  local negative_config_overrides=("${CODEX_CONFIG_OVERRIDES[@]}")
  if [[ -n "$NEGATIVE_AUTH_BASE_URL" ]]; then
    local negative_provider_base_url="${NEGATIVE_AUTH_BASE_URL%/}"
    if [[ "$negative_provider_base_url" != */v1 ]]; then
      negative_provider_base_url="$negative_provider_base_url/v1"
    fi
    negative_config_overrides=(-c "model_providers.ump-v2.base_url=\"$negative_provider_base_url\"")
  fi
  local command=(env CODEX_HOME="$empty_home" UMP_V2_CODEX_HOME="$empty_home" "$CODEX_BIN" exec "${negative_config_overrides[@]}" --profile "$PROFILE" --json -C "$REPO_ROOT" --sandbox read-only 'hi')
  write_command "$row_dir" "${command[@]}"
  run_with_retry "$row_dir" "${command[@]}"
  local code=$?
  redact_file "$empty_home/config.toml" "$row_dir/empty-codex-config.redacted.toml"
  rm -f "$empty_home/config.toml"
  redact_file "$row_dir/stdout.jsonl" "$row_dir/stdout.redacted.jsonl"
  redact_file "$row_dir/stderr.log" "$row_dir/stderr.redacted.log"
  attach_proxy_log "$row_dir"
  local latency_ms status error_code note
  latency_ms="$(awk -F= '/latency_ms=/ {sum += $4} END {print sum+0}' "$row_dir/timing.txt" 2>/dev/null)"
  if [[ $code -ne 0 ]] && grep -Eiq 'auth|login|credential|token|unauthorized|not authenticated' "$row_dir/stdout.jsonl" "$row_dir/stderr.log"; then
    status="pass"; error_code=""; note="missing auth failed closed"
  elif [[ -z "$NEGATIVE_AUTH_BASE_URL" && $code -eq 0 ]]; then
    status="warn"; error_code="server_auth_not_isolated"; note="missing-auth row needs UMP_V2_NEGATIVE_AUTH_BASE_URL for server-side auth isolation"
  else
    status="fail"; error_code="missing_auth_did_not_fail_closed"; note="missing auth did not produce expected closed failure"
  fi
  record_redaction_scan "$row_dir" "$row" || { status="fail"; error_code="redaction_scan_failed"; note="redaction scan found sensitive marker"; }
  write_row_summary "$row_dir" "$row" "$status" "$latency_ms" "$error_code" "$note"
  write_row_event "$row" "$MODEL" "$PROFILE" "$status" "$latency_ms" "$error_code" "$note"
  status_rank_update "$status"
}

check_gates() {
  if [[ "${UMP_V2_LIVE_HARNESS:-}" != "1" ]]; then
    mark_blocked "missing env gate: UMP_V2_LIVE_HARNESS=1"
  fi
  if [[ "${UMP_V2_LIVE_COMPOSER_CODEX_CLI:-}" != "1" ]]; then
    mark_blocked "missing env gate: UMP_V2_LIVE_COMPOSER_CODEX_CLI=1"
  fi
  if [[ -n "${CI:-}" && "${UMP_V2_ALLOW_LIVE_TESTS_IN_CI:-}" != "1" ]]; then
    mark_blocked "missing CI live-test gate: UMP_V2_ALLOW_LIVE_TESTS_IN_CI=1"
  fi
}

check_prerequisites() {
  if ! command -v "$CODEX_BIN" >/dev/null 2>&1; then
    mark_blocked "missing Codex CLI: basename=$(basename "$CODEX_BIN") path_hash=$(path_hash "$CODEX_BIN")"
  fi
  if [[ ! -f "$CODEX_CONFIG" ]]; then
    mark_blocked "missing Codex config: basename=$(basename "$CODEX_CONFIG") path_hash=$(path_hash "$CODEX_CONFIG")"
  else
    if [[ -z "$CODEX_CONFIG_BACKUP" ]]; then
      CODEX_CONFIG_BACKUP="$(latest_config_backup)"
    fi
    if [[ -z "$CODEX_CONFIG_BACKUP" || ! -f "$CODEX_CONFIG_BACKUP" ]]; then
      mark_blocked "missing Codex config backup evidence: set CODEX_CONFIG_BACKUP or create ~/.codex/backups/composer-*/config.toml"
    fi
    for profile in "$PROFILE" "$FAST_PROFILE" "$LEGACY_PROFILE"; do
      if ! grep -Eq "^\\[profiles\\.${profile//./\\.}\\]|^\\[profiles\\.\"$profile\"\\]" "$CODEX_CONFIG"; then
        mark_blocked "missing Codex profile: $profile in basename=$(basename "$CODEX_CONFIG") path_hash=$(path_hash "$CODEX_CONFIG")"
      fi
    done
    check_compaction_safety "$CODEX_CONFIG"
  fi
  if [[ -z "$UMP_BASE_URL" ]]; then
    mark_blocked "missing UMP_BASE_URL; start an explicit proxy and pass its bound address, preferably from UMP_V2_LISTEN_ADDR=127.0.0.1:0"
  fi
  if [[ ! -f "$CODEX_CATALOG" ]]; then
    mark_blocked "missing model catalog: basename=$(basename "$CODEX_CATALOG") path_hash=$(path_hash "$CODEX_CATALOG")"
  else
    for model in "$MODEL" "$FAST_MODEL" "$LEGACY_MODEL"; do
      if ! grep -Fq '"'"$model"'"' "$CODEX_CATALOG"; then
        mark_blocked "composer-catalog-unavailable: missing model slug $model in basename=$(basename "$CODEX_CATALOG") path_hash=$(path_hash "$CODEX_CATALOG")"
      fi
    done
  fi
  if [[ ! -s "$CODEX_HOME_DIR/auth.json" && ! -s "$CODEX_HOME_DIR/credentials.json" ]]; then
    mark_blocked "missing Codex credentials: no auth.json or credentials.json under configured Codex home"
  fi
  if [[ -n "$UMP_BASE_URL" ]]; then
    if ! command -v curl >/dev/null 2>&1; then
      mark_blocked "missing dependency-light prerequisite: curl"
      return
    fi
    if ! curl -fsS --max-time 5 "$UMP_BASE_URL/health" > "$RUN_DIR/health.json" 2> "$RUN_DIR/health.err"; then
      mark_blocked "missing or unhealthy UMP endpoint: $UMP_BASE_URL/health"
    fi
    if ! curl -fsS --max-time 5 "$UMP_BASE_URL/v1/models" > "$RUN_DIR/models.json" 2> "$RUN_DIR/models.err"; then
      mark_blocked "missing UMP models endpoint: $UMP_BASE_URL/v1/models"
    else
      for model in "$MODEL" "$FAST_MODEL" "$LEGACY_MODEL"; do
        if ! grep -Fq '"'"$model"'"' "$RUN_DIR/models.json"; then
          mark_blocked "composer-catalog-unavailable: UMP /v1/models missing $model"
        fi
      done
    fi
  fi
}

write_redaction_report() {
  local status="pass"
  local finding_count=0
  if [[ ${REDACTION_FINDINGS+x} ]]; then
    finding_count=${#REDACTION_FINDINGS[@]}
  fi
  if (( finding_count > 0 )); then status="fail"; fi
  {
    printf '{\n'
    printf '  "redaction_version": "v1",\n'
    printf '  "status": "%s",\n' "$status"
    printf '  "placeholder": true,\n'
    printf '  "coverage": ["bearer_tokens", "api_keys", "oauth_tokens", "cookies", "client_secrets", "account_ids", "urls_with_query_strings", "home_paths"],\n'
    printf '  "findings": ['
    local first=1 finding
    if [[ ${REDACTION_FINDINGS+x} ]]; then
      for finding in "${REDACTION_FINDINGS[@]}"; do
        if [[ $first -eq 0 ]]; then printf ', '; fi
        first=0
        printf '"%s"' "$(json_escape "$finding")"
      done
    fi
    printf ']\n}\n'
  } > "$REDACTION_REPORT"
}

write_overall_summary() {
  write_redaction_report
  {
    printf '{\n'
    printf '  "status": "%s",\n' "$(json_escape "$OVERALL_STATUS")"
    printf '  "schema_version": 1,\n'
    printf '  "run_id": "%s",\n' "$(json_escape "$(basename "$RUN_DIR")")"
    printf '  "run_dir_hash": "%s",\n' "$(path_hash "$RUN_DIR")"
    printf '  "timestamp": "%s",\n' "$(json_escape "$STAMP")"
    printf '  "profile": "%s",\n' "$(json_escape "$PROFILE")"
    printf '  "model": "%s",\n' "$(json_escape "$MODEL")"
    printf '  "ump_base_url": "%s",\n' "$(json_escape "$UMP_BASE_URL")"
    printf '  "negative_auth_base_url": "%s",\n' "$(json_escape "$NEGATIVE_AUTH_BASE_URL")"
    printf '  "gates": {"UMP_V2_LIVE_HARNESS": "%s", "UMP_V2_LIVE_COMPOSER_CODEX_CLI": "%s", "UMP_V2_ALLOW_LIVE_TESTS_IN_CI": "%s", "enable_request_compression": "%s", "%s_remote_compaction_v2": "%s", "%s_remote_compaction_v2": "%s", "%s_remote_compaction_v2": "%s"},\n' "$(json_escape "${UMP_V2_LIVE_HARNESS:-absent}")" "$(json_escape "${UMP_V2_LIVE_COMPOSER_CODEX_CLI:-absent}")" "$(json_escape "${UMP_V2_ALLOW_LIVE_TESTS_IN_CI:-absent}")" "$(json_escape "$(read_features_value "$CODEX_CONFIG" enable_request_compression 2>/dev/null || printf absent)")" "$(json_escape "$PROFILE")" "$(json_escape "$(read_profile_feature_value "$CODEX_CONFIG" "$PROFILE" remote_compaction_v2 2>/dev/null || printf absent)")" "$(json_escape "$FAST_PROFILE")" "$(json_escape "$(read_profile_feature_value "$CODEX_CONFIG" "$FAST_PROFILE" remote_compaction_v2 2>/dev/null || printf absent)")" "$(json_escape "$LEGACY_PROFILE")" "$(json_escape "$(read_profile_feature_value "$CODEX_CONFIG" "$LEGACY_PROFILE" remote_compaction_v2 2>/dev/null || printf absent)")"
    printf '  "config": {"codex_config": {"basename": "%s", "path_hash": "%s", "content_hash": "%s"}, "backup": {"basename": "%s", "path_hash": "%s", "content_hash": "%s"}},\n' "$(json_escape "$(basename "$CODEX_CONFIG")")" "$(path_hash "$CODEX_CONFIG")" "$(sha256_file "$CODEX_CONFIG")" "$(json_escape "$(basename "${CODEX_CONFIG_BACKUP:-missing}")")" "$(path_hash "${CODEX_CONFIG_BACKUP:-missing}")" "$(sha256_file "${CODEX_CONFIG_BACKUP:-}")"
    printf '  "retry_policy": {"max_attempts": 3, "backoff_seconds": [10, 30]},\n'
    printf '  "blockers": ['
    local first=1 item
    if [[ ${BLOCKERS+x} ]]; then
      for item in "${BLOCKERS[@]}"; do
        if [[ $first -eq 0 ]]; then printf ', '; fi
        first=0
        printf '"%s"' "$(json_escape "$item")"
      done
    fi
    printf '],\n'
    printf '  "rows": ['
    first=1
    if [[ ${ROW_RESULTS+x} ]]; then
      for item in "${ROW_RESULTS[@]}"; do
        if [[ $first -eq 0 ]]; then printf ', '; fi
        first=0
        printf '"%s"' "$(json_escape "$item")"
      done
    fi
    printf '],\n'
    printf '  "artifacts": {"events": "events.jsonl", "harness_env": "harness-env.txt", "blockers": "live-blockers.txt", "redaction_report": "redaction-report.json"}\n'
    printf '}\n'
  } > "$SUMMARY_JSON"
}

run_matrix() {
  run_codex_row "streaming_chat" 'Say exactly "stream-ok", then count 1 through 5 with one number per line.' 'stream-ok|"delta"|"event"|1.*2.*3.*4.*5'
  run_reasoning_row
  run_continuation_row
  run_codex_row "single_tool_call" 'Use the shell tool to run pwd. Then report only the basename of the current directory.' 'unified-model-proxy-v2.*(tool|shell|pwd)|(tool|shell|pwd).*unified-model-proxy-v2'
  run_codex_row "parallel_tool_calling" 'Inspect README.md and Cargo.toml in parallel, then give one fact from each file.' 'README\.md.*Cargo\.toml|Cargo\.toml.*README\.md'
  run_context_row
  run_negative_model_row
  run_missing_auth_row
}

main() {
  if [[ "${1:-}" == "--help" || "${1:-}" == "-h" ]]; then
    usage
    exit 0
  fi

  {
    printf 'script=%s\n' "$SCRIPT_NAME"
    printf 'repo_root_hash=%s\n' "$(path_hash "$REPO_ROOT")"
    printf 'run_id=%s\n' "$(basename "$RUN_DIR")"
    printf 'run_dir_hash=%s\n' "$(path_hash "$RUN_DIR")"
    printf 'codex_bin_basename=%s\n' "$(basename "$CODEX_BIN")"
    printf 'codex_bin_path_hash=%s\n' "$(path_hash "$CODEX_BIN")"
    printf 'codex_config_basename=%s\n' "$(basename "$CODEX_CONFIG")"
    printf 'codex_config_path_hash=%s\n' "$(path_hash "$CODEX_CONFIG")"
    printf 'codex_config_sha256=%s\n' "$(sha256_file "$CODEX_CONFIG")"
    if [[ -f "$CODEX_CONFIG" ]]; then
      printf 'codex_feature_enable_request_compression=%s\n' "$(read_features_value "$CODEX_CONFIG" enable_request_compression)"
      printf 'codex_profile_%s_remote_compaction_v2=%s\n' "$PROFILE" "$(read_profile_feature_value "$CODEX_CONFIG" "$PROFILE" remote_compaction_v2)"
      printf 'codex_profile_%s_remote_compaction_v2=%s\n' "$FAST_PROFILE" "$(read_profile_feature_value "$CODEX_CONFIG" "$FAST_PROFILE" remote_compaction_v2)"
      printf 'codex_profile_%s_remote_compaction_v2=%s\n' "$LEGACY_PROFILE" "$(read_profile_feature_value "$CODEX_CONFIG" "$LEGACY_PROFILE" remote_compaction_v2)"
    fi
    if [[ -z "$CODEX_CONFIG_BACKUP" ]]; then
      CODEX_CONFIG_BACKUP="$(latest_config_backup)"
    fi
    printf 'codex_config_backup_basename=%s\n' "$(basename "${CODEX_CONFIG_BACKUP:-missing}")"
    printf 'codex_config_backup_path_hash=%s\n' "$(path_hash "${CODEX_CONFIG_BACKUP:-missing}")"
    printf 'codex_config_backup_sha256=%s\n' "$(sha256_file "${CODEX_CONFIG_BACKUP:-}")"
    printf 'codex_catalog_basename=%s\n' "$(basename "$CODEX_CATALOG")"
    printf 'codex_catalog_path_hash=%s\n' "$(path_hash "$CODEX_CATALOG")"
    printf 'codex_home_hash=%s\n' "$(path_hash "$CODEX_HOME_DIR")"
    printf 'ump_base_url=%s\n' "${UMP_BASE_URL:-}"
    printf 'codex_provider_base_url=%s\n' "${CODEX_PROVIDER_BASE_URL:-}"
    printf 'negative_auth_base_url=%s\n' "${NEGATIVE_AUTH_BASE_URL:-}"
    printf 'proxy_log_basename=%s\n' "$(basename "${UMP_PROXY_LOG:-missing}")"
    printf 'proxy_log_path_hash=%s\n' "$(path_hash "${UMP_PROXY_LOG:-missing}")"
  } > "$RUN_DIR/harness-env.txt"

  check_gates
  if [[ "$OVERALL_STATUS" == "live-blocked" ]]; then
    write_overall_summary
    printf 'live-blocked before prerequisites; run_id=%s run_dir_hash=%s summary=summary.json\n' "$(basename "$RUN_DIR")" "$(path_hash "$RUN_DIR")" >&2
    exit 2
  fi

  check_prerequisites
  if [[ "$OVERALL_STATUS" == "live-blocked" ]]; then
    write_overall_summary
    printf 'live-blocked before live matrix; run_id=%s run_dir_hash=%s summary=summary.json\n' "$(basename "$RUN_DIR")" "$(path_hash "$RUN_DIR")" >&2
    exit 2
  fi

  run_matrix
  write_overall_summary
  printf 'wrote live harness artifacts: run_id=%s run_dir_hash=%s\n' "$(basename "$RUN_DIR")" "$(path_hash "$RUN_DIR")"

  case "$OVERALL_STATUS" in
    pass|warn) exit 0 ;;
    live-blocked) exit 2 ;;
    *) exit 1 ;;
  esac
}

main "$@"
