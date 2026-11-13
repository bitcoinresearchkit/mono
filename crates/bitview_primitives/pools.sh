#!/usr/bin/env bash
# Refreshes the mining pool list and the showcases' pool logos from mempool.
set -euo pipefail

cd "$(dirname "$0")"

curl -fL \
  https://raw.githubusercontent.com/mempool/mining-pools/refs/heads/master/pools-v2.json \
  -o pools-v2.json

# Logos are named like the showcases' pool slug: the name's lowercase letters and digits. The pages draw only the logos
# assets/pools/logos.js lists, written here from the folder (a logo added by hand too: run again).
python3 - <<'PY'
import json, os, re, textwrap, urllib.request

ASSETS = "../../showcases/assets/pools"
BASE = "https://mempool.space/resources/mining-pools"

def fetch(name):
    try:
        with urllib.request.urlopen(f"{BASE}/{name}", timeout=20) as response:
            if response.headers.get("Content-Type", "").startswith("image/svg"):
                return response.read()
    except Exception:
        pass
    return None

default = fetch("default.svg")
saved = 0
for pool in json.load(open("pools-v2.json")):
    slug = re.sub(r"[^a-z0-9]", "", pool["name"].lower())
    logo = fetch(f"{slug}.svg")
    if logo and logo != default:
        open(f"{ASSETS}/{slug}.svg", "wb").write(logo)
        saved += 1
print(f"{saved} pool logos")

# Each list a line of slugs filled to the width before the next.
files = sorted(name for name in os.listdir(ASSETS) if name.endswith(".svg"))
lists = {
    "LOGOS": [name[: -len(".svg")] for name in files if not name.endswith(".light.svg")],
    "LIGHT": [name[: -len(".light.svg")] for name in files if name.endswith(".light.svg")],
}
path = f"{ASSETS}/logos.js"
js = open(path).read()
for name, slugs in lists.items():
    items = textwrap.fill(" ".join(f'"{slug}",' for slug in slugs), 117, initial_indent="  ", subsequent_indent="  ")
    js, found = re.subn(rf"const {name} = new Set\(\[.*?\]\);", lambda _: f"const {name} = new Set([\n{items}\n]);", js, count=1, flags=re.S)
    assert found, f"{path}: no {name} list to write"
open(path, "w").write(js)
print(f"{path}: {len(files)} logo files")
PY
