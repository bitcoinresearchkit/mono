#!/bin/bash
set -e

rm -rf pools
curl -sLO https://github.com/mempool/mining-pool-logos/archive/refs/heads/master.zip
unzip -qo master.zip
rm master.zip
mv mining-pool-logos-master pools
rm -r ./pools/.github
find pools -type f ! -name '*.svg' -delete
