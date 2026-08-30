#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT_DIR="$SCRIPT_DIR/.."

echo "=== BRK Quick Release ==="
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

CURRENT_VERSION=$(sed -n 's/^package\.version = "\([^"]*\)"/\1/p' "$ROOT_DIR/Cargo.toml")
RESUME_RELEASE=false

if [ "$RELEASE_ARG" = "$CURRENT_VERSION" ] &&
    git -C "$ROOT_DIR" rev-parse --verify --quiet "refs/tags/v$CURRENT_VERSION" >/dev/null; then
    RESUME_RELEASE=true
    echo "Resuming v$CURRENT_VERSION"
    echo ""
fi

# Load tokens
if [ -f "$SCRIPT_DIR/.tokens" ]; then
    source "$SCRIPT_DIR/.tokens"
fi

# ============================================================================
# 0. VERIFY TOKENS
# ============================================================================

echo "=== Verifying tokens ==="
echo ""

echo "--- npm ---"
npm whoami || { echo "npm not authenticated. Run: npm login"; exit 1; }
echo ""

echo "--- PyPI ---"
if [ -z "$UV_PUBLISH_TOKEN" ]; then
    echo "UV_PUBLISH_TOKEN not set. Add it to scripts/.tokens"
    exit 1
fi
echo "OK"
echo ""

# ============================================================================
# 1. BUILD
# ============================================================================

if [ "$RESUME_RELEASE" = false ]; then
    echo "=== Building ==="
    echo ""

    echo "--- Rust ---"
    cd "$ROOT_DIR"
    cargo build --workspace --release
    echo ""

    echo "--- JavaScript ---"
    cd "$ROOT_DIR/modules/bitview-client"
    # JS doesn't need build step, just verify it loads
    node -e "import('./index.js')"
    echo "OK"
    echo ""

    echo "--- Python ---"
    cd "$ROOT_DIR/packages/bitview_client"
    uv build
    echo ""

    # ============================================================================
    # 2. GENERATE DOCS
    # ============================================================================

    echo "=== Generating docs ==="
    echo ""

    echo "--- JavaScript ---"
    "$SCRIPT_DIR/js-docs.sh"
    echo ""

    echo "--- Python ---"
    "$SCRIPT_DIR/python-docs.sh"
    echo ""

    # Commit generated docs
    cd "$ROOT_DIR"
    git add -A
    git commit -m "docs: update generated docs" || echo "No doc changes to commit"
    echo ""
fi

# ============================================================================
# 3. CARGO RELEASE (Rust crates)
# ============================================================================

echo "=== Rust release ==="
echo ""

cd "$ROOT_DIR"

if [ "$RESUME_RELEASE" = true ]; then
    echo "Version v$CURRENT_VERSION and its tag already exist; skipping version bump and tag"
    VERSION="$CURRENT_VERSION"
else
    # Version bump, commit, and tag (but don't publish yet)
    cargo release "$RELEASE_ARG" --execute --no-publish --no-confirm
    VERSION=$(grep '^package.version' "$ROOT_DIR/Cargo.toml" | sed 's/.*= *"//' | sed 's/".*//')
fi

# Publish crates, skipping versions already available in the registry
"$SCRIPT_DIR/rust-publish.sh"

echo ""
echo "Released Rust crates at version: $VERSION"

# ============================================================================
# 4. JAVASCRIPT PACKAGE
# ============================================================================

echo ""
echo "=== JavaScript package ==="
"$SCRIPT_DIR/js-publish.sh" "$VERSION"
echo ""

# ============================================================================
# 5. PYTHON PACKAGE
# ============================================================================

echo ""
echo "=== Python package ==="
"$SCRIPT_DIR/python-publish.sh" "$VERSION"
echo ""

# ============================================================================
# 6. MCP REGISTRY
# ============================================================================

echo ""
echo "=== MCP Registry ==="
"$SCRIPT_DIR/mcp-publish.sh" "$VERSION"
echo ""

# ============================================================================
# DONE
# ============================================================================

echo "=== Done! ==="
echo "Released v$VERSION"
