#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT_DIR="$SCRIPT_DIR/.."
REGISTRY_URL="https://registry.modelcontextprotocol.io"
MANIFEST="$ROOT_DIR/crates/bitview_mcp/server.json"

CHECK_ONLY=false
if [ "$1" = "--check" ]; then
    CHECK_ONLY=true
    shift
fi

VERSION="$1"
if [ -z "$VERSION" ]; then
    echo "Usage: $0 [--check] <version>"
    exit 1
fi

if [ -f "$SCRIPT_DIR/.tokens" ]; then
    source "$SCRIPT_DIR/.tokens"
fi

for COMMAND in curl jq mcp-publisher; do
    if ! command -v "$COMMAND" >/dev/null 2>&1; then
        echo "$COMMAND is required to publish to the MCP Registry"
        exit 1
    fi
done

if [ ! -f "$MANIFEST" ]; then
    echo "Missing $MANIFEST"
    echo "Generate it with: cargo run -p bitviewd --example bindgen --features bindgen"
    exit 1
fi

SERVER_NAME=$(jq -er '.name' "$MANIFEST")
MANIFEST_VERSION=$(jq -er '.version' "$MANIFEST")

if [ "$MANIFEST_VERSION" != "$VERSION" ]; then
    echo "MCP server.json version $MANIFEST_VERSION does not match release version $VERSION"
    exit 1
fi

echo "Validating $SERVER_NAME v$VERSION"
VALIDATION_RESPONSE=$(curl -sS --fail-with-body \
    -X POST \
    -H "Content-Type: application/json" \
    --data-binary "@$MANIFEST" \
    "$REGISTRY_URL/v0.1/validate")

if ! echo "$VALIDATION_RESPONSE" | jq -e '.valid == true' >/dev/null; then
    echo "MCP Registry validation failed:"
    echo "$VALIDATION_RESPONSE" | jq .
    exit 1
fi

SERVER_NAME_ENCODED=${SERVER_NAME/\//%2F}
LOOKUP_URL="$REGISTRY_URL/v0.1/servers/$SERVER_NAME_ENCODED/versions/$VERSION"
LOOKUP_RESPONSE=$(mktemp -t bitview-mcp-registry)
trap 'rm -f "$LOOKUP_RESPONSE"' EXIT

LOOKUP_STATUS=$(curl -sS -o "$LOOKUP_RESPONSE" -w "%{http_code}" "$LOOKUP_URL")
case "$LOOKUP_STATUS" in
    200)
        echo "$SERVER_NAME v$VERSION is already published; skipping"
        exit 0
        ;;
    404)
        ;;
    *)
        echo "MCP Registry lookup failed with HTTP $LOOKUP_STATUS:"
        cat "$LOOKUP_RESPONSE"
        exit 1
        ;;
esac

if [ "$CHECK_ONLY" = true ]; then
    echo "$SERVER_NAME v$VERSION is valid and not yet published"
    exit 0
fi

if [ -z "$MCP_GITHUB_TOKEN" ]; then
    echo "MCP_GITHUB_TOKEN not set. Add a GitHub token with read:org access to scripts/.tokens"
    exit 1
fi

cleanup() {
    rm -f "$LOOKUP_RESPONSE"
    mcp-publisher logout >/dev/null 2>&1 || true
}
trap cleanup EXIT

mcp-publisher logout >/dev/null 2>&1 || true
mcp-publisher login github --token "$MCP_GITHUB_TOKEN"
mcp-publisher publish "$MANIFEST"

VERIFY_STATUS=$(curl -sS -o "$LOOKUP_RESPONSE" -w "%{http_code}" "$LOOKUP_URL")
if [ "$VERIFY_STATUS" != "200" ]; then
    echo "MCP Registry verification failed with HTTP $VERIFY_STATUS:"
    cat "$LOOKUP_RESPONSE"
    exit 1
fi

PUBLISHED_NAME=$(jq -er '.server.name' "$LOOKUP_RESPONSE")
PUBLISHED_VERSION=$(jq -er '.server.version' "$LOOKUP_RESPONSE")

if [ "$PUBLISHED_NAME" != "$SERVER_NAME" ] || [ "$PUBLISHED_VERSION" != "$VERSION" ]; then
    echo "MCP Registry verification returned unexpected metadata"
    cat "$LOOKUP_RESPONSE"
    exit 1
fi

echo "Verified $PUBLISHED_NAME v$PUBLISHED_VERSION"
