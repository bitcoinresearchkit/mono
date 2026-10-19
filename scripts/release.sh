#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT_DIR="$SCRIPT_DIR/.."

echo "=== BRK Release ==="
echo ""

if [ -z "$1" ]; then
    echo "Usage: $0 <version|bump>"
    echo "Examples:"
    echo "  $0 0.1.0-alpha.3"
    echo "  $0 patch"
    echo "  $0 minor"
    echo "  $0 major"
    exit 1
fi

RELEASE_ARG="$1"
echo "Release argument: $RELEASE_ARG"
echo ""

echo "=== Running checks ==="
echo ""

echo "--- Rust ---"
cd "$ROOT_DIR"
# --no-verify skips the rebuild (the version can collide with crates.io's); quick-release.sh builds
# the workspace.
cargo package --workspace --allow-dirty --no-verify
cargo api
echo ""

"$SCRIPT_DIR/quick-release.sh" "$RELEASE_ARG"
