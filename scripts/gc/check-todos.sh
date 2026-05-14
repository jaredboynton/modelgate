#!/usr/bin/env sh
set -eu

repo_root=$(git rev-parse --show-toplevel 2>/dev/null || pwd)
cd "$repo_root"

report=$(mktemp)
trap 'rm -f "$report"' EXIT

for path in $(git ls-files); do
  [ -f "$path" ] || continue
  file "$path" 2>/dev/null | grep -Eq 'text|empty|JSON|TOML|YAML|XML|shell|script|Rust|Markdown|ASCII|UTF-8' || continue
  grep -nE 'TODO|FIXME|HACK|XXX' "$path" 2>/dev/null >> "$report" || true
done

if [ -s "$report" ]; then
  echo "GC report: TODO/FIXME/HACK inventory found."
  cat "$report"
else
  echo "OK: no TODO/FIXME/HACK markers found in tracked files."
fi

exit 0
