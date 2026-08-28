#!/bin/bash
set -eo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT_DIR="$SCRIPT_DIR/.."

cd "$ROOT_DIR"

wait_for_crates_io_rate_limit() {
    local publish_log="$1"

    if ! grep -q "429 Too Many Requests" "$publish_log"; then
        return 1
    fi

    local retry_at
    retry_at=$(
        grep -Eo 'Please try again after [[:alpha:]]{3}, [0-9]{2} [[:alpha:]]{3} [0-9]{4} [0-9]{2}:[0-9]{2}:[0-9]{2} GMT' "$publish_log" |
            tail -1 |
            sed 's/^Please try again after //' || true
    )

    if [ -z "$retry_at" ]; then
        echo "crates.io rate limit response did not include a retry time" >&2
        return 1
    fi

    local retry_epoch
    if retry_epoch=$(LC_ALL=C date -j -f '%a, %d %b %Y %H:%M:%S %Z' "$retry_at" '+%s' 2>/dev/null); then
        :
    elif retry_epoch=$(LC_ALL=C date -d "$retry_at" '+%s' 2>/dev/null); then
        :
    else
        echo "Could not parse crates.io retry time: $retry_at" >&2
        return 1
    fi

    local wait_seconds
    wait_seconds=$((retry_epoch - $(date '+%s') + 5))
    if [ "$wait_seconds" -lt 1 ]; then
        wait_seconds=1
    fi

    echo "crates.io rate limit reached; retrying after $retry_at (${wait_seconds}s)"
    sleep "$wait_seconds"
}

publish_release_selection() {
    local label="$1"
    shift

    local publish_log
    while true; do
        publish_log=$(mktemp -t brk-rust-publish)

        if CARGO_TERM_COLOR=always cargo release publish "$@" --execute --no-confirm 2>&1 | tee "$publish_log"; then
            rm -f "$publish_log"
            return 0
        elif ! grep -q "uncommitted changes detected" "$publish_log" &&
            grep -q "disabled due to previous publish" "$publish_log" &&
            grep -q "no packages selected" "$publish_log"; then
            echo "$label is already published; skipping"
            rm -f "$publish_log"
            return 0
        elif wait_for_crates_io_rate_limit "$publish_log"; then
            rm -f "$publish_log"
        else
            rm -f "$publish_log"
            return 1
        fi
    done
}

# bitview_website embeds ignored assets through symlinks, so Cargo requires
# --allow-dirty even when the Git worktree itself is clean. Publish its
# workspace dependencies first so Cargo can resolve the packaged crate.
publish_release_selection "brk_logger" --package brk_logger
publish_release_selection "importmap" --package importmap

PUBLISH_LOG=$(mktemp -t brk-rust-publish)
trap 'rm -f "$PUBLISH_LOG"' EXIT

while true; do
    if CARGO_TERM_COLOR=always cargo publish --package bitview_website --allow-dirty 2>&1 | tee "$PUBLISH_LOG"; then
        break
    elif grep -q "already exists on" "$PUBLISH_LOG"; then
        echo "bitview_website is already published; skipping"
        break
    elif wait_for_crates_io_rate_limit "$PUBLISH_LOG"; then
        :
    else
        exit 1
    fi
done

publish_release_selection "Rust workspace" --workspace

echo "Done!"
