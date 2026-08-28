#!/bin/bash
set -eo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT_DIR="$SCRIPT_DIR/.."

cd "$ROOT_DIR"

publish_release_selection() {
    local label="$1"
    shift

    local publish_log
    publish_log=$(mktemp -t brk-rust-publish)

    if cargo release publish "$@" --execute --no-confirm 2>&1 | tee "$publish_log"; then
        rm -f "$publish_log"
    elif ! grep -q "uncommitted changes detected" "$publish_log" &&
        grep -q "disabled due to previous publish" "$publish_log" &&
        grep -q "no packages selected" "$publish_log"; then
        echo "$label is already published; skipping"
        rm -f "$publish_log"
    else
        rm -f "$publish_log"
        return 1
    fi
}

# bitview_website embeds ignored assets through symlinks, so Cargo requires
# --allow-dirty even when the Git worktree itself is clean. Publish its
# workspace dependencies first so Cargo can resolve the packaged crate.
publish_release_selection "brk_logger" --package brk_logger
publish_release_selection "importmap" --package importmap

PUBLISH_LOG=$(mktemp -t brk-rust-publish)
trap 'rm -f "$PUBLISH_LOG"' EXIT

if cargo publish --package bitview_website --allow-dirty 2>&1 | tee "$PUBLISH_LOG"; then
    :
elif grep -q "already exists on" "$PUBLISH_LOG"; then
    echo "bitview_website is already published; skipping"
else
    exit 1
fi

publish_release_selection "Rust workspace" --workspace

echo "Done!"
