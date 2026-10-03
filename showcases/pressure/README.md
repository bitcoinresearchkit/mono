# Bitcoin Pressure

A standalone page in the cloud pages' style with three panes: Bitcoin price, the amount above 50%, and a signed density histogram. Open `index.html` directly; CSS, page JavaScript and daily data are embedded. Fonts and Lightweight Charts 5.2.1 load from a CDN, using the same module import as the cloud pages.

Uses the **<6m cointime-weighted ±5% supply density**. A day qualifies when total density is **at least 50%**. Its histogram value is **in-profit density minus in-loss density**, in percentage points. Non-negative values are green; negative values are red. Below-threshold dates are whitespace, not zero. A qualifying tie is a true zero. A thin orange line in the middle pane shows the amount above the threshold: total density minus 50%, in percentage points (60% density is +10 pp). It has its own scale starting at zero and stays at zero below the threshold. Displayed differences omit the unit suffix, but still represent percentage points. Density uses the cohort's own weighted supply as denominator.

Above 60% density, the middle-pane line turns yellow with a light, flat fill above a dashed +10 pp threshold.

The price switches to candles when zoomed in. All three panes share the time axis and crosshair. A compact date and price block follows the hovered day, otherwise the last visible daily observation. Double-click to restore the four-year range. The footer shows generation time in UTC.

Refresh from this folder with the backend running:

```sh
node generate.mjs
```

Optional arguments: `--api http://localhost:3110/api` and `--start 2011-01-01`. Today's partial day is included. All four histories are fetched in one request and validated before atomically replacing the embedded snapshot; missing or inconsistent inputs leave the previous page intact. The generator needs only Node.js built-in modules and works when the folder is copied elsewhere.

For sharing, send only `index.html`. The generator keeps the source series and calculations private to this folder; the HTML embeds only OHLC and final plotted indicators, with generic names. Plotted data remains inspectable. Do not publish this folder wholesale while keeping the method private: the generator, tests, and this README describe it.
