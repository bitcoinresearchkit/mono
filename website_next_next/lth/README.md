# Long Term Holders Cloud

Open `index.html` directly in a browser. Styles, chart code, and data are embedded; fonts and the chart library load from a CDN.

The cloud spans the daily minimum and maximum of 12 prices: cointime (awake) and coinflow weighted realized and capitalized prices for >4 months, LTH (>5 months), and >6 months. The cutoffs include UTXOs at least 120, 150, and 180 days old, respectively.

The page shows Bitcoin price and the cloud, with candles when zoomed in and no trend lines. Its fuchsia accent matches `colors.term.long` in `website/`.

With the updated backend running on localhost:3110, refresh from this folder:

```sh
node generate.mjs
```

Optional arguments: `--api http://localhost:3110/api` and `--start 2011-03-22`. The default start is the first day with positive values for all 12 sources; earlier zero prices cannot form this logarithmic cloud.

The generator includes today's partial data and validates all 12 inputs before atomically replacing the embedded snapshot. The generator is self-contained and uses only Node.js built-in modules.
