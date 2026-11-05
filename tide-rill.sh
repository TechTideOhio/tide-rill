#!/usr/bin/env bash
# tide-rill: TideOS Eval Kit Tool 5 runner wrapper

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BIN_RELEASE="${SCRIPT_DIR}/target/release/tide-rill"
BIN_DEBUG="${SCRIPT_DIR}/target/debug/tide-rill"

if [ ! -f "$BIN_RELEASE" ] && [ ! -f "$BIN_DEBUG" ]; then
    echo "[tide-rill] Compiling release binary..."
    (cd "$SCRIPT_DIR" && cargo build --release)
fi

if [ -f "$BIN_RELEASE" ]; then
    exec "$BIN_RELEASE" "$@"
else
    exec "$BIN_DEBUG" "$@"
fi
