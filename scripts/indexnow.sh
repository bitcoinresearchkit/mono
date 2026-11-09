#!/bin/bash
# Tells search engines which of bitview.space's pages changed between two commits (IndexNow), so
# only those get crawled again. A page counts when the sitemaps list it and its index.html changed,
# or when it joined or left the sitemaps. Everything else is left to the sitemaps.
# usage: scripts/indexnow.sh [--dry-run] <old-commit> <new-commit>
set -euo pipefail

HOST=bitview.space
ENDPOINT=https://api.indexnow.org/indexnow
MAX_URLS_PER_POST=10000
ROOT="$(cd "$(dirname "$0")/.." && pwd)"

DRY_RUN=false
if [ "${1:-}" = "--dry-run" ]; then
    DRY_RUN=true
    shift
fi
if [ $# -ne 2 ]; then
    echo "Usage: $0 [--dry-run] <old-commit> <new-commit>" >&2
    exit 2
fi
OLD=$1
NEW=$2

git -C "$ROOT" cat-file -e "$NEW^{commit}" 2>/dev/null || git -C "$ROOT" fetch -q origin prod

WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

listed() {
    for sitemap in showcases/sitemap.xml showcases/studio/sitemap.xml; do
        git -C "$ROOT" show "$1:$sitemap" 2>/dev/null || true
    done | sed -n 's|.*<loc>\(.*\)</loc>.*|\1|p' | sed 's/&amp;/\&/g' | LC_ALL=C sort -u
}
listed "$OLD" > "$WORK/old"
listed "$NEW" > "$WORK/new"

git -C "$ROOT" diff --name-only "$OLD" "$NEW" -- showcases/index.html 'showcases/*/index.html' |
    sed -e 's|^showcases/index\.html$||' -e 's|^showcases/\(.*\)/index\.html$|\1/|' -e "s|^|https://$HOST/|" |
    LC_ALL=C sort -u > "$WORK/edited"

{
    LC_ALL=C comm -12 "$WORK/edited" "$WORK/new"
    LC_ALL=C comm -3 "$WORK/old" "$WORK/new" | tr -d '\t'
} | LC_ALL=C sort -u > "$WORK/urls"

if [ "$DRY_RUN" = true ]; then
    cat "$WORK/urls"
    exit 0
fi
[ -s "$WORK/urls" ] || exit 0

KEY_FILE="$(git -C "$ROOT" ls-tree --name-only "$NEW" showcases/ | grep -E '^showcases/[0-9a-f]{32}\.txt$' || true)"
if [ -z "$KEY_FILE" ]; then
    echo "$NEW has no IndexNow key; nothing sent" >&2
    exit 0
fi
KEY="$(basename "$KEY_FILE" .txt)"

split -l "$MAX_URLS_PER_POST" "$WORK/urls" "$WORK/batch."
for batch in "$WORK"/batch.*; do
    status="$(jq -R -n --arg host "$HOST" --arg key "$KEY" \
        '{host: $host, key: $key, keyLocation: "https://\($host)/\($key).txt", urlList: [inputs]}' < "$batch" |
        curl -sS -m 30 -o /dev/null -w '%{http_code}' -H 'Content-Type: application/json; charset=utf-8' \
            --data-binary @- "$ENDPOINT")"
    case "$status" in
        200 | 202) ;;
        *)
            echo "IndexNow answered HTTP $status" >&2
            exit 1
            ;;
    esac
done
echo "IndexNow: $(wc -l < "$WORK/urls" | tr -d ' ') URLs"
