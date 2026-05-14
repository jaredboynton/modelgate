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

check_path() {
  path=$1
  reason=
  case "$path" in
    .env|*/.env|.env.*|*/.env.*)
      case "$path" in
        *.example|*.sample|*.template|*.md) reason= ;;
        *) reason="runtime env file" ;;
      esac
      ;;
    .omx|.omx/*|*/.omx|*/.omx/*) reason="OMX runtime state" ;;
    target|target/*|*/target|*/target/*) reason="Cargo build output" ;;
    logs|logs/*|*/logs|*/logs/*|log|log/*|*/log|*/log/*|*.log) reason="runtime log artifact" ;;
    coverage|coverage/*|*/coverage|*/coverage/*|.coverage|*/.coverage|lcov.info|*/lcov.info|*.lcov) reason="coverage artifact" ;;
    captures|captures/*|*/captures|*/captures/*|live-captures|live-captures/*|*/live-captures|*/live-captures/*|live-validation|live-validation/*|*/live-validation|*/live-validation/*|*.har|*.pcap|*.pcapng) reason="live/runtime capture" ;;
  esac

  if [ -n "$reason" ]; then
    printf '%s\t%s\n' "$path" "$reason" >> "$failures"
  fi
}

list_paths | sort -u | while IFS= read -r path; do
  [ -n "$path" ] || continue
  check_path "$path"
done

if [ -s "$failures" ]; then
  echo "GC hard fail: tracked or staged runtime artifacts were found." >&2
  echo "Remove these files from git and keep them ignored/local:" >&2
  while IFS='\t' read -r path reason; do
    printf '  - %s (%s)\n' "$path" "$reason" >&2
  done < "$failures"
  echo "Remediation: git rm --cached <path> for accidental tracked files; move live captures under ignored local scratch." >&2
  exit 1
fi

echo "OK: no tracked/staged runtime artifacts found."
