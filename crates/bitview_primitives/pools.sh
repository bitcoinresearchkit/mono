#!/usr/bin/env bash
# Refreshes the mining pool list and the website's pool logos from mempool.
set -euo pipefail

cd "$(dirname "$0")"

curl -fL \
  https://raw.githubusercontent.com/mempool/mining-pools/refs/heads/master/pools-v2.json \
  -o pools-v2.json

# Logos are named like the website's pool slug: the name's lowercase letters and digits.
python3 - <<'PY'
import json, re, urllib.request

ASSETS = "../../website/assets/pools"
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
PY
