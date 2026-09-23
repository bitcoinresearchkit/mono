# Holders Cloud

Open `index.html` directly in a browser. Styles, chart code, and the data snapshot are embedded. Fonts and the chart library load from a CDN.

The cloud is the daily minimum and maximum of these four all-holder prices:

- Cointime-weighted realized price (`awake_price_cents`)
- Cointime-weighted capitalized price (`awake_capitalized_price_cents`)
- Coinflow-weighted realized price (`coinflow_price_cents`)
- Coinflow-weighted capitalized price (`coinflow_capitalized_price_cents`)

The page shows only Bitcoin price and the cloud. Price switches to candles when zoomed in. No trend lines or methodology details appear on the page.

The folder contains the page (`index.html`) and its generator (`generate.mjs`). With the updated backend running on localhost:3110, refresh from this folder:

```sh
node generate.mjs
```

Optional arguments: `--api http://localhost:3110/api` and `--start 2011-01-01`.

The generator includes today's partial data and validates all four inputs before atomically replacing the embedded snapshot. The generator is self-contained and uses only Node.js built-in modules.
