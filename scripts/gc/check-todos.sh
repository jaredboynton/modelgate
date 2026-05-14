#!/usr/bin/env sh
set -eu

repo_root=$(git rev-parse --show-toplevel 2>/dev/null || pwd)
cd "$repo_root"

report=$(mktemp)
trap 'rm -f "$report"' EXIT

git ls-files | while IFS= read -r path; do
  [ -f "$path" ] || continue
  file "$path" 2>/dev/null | grep -Eq 'text|empty|JSON|TOML|YAML|XML|shell|script|Rust|Markdown|ASCII|UTF-8' || continue
  grep -nE '\b(T[O]DO|F[I]XME|H[A]CK|X[X]X)\b' "$path" 2>/dev/null >> "$report" || true
done

if [ -s "$report" ]; then
  echo "GC report: task marker inventory found."
  cat "$report"
else
  echo "OK: no task markers found in tracked files."
fi

exit 0
