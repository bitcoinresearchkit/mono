#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")"

curl -fL \
  https://raw.githubusercontent.com/mempool/mining-pools/refs/heads/master/pools-v2.json \
  -o pools-v2.json
