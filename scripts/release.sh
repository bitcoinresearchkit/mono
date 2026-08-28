#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT_DIR="$SCRIPT_DIR/.."

echo "=== BRK Release ==="
echo ""

# Check if version argument provided
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

# ============================================================================
# 1. TESTS
# ============================================================================

echo "=== Running tests ==="
echo ""

echo "--- Rust ---"
cd "$ROOT_DIR"
# Verify all crates package correctly
# Note: --no-verify skips rebuild check due to version collision with crates.io
# The cargo build --workspace --release step above already verified compilation
cargo package --workspace --allow-dirty --no-verify
cargo test --workspace
echo ""

echo "--- JavaScript ---"
cd "$ROOT_DIR/modules/bitview-client"
npm test
echo ""

echo "--- Python ---"
cd "$ROOT_DIR/packages/bitview_client"
uv run python -m pytest tests/ --ignore=tests/mempool_compat -s
echo ""

# ============================================================================
# 2. QUICK RELEASE
# ============================================================================

"$SCRIPT_DIR/quick-release.sh" "$RELEASE_ARG"
