#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT_DIR="$SCRIPT_DIR/.."

VERSION="$1"
if [ -z "$VERSION" ]; then
    echo "Usage: $0 <version>"
    exit 1
fi

cd "$ROOT_DIR/packages/bitview_client"

# The release's own commit sets both (cargo-release: bitviewd's replacement, bindgen's hook): what's
# published is what's tagged.
PROJECT=$(sed -n 's/^version = "\([^"]*\)"/\1/p' pyproject.toml)
CONSTANT=$(sed -n 's/.*VERSION = "v\([^"]*\)".*/\1/p' bitview_client/__init__.py | head -1)
if [ "$PROJECT" != "$VERSION" ] || [ "$CONSTANT" != "$VERSION" ]; then
    echo "pyproject.toml ($PROJECT) or __init__.py VERSION ($CONSTANT) isn't $VERSION: the release commit sets them (cargo release)"
    exit 1
fi

rm -rf dist

uv build
uv publish
