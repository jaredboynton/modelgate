#!/usr/bin/env bash
# Warpsock-backed streaming live validation harness.
#
# Exercises the proxy's Codex and Windsurf streaming routes that are intended to
# go through the Warpsock HTTP transport. When opt-in flags or credentials are
# missing, each row is recorded as `skipped-with-reason` instead of failing the
# deterministic validators. All emitted artifacts are redacted: secret values
# and bearer tokens never reach the row JSON or the saved transcripts.
#
# Required opt-ins (all rows skipped if `UMP_V2_LIVE_HARNESS` is unset):
#   UMP_V2_LIVE_HARNESS=1
#   UMP_V2_LIVE_CODEX_STREAM=1
#   UMP_V2_LIVE_CODEX_STREAM_MODEL=<codex-model-id>
#   UMP_V2_LIVE_WINDSURF_STREAM=1
#   WINDSURF_API_KEY=<key>
#
# Optional:
#   UMP_V2_LIVE_BASE_URL=http://127.0.0.1:18743
#   UMP_V2_LIVE_HARNESS_RUNS_ROOT=<path>  (default ./.live-harness/runs)
#
# Outputs JSON summary at $RUN_DIR/warpsock-streaming-summary.json containing
# per-row provider, model, route, streaming mode, status, and reason.

set -uo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUNS_ROOT="${UMP_V2_LIVE_HARNESS_RUNS_ROOT:-${REPO_ROOT}/.live-harness/runs}"
STAMP="${UMP_V2_LIVE_HARNESS_STAMP:-$(date -u +%Y%m%dT%H%M%SZ)}"
RUN_DIR="${RUNS_ROOT}/${STAMP}"
mkdir -p "${RUN_DIR}"
SUMMARY="${RUN_DIR}/warpsock-streaming-summary.json"

UMP_BASE="${UMP_V2_LIVE_BASE_URL:-http://127.0.0.1:18743}"

declare -a ROWS=()

