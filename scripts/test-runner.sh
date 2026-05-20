#!/bin/bash
if [ "$NEXTEST" = "1" ]; then
    exec "$@"
else
    echo ""
    echo "========================================================================="
    echo " ERROR: Direct 'cargo test' execution is blocked in this repository."
    echo " Please run tests using cargo-nextest instead:"
    echo "     cargo nextest run"
    echo "========================================================================="
    echo ""
    exit 1
fi
