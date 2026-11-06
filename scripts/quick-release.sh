#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT_DIR="$SCRIPT_DIR/.."

echo "=== BRK Quick Release ==="
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

# npm and PyPI drop build metadata, and cargo-release's `{{version}}` leaves it out: the packages
# would disagree on their version.
case "$RELEASE_ARG" in
    *+*)
        echo "Build metadata (+...) isn't supported: npm, PyPI and the manifests would disagree"
        exit 1
        ;;
esac

CURRENT_VERSION=$(sed -n 's/^package\.version = "\([^"]*\)"/\1/p' "$ROOT_DIR/Cargo.toml")
RESUME_RELEASE=false

if [ "$RELEASE_ARG" = "$CURRENT_VERSION" ] &&
    git -C "$ROOT_DIR" rev-parse --verify --quiet "refs/tags/v$CURRENT_VERSION" >/dev/null; then
    RESUME_RELEASE=true
    # What's published is what's tagged: only from the tagged commit itself.
    if [ "$(git -C "$ROOT_DIR" rev-parse HEAD)" != "$(git -C "$ROOT_DIR" rev-parse "v$CURRENT_VERSION^{commit}")" ]; then
        echo "HEAD isn't v$CURRENT_VERSION's commit: check it out (git checkout v$CURRENT_VERSION) to resume"
        exit 1
    fi
    echo "Resuming v$CURRENT_VERSION"
    echo ""
fi

# A release commits and tags what's in the tree, and publishes what's tagged: only from a clean one
# (other work in this checkout would ride along), resumed or not.
if [ -n "$(git -C "$ROOT_DIR" status --porcelain)" ]; then
    echo "The working tree has changes: commit or move them first"
    exit 1
fi

if [ -f "$SCRIPT_DIR/.tokens" ]; then
    source "$SCRIPT_DIR/.tokens"
fi

echo "=== Checking credentials ==="
echo ""

echo "--- npm ---"
(cd "$ROOT_DIR/modules/bitview-client" && bun pm whoami) || { echo "npm not authenticated. Run: bunx --bun npm login"; exit 1; }
echo ""

echo "--- PyPI ---"
if [ -z "$UV_PUBLISH_TOKEN" ]; then
    echo "UV_PUBLISH_TOKEN not set. Add it to scripts/.tokens"
    exit 1
fi
echo "OK"
echo ""

if [ "$RESUME_RELEASE" = false ]; then
    echo "=== Building ==="
    echo ""

    echo "--- Rust ---"
    cd "$ROOT_DIR"
    cargo build --workspace --release
    echo ""

    echo "--- JavaScript ---"
    cd "$ROOT_DIR/modules/bitview-client"
    # JavaScript needs no build step; importing checks the generated module, tsc its types (the
    # declarations published from them).
    bun -e "await import('./index.js')"
    bunx --bun -p typescript@5 tsc -p tsconfig.json --noEmit
    echo "OK"
    echo ""

    echo "--- Python ---"
    cd "$ROOT_DIR/packages/bitview_client"
    uv build
    echo ""

    echo "=== Generating docs ==="
    echo ""

    echo "--- JavaScript ---"
    "$SCRIPT_DIR/js-docs.sh"
    echo ""

    echo "--- Python ---"
    "$SCRIPT_DIR/python-docs.sh"
    echo ""

    cd "$ROOT_DIR"
    DOCS=(modules/bitview-client/docs packages/bitview_client/DOCS.md)
    git add -- "${DOCS[@]}"
    git commit -m "docs: update generated docs" -- "${DOCS[@]}" || echo "No doc changes to commit"
    echo ""
fi

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

# Publish crates, skipping versions already available in the registry. A resume runs from the tag's
# own commit (checked above), detached HEAD included.
RELEASE_RESUME="$RESUME_RELEASE" "$SCRIPT_DIR/rust-publish.sh"

echo ""
echo "Released Rust crates at version: $VERSION"

echo ""
echo "=== JavaScript package ==="
"$SCRIPT_DIR/js-publish.sh" "$VERSION"
echo ""

echo ""
echo "=== Python package ==="
"$SCRIPT_DIR/python-publish.sh" "$VERSION"
echo ""

echo ""
echo "=== MCP Registry ==="
"$SCRIPT_DIR/mcp-publish.sh" "$VERSION"
echo ""

echo "Released v$VERSION"