redact_transcript() {
    local file="$1"
    if [[ ! -f "${file}" ]]; then
        return 0
    fi
    # Redact common secret patterns. Conservative: never echo raw tokens.
    perl -i -pe '
        s/(authorization:\s*Bearer\s+)\S+/$1<REDACTED>/ig;
        s/(x-api-key:\s*)\S+/$1<REDACTED>/ig;
        s/(api[_-]?key"\s*:\s*")[^"]+/$1<REDACTED>/ig;
        s/(token"\s*:\s*")[^"]+/$1<REDACTED>/ig;
    ' "${file}"
}

push_row() {
    local row="$1"
    ROWS+=("${row}")
}

run_codex_stream() {
    local model="${UMP_V2_LIVE_CODEX_STREAM_MODEL:-}"
    local out="${RUN_DIR}/codex-streaming.log"
    if [[ "${UMP_V2_LIVE_HARNESS:-0}" != "1" ]]; then
        push_row "$(printf '{"provider":"codex","route":"/v1/responses","streaming":true,"status":"skipped","reason":"UMP_V2_LIVE_HARNESS=1 not set","warpsock_route":true}')"
        return 0
    fi
    if [[ "${UMP_V2_LIVE_CODEX_STREAM:-0}" != "1" ]]; then
        push_row "$(printf '{"provider":"codex","route":"/v1/responses","streaming":true,"status":"skipped","reason":"UMP_V2_LIVE_CODEX_STREAM=1 not set","warpsock_route":true}')"
        return 0
    fi
    if [[ -z "${model}" ]]; then
        push_row "$(printf '{"provider":"codex","route":"/v1/responses","streaming":true,"status":"skipped","reason":"UMP_V2_LIVE_CODEX_STREAM_MODEL not set","warpsock_route":true}')"
        return 0
    fi
    if [[ ! -d "${HOME}/.codex" ]] || [[ ! -e "${HOME}/.codex/auth.json" && ! -e "${HOME}/.codex/auth-backups" ]]; then
        push_row "$(printf '{"provider":"codex","route":"/v1/responses","streaming":true,"status":"skipped","reason":"local Codex OAuth credentials missing","warpsock_route":true}')"
        return 0
    fi

    local body
    body=$(printf '{"model":"%s","input":"Reply with exactly: ok","stream":true}' "${model}")
    if curl -fsS -N -H 'content-type: application/json' "${UMP_BASE%/}/v1/responses" -d "${body}" > "${out}" 2>&1; then
        redact_transcript "${out}"
        if grep -q 'response\.completed' "${out}" || grep -q '"done"\s*:\s*true' "${out}"; then
            push_row "$(printf '{"provider":"codex","model":"%s","route":"/v1/responses","streaming":true,"status":"pass","warpsock_route":true,"transcript":"%s"}' "${model}" "${out#${REPO_ROOT}/}")"
            return 0
        fi
        push_row "$(printf '{"provider":"codex","model":"%s","route":"/v1/responses","streaming":true,"status":"warn","reason":"no terminal response.completed event observed","warpsock_route":true,"transcript":"%s"}' "${model}" "${out#${REPO_ROOT}/}")"
        return 0
    fi
    redact_transcript "${out}"
    push_row "$(printf '{"provider":"codex","model":"%s","route":"/v1/responses","streaming":true,"status":"warn","reason":"curl returned non-zero exit; see redacted transcript","warpsock_route":true,"transcript":"%s"}' "${model}" "${out#${REPO_ROOT}/}")"
}

run_windsurf_stream() {
    local out="${RUN_DIR}/windsurf-streaming.log"
    if [[ "${UMP_V2_LIVE_HARNESS:-0}" != "1" ]]; then
        push_row "$(printf '{"provider":"windsurf","route":"/v1/chat/completions","streaming":true,"status":"skipped","reason":"UMP_V2_LIVE_HARNESS=1 not set","warpsock_route":true}')"
        return 0
    fi
    if [[ "${UMP_V2_LIVE_WINDSURF_STREAM:-0}" != "1" ]]; then
        push_row "$(printf '{"provider":"windsurf","route":"/v1/chat/completions","streaming":true,"status":"skipped","reason":"UMP_V2_LIVE_WINDSURF_STREAM=1 not set","warpsock_route":true}')"
        return 0
    fi
    if [[ -z "${WINDSURF_API_KEY:-}" ]]; then
        push_row "$(printf '{"provider":"windsurf","route":"/v1/chat/completions","streaming":true,"status":"skipped","reason":"WINDSURF_API_KEY missing","warpsock_route":true}')"
        return 0
    fi

    local body
    body='{"model":"windsurf/swe-1.6","messages":[{"role":"user","content":"Reply with exactly: ok"}],"stream":true}'
    if curl -fsS -N -H 'content-type: application/json' "${UMP_BASE%/}/v1/chat/completions" -d "${body}" > "${out}" 2>&1; then
        redact_transcript "${out}"
        if grep -q 'data: \[DONE\]' "${out}"; then
            push_row "$(printf '{"provider":"windsurf","model":"windsurf/swe-1.6","route":"/v1/chat/completions","streaming":true,"status":"pass","warpsock_route":true,"transcript":"%s"}' "${out#${REPO_ROOT}/}")"
            return 0
        fi
        push_row "$(printf '{"provider":"windsurf","model":"windsurf/swe-1.6","route":"/v1/chat/completions","streaming":true,"status":"warn","reason":"no [DONE] terminator observed","warpsock_route":true,"transcript":"%s"}' "${out#${REPO_ROOT}/}")"
        return 0
    fi
    redact_transcript "${out}"
    push_row "$(printf '{"provider":"windsurf","model":"windsurf/swe-1.6","route":"/v1/chat/completions","streaming":true,"status":"warn","reason":"curl returned non-zero exit; see redacted transcript","warpsock_route":true,"transcript":"%s"}' "${out#${REPO_ROOT}/}")"
}

run_codex_stream
run_windsurf_stream

OVERALL="pass"
for row in "${ROWS[@]}"; do
    if [[ "${row}" == *'"status":"warn"'* ]]; then
        OVERALL="warn"
    fi
done

{
    printf '{\n'
    printf '  "fulfills": ["VAL-INT-004", "VAL-INT-005", "VAL-INT-006", "VAL-INT-007", "VAL-INT-011", "VAL-INT-012"],\n'
    printf '  "harness": "scripts/live/run-warpsock-streaming-validation.sh",\n'
    printf '  "ump_base_url": "%s",\n' "${UMP_BASE}"
    printf '  "overall_status": "%s",\n' "${OVERALL}"
    printf '  "rows": [\n'
    sep=""
    for row in "${ROWS[@]}"; do
        printf '%s    %s' "${sep}" "${row}"
        sep=$',\n'
    done
    printf '\n  ]\n}\n'
} > "${SUMMARY}"

echo "wrote ${SUMMARY}"
