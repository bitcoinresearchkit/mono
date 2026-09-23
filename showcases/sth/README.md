# Short Term Holders Cloud

Open `index.html` directly in a browser. The data, page styles, and chart code are embedded; the chart library and fonts load from a CDN, so an internet connection is required. No local server is needed.

The cloud spans the daily minimum and maximum of 12 prices: cointime (awake) and coinflow weighted realized and capitalized prices for <4 months, STH (<5 months), and <6 months. The three URPD trend bounds remain separate from the cloud.

## Refresh

With the local backend running on localhost:3110, run from this folder:

```sh
node generate.mjs
```

The generator includes today's in-progress data and updates the embedded snapshot in `index.html` atomically after validating all inputs. Run it again to refresh the partial day. The generator is self-contained and uses only Node.js built-in modules.

The updated backend must expose all 12 weighted price histories and the Bedrock <4M, <5M, and <6M cost-basis min/max histories. Each pair contributes one trend line: a max increasing from the previous day selects min as the floor, and a min decreasing selects max as the ceiling. Flat bounds and contractions (a rising min or falling max as older coins age out) retain the selected side. Selection uses the bounds themselves, not candle overlap. If both extremes expand on the same day, retain the prior selection because their ordering is unknown. Before the first unambiguous expansion, or after missing observations, the line stays empty until a side is selected again. Zero-price buckets remain valid in the snapshot but are omitted from the logarithmic chart.

Selection starts at the beginning of the API history before cropping to `--start`. Cloud inputs remain separate. The snapshot stores only selected bounds; older snapshots without bounds remain readable until regenerated. Missing latest bounds or malformed inputs abort the refresh without replacing the existing snapshot.

Optional arguments: `--api http://localhost:3110/api` and `--start 2011-01-01`.
