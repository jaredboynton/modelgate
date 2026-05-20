#!/usr/bin/env bash
# Fast local test wrapper that picks the right caching mode automatically.
#
# Two profiles, controlled by UMP_TEST_MODE (default: inner):
#
#   inner  -> CARGO_INCREMENTAL=1, RUSTC_WRAPPER unset.
#             Best for "edit one file, re-test". Cargo incremental keeps
#             per-crate dep-info hot. sccache would be silently bypassed
#             here (it refuses to cache incremental compiles), so we
#             explicitly unset it to avoid wasted disk writes.
#
#   cold   -> CARGO_INCREMENTAL=0, RUSTC_WRAPPER=sccache (if installed).
#             Best for first build after a branch hop, a `cargo clean`,
#             or a worktree switch. sccache hit-rate on this repo is
#             >98% so this beats incremental for cold builds.
#
# Any extra args are forwarded to `cargo nextest run`. Examples:
#
#   scripts/dev-test.sh                                  # full nextest, inner
#   UMP_TEST_MODE=cold scripts/dev-test.sh               # cold, sccache-backed
#   scripts/dev-test.sh -E 'test(integration_routes)'    # filter expression
#   scripts/dev-test.sh --profile ci                     # use ci nextest profile

set -euo pipefail

mode="${UMP_TEST_MODE:-inner}"

case "$mode" in
  inner)
    unset RUSTC_WRAPPER || true
    export CARGO_INCREMENTAL=1
    ;;
  cold)
    export CARGO_INCREMENTAL=0
    if command -v sccache >/dev/null 2>&1; then
      export RUSTC_WRAPPER=sccache
    else
      echo "scripts/dev-test.sh: sccache not on PATH; cold mode falls back to plain rustc" >&2
      unset RUSTC_WRAPPER || true
    fi
    ;;
  *)
    echo "scripts/dev-test.sh: unknown UMP_TEST_MODE=$mode (expected: inner|cold)" >&2
    exit 2
    ;;
esac

if ! command -v cargo-nextest >/dev/null 2>&1 && ! cargo nextest --version >/dev/null 2>&1; then
  echo "scripts/dev-test.sh: cargo-nextest is not installed. Run: cargo install cargo-nextest --locked" >&2
  exit 127
fi

exec cargo nextest run "$@"
