#!/usr/bin/env sh
set -eu

repo_root=$(git rev-parse --show-toplevel 2>/dev/null || pwd)
cd "$repo_root"

report=$(mktemp)
trap 'rm -f "$report"' EXIT

is_text_file() {
  file "$1" 2>/dev/null | grep -Eq 'text|empty|JSON|TOML|YAML|XML|shell|script|Rust|Markdown|ASCII|UTF-8'
}

classify() {
  path=$1
  line=$2
  case "$path" in
    Cargo.toml|Cargo.lock|*/Cargo.toml|*/Cargo.lock|package.json|*/package.json|pnpm-lock.yaml|*/pnpm-lock.yaml|package-lock.json|*/package-lock.json|yarn.lock|*/yarn.lock)
      if printf '%s' "$line" | grep -Eq 'path[[:space:]]*=|file:|/Users/|/home/'; then
        printf 'pre-remote blocker'
        return
      fi
      ;;
  esac
  printf 'report-only'
}

git ls-files | while IFS= read -r path; do
  [ -f "$path" ] || continue
  [ "$path" = "scripts/gc/check-local-paths.sh" ] && continue
  is_text_file "$path" || continue
  grep -nE '(^|[^A-Za-z0-9_])(/Users/|/home/|/private/var/|/var/folders/|/tmp/|/opt/homebrew/|/usr/local/)' "$path" 2>/dev/null |
  while IFS= read -r match; do
    line_no=${match%%:*}
    line_text=${match#*:}
    case "$line_text" in
      *"grep "*"/Users/"*|*"git grep "*"/Users/"*) continue ;;
    esac
    label=$(classify "$path" "$line_text")
    printf '%s:%s [%s] %s\n' "$path" "$line_no" "$label" "$line_text" >> "$report"
  done
done

if [ -s "$report" ]; then
  echo "GC report: absolute local paths found in tracked files."
  echo "Manifest dependency paths labeled pre-remote blocker must be resolved before remote/distribution claims."
  cat "$report"
else
  echo "OK: no absolute local paths found in tracked files."
fi

exit 0
