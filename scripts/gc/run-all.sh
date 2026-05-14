#!/usr/bin/env sh
set -eu

repo_root=$(git rev-parse --show-toplevel 2>/dev/null || pwd)
cd "$repo_root"

status=0
run_check() {
  name=$1
  shift
  echo "==> $name"
  if "$@"; then
    echo "==> $name: ok"
  else
    code=$?
    echo "==> $name: failed ($code)" >&2
    status=$code
  fi
}

run_check "tracked runtime artifacts" scripts/gc/check-tracked-runtime-artifacts.sh
run_check "secret marker filenames" scripts/gc/check-secret-markers.sh
run_check "absolute local paths report" scripts/gc/check-local-paths.sh
run_check "task marker inventory report" scripts/gc/check-todos.sh
if [ -x scripts/gc/check-large-files.sh ]; then
  run_check "large files report" scripts/gc/check-large-files.sh
fi

exit "$status"
