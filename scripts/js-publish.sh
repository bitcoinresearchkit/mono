#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT_DIR="$SCRIPT_DIR/.."

VERSION="${1:-}"
if [ -z "$VERSION" ]; then
    echo "Usage: $0 <version>"
    exit 1
fi

# Both packages follow the workspace release version.
node --input-type=module - "$ROOT_DIR/modules" "$VERSION" <<'JS'
import { readFileSync, writeFileSync } from 'node:fs';
const [root, version] = process.argv.slice(2);
if (!/^\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?(?:\+[0-9A-Za-z.-]+)?$/.test(version)) {
  throw new Error(`Invalid release version: ${version}`);
}
for (const name of ['bitview-client', 'quickmatch-js']) {
  const path = `${root}/${name}/package.json`;
  const pkg = JSON.parse(readFileSync(path, 'utf8'));
  pkg.version = version;
  writeFileSync(path, JSON.stringify(pkg, null, 2) + '\n');
}
const source = `${root}/bitview-client/index.js`;
writeFileSync(source, readFileSync(source, 'utf8').replace(/VERSION = "v[^"]*"/, `VERSION = "v${version}"`));
JS

publish_if_missing() {
    local package_dir="$1" name version tag metadata
    name=$(cd "$package_dir" && node -p 'JSON.parse(require("fs").readFileSync("package.json")).name')
    version=$(cd "$package_dir" && node -p 'JSON.parse(require("fs").readFileSync("package.json")).version')
    if metadata=$(npm view "$name@$version" version --json --loglevel=silent 2>&1); then
        echo "$name@$version already published; skipping"
        return
    fi
    # Only an explicit registry E404 means unpublished. Auth/network failures
    # must stop the release rather than being mistaken for a missing version.
    if ! printf '%s' "$metadata" | node --input-type=module -e '
        import { readFileSync } from "node:fs";
        try { process.exit(JSON.parse(readFileSync(0, "utf8")).error?.code === "E404" ? 0 : 1); }
        catch { process.exit(1); }
    '; then
        printf '%s\n' "$metadata" >&2
        return 1
    fi

    case "$version" in
        *-alpha*) tag=alpha ;;
        *-beta*) tag=beta ;;
        *-rc*) tag=rc ;;
        *) tag=latest ;;
    esac

    if [ -z "${NPM_CONFIG_OTP:-}" ] && [ -t 0 ]; then
        read -r -s -p "npm OTP for $name (blank for a bypass-2FA token): " NPM_OTP
        echo ""
    fi
    echo "Publishing $name@$version ($tag)"
    if [ -n "${NPM_OTP:-}" ]; then
        (cd "$package_dir" && NPM_CONFIG_OTP="$NPM_OTP" npm publish --access public --tag "$tag")
        unset NPM_OTP
    else
        (cd "$package_dir" && npm publish --access public --tag "$tag")
    fi
}

publish_if_missing "$ROOT_DIR/modules/bitview-client"
publish_if_missing "$ROOT_DIR/modules/quickmatch-js"
