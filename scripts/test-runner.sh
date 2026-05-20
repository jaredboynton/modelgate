#!/bin/bash
if [ "$NEXTEST" = "1" ]; then
    exec "$@"
else
    # We are running under 'cargo test'.
    # Redirect to 'cargo nextest run'.
    mkdir -p target

    # Clean up any stale lockfiles from previous runs
    for f in target/nextest-run-*.tmp; do
        if [ "$f" != "target/nextest-run-$PPID.tmp" ] && [ -f "$f" ]; then
            rm -f "$f"
        fi
    done

    LOCKFILE="target/nextest-run-$PPID.tmp"
    if [ ! -f "$LOCKFILE" ]; then
        touch "$LOCKFILE"
        echo ">>> Automatically routing 'cargo test' to 'cargo nextest run'..."
        cargo nextest run "${@:2}"
        exit $?
    else
        # Sibling binary run within the same cargo test invocation
        exit 0
    fi
fi
