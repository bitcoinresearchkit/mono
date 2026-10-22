#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT_DIR="$SCRIPT_DIR/.."

cd "$ROOT_DIR/modules/bitview-client"
bunx --bun -p typedoc -p typedoc-plugin-markdown typedoc index.js --plugin typedoc-plugin-markdown --out docs --excludeNotDocumented --excludeExternals --basePath "$ROOT_DIR" --gitRevision main
