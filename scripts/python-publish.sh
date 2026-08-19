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

# Update version in pyproject.toml
sed -i '' 's/^version = "[^"]*"/version = "'"$VERSION"'"/' pyproject.toml
echo "Updated pyproject.toml to $VERSION"

# Update VERSION in __init__.py
sed -i '' 's/VERSION = "v[^"]*"/VERSION = "v'"$VERSION"'"/' bitview_client/__init__.py
echo "Updated __init__.py VERSION to v$VERSION"

# Clean old build artifacts
rm -rf dist

uv build
uv publish
