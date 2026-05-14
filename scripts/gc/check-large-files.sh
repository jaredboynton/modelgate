#!/usr/bin/env sh
set -eu

repo_root=$(git rev-parse --show-toplevel 2>/dev/null || pwd)
cd "$repo_root"

limit_bytes=${GC_LARGE_FILE_LIMIT_BYTES:-1048576}
report=$(mktemp)
trap 'rm -f "$report"' EXIT

is_allowed_large_file() {
  case "$1" in
    Cargo.lock) return 0 ;;
  esac
  return 1
}

git ls-files | while IFS= read -r path; do
  [ -f "$path" ] || continue
  is_allowed_large_file "$path" && continue
  size=$(wc -c < "$path" | tr -d ' ')
  if [ "$size" -gt "$limit_bytes" ]; then
    printf '%s\t%s bytes\n' "$path" "$size" >> "$report"
  fi
done

if [ -s "$report" ]; then
  echo "GC report: large tracked files found above ${limit_bytes} bytes."
  cat "$report"
  echo "Set GC_LARGE_FILE_LIMIT_BYTES to tune this report-only threshold."
else
  echo "OK: no large tracked files above ${limit_bytes} bytes."
fi

exit 0
