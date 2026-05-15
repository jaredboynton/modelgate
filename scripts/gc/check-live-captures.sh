#!/usr/bin/env sh
set -eu

repo_root=$(git rev-parse --show-toplevel 2>/dev/null || pwd)
cd "$repo_root"

failures=$(mktemp)
trap 'rm -f "$failures"' EXIT

list_paths() {
  git ls-files
  git diff --cached --name-only --diff-filter=ACMRT
}

is_synthetic_fixture() {
  path=$1
  lower=$(printf '%s' "$path" | tr '[:upper:]' '[:lower:]')

  case "$lower" in
    tests/fixtures/*|fixtures/*)
      case "$lower" in
        *synthetic*|*redacted*|*sample*) return 0 ;;
      esac
      ;;
  esac

  return 1
}

check_path() {
  path=$1
  lower=$(printf '%s' "$path" | tr '[:upper:]' '[:lower:]')
  reason=

  case "$lower" in
    .live-harness|.live-harness/*|*/.live-harness|*/.live-harness/*) reason="live harness artifact" ;;
    live-captures|live-captures/*|*/live-captures|*/live-captures/*|live-validation|live-validation/*|*/live-validation|*/live-validation/*) reason="live capture directory" ;;
    *.har|*.pcap|*.pcapng) reason="browser/network capture artifact" ;;
    *.jsonl|*.ndjson) reason="raw JSONL transcript artifact" ;;
    *raw-transcript*|*raw_transcript*|*transcript.raw*|*request-response*|*request_response*) reason="raw transcript artifact" ;;
    .env|*/.env|.env.*|*/.env.*) reason="runtime env file" ;;
    *auth.json|*oauth*.json|*session*.json|*cookies*.json|*cookie*.json|*token*.json|*tokens*.json|*credentials*.json|*credential*.json|*secrets*.json|*secret*.json) reason="auth or credential artifact" ;;
  esac

  if [ -n "$reason" ] && ! is_synthetic_fixture "$path"; then
    printf '%s\t%s\n' "$path" "$reason" >> "$failures"
  fi
}

list_paths | sort -u | while IFS= read -r path; do
  [ -n "$path" ] || continue
  check_path "$path"
done

if [ -s "$failures" ]; then
  echo "GC hard fail: tracked or staged live capture/auth/env artifacts were found." >&2
  echo "Remove live captures and credential-bearing artifacts from git; commit redacted synthetic fixtures only." >&2
  while IFS="$(printf '\t')" read -r path reason; do
    printf '  - %s (%s)\n' "$path" "$reason" >&2
  done < "$failures"
  echo "Remediation: git rm --cached <path>, move live files under ignored local scratch, or rename/redact fixtures clearly." >&2
  exit 1
fi

echo "OK: no tracked/staged live capture, auth, env, HAR, or raw transcript artifacts found."
