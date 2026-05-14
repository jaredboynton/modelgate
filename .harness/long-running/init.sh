#!/usr/bin/env sh
set -eu

root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$root"

mkdir -p .harness/long-running .harness/outputs .harness/coordination/handoffs

if [ ! -f .harness/long-running/progress.md ]; then
  cat > .harness/long-running/progress.md <<'EOF'
# Long-Running Progress

## Current Goal

## Active Scope

## Latest Evidence

## Next Safe Action

## Blockers
EOF
fi

if [ ! -f .harness/long-running/handoff.md ]; then
  cat > .harness/long-running/handoff.md <<'EOF'
# Long-Running Handoff

## Stop Point

## Changed Files

## Validation Evidence

## Known Risks

## Resume Command
EOF
fi

printf 'Initialized long-running harness files under .harness/long-running\n'
