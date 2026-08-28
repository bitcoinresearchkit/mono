#!/bin/bash
set -eo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT_DIR="$SCRIPT_DIR/.."

cd "$ROOT_DIR"

# bitview_website embeds ignored assets through symlinks, so Cargo requires
# --allow-dirty even when the Git worktree itself is clean.
cargo release publish --package brk_logger --execute --no-confirm
cargo release publish --package importmap --execute --no-confirm

PUBLISH_LOG=$(mktemp -t brk-rust-publish)
trap 'rm -f "$PUBLISH_LOG"' EXIT

if cargo publish --package bitview_website --allow-dirty 2>&1 | tee "$PUBLISH_LOG"; then
    :
elif grep -q "already exists on" "$PUBLISH_LOG"; then
    echo "bitview_website is already published; skipping"
else
    exit 1
fi

cargo release publish --workspace --execute --no-confirm

echo "Done!"
