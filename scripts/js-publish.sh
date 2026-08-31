#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT_DIR="$SCRIPT_DIR/.."

VERSION="$1"
if [ -z "$VERSION" ]; then
    echo "Usage: $0 <version>"
    exit 1
fi

cd "$ROOT_DIR/modules/bitview-client"

# Update version in package.json
sed -i '' 's/"version": "[^"]*"/"version": "'"$VERSION"'"/' package.json
echo "Updated package.json to $VERSION"

# Update VERSION in index.js
sed -i '' 's/VERSION = "v[^"]*"/VERSION = "v'"$VERSION"'"/' index.js
echo "Updated index.js VERSION to v$VERSION"

# Determine npm tag based on version
if [[ "$VERSION" == *"-alpha"* ]]; then
    NPM_TAG="alpha"
elif [[ "$VERSION" == *"-beta"* ]]; then
    NPM_TAG="beta"
elif [[ "$VERSION" == *"-rc"* ]]; then
    NPM_TAG="rc"
else
    NPM_TAG="latest"
fi

if [ -z "${NPM_CONFIG_OTP:-}" ] && [ -t 0 ]; then
    read -r -s -p "npm OTP (leave blank for a bypass-2FA token): " NPM_OTP
    echo ""
fi

if [ -n "${NPM_OTP:-}" ]; then
    NPM_CONFIG_OTP="$NPM_OTP" npm publish --access public --tag "$NPM_TAG"
else
    npm publish --access public --tag "$NPM_TAG"
fi
