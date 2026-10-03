# Bitcoin Middle

Open `index.html` directly. It contains the complete daily dataset, styling, and calculation code; fonts and the chart library use the same CDN module loading as the cloud pages.

The upper pane shows Bitcoin, all-holder cointime-weighted capitalized price (green), and True Market Mean (blue). The middle pane fills each series toward a 50% baseline and computes, for each day, `100 × qualifying days / elapsed days` since the selected start. A day qualifies when its Bitcoin close is greater than or equal to that day's model value. The comparison uses integer cents and includes ties. Both metrics use the same dates and denominator. Days without positive values for both models are excluded from both percentages, while available Bitcoin prices remain visible.

The default starts at the first available Bitcoin price. The start is adjustable; the endpoint is always the latest completed day in the snapshot. The generator excludes the current UTC day. Hover follows the selected day; without hovering, the figures use the last visible day. Panning and zooming do not restart the calculation. Bitcoin switches to candles when zoomed in; double-click fits the selected period.

Refresh from the repository root with the local API running:

```sh
node generate.mjs
```

Optional generator arguments: `--api http://localhost:3110/api` and `--start 2011-01-01` (optional later history cutoff). The generator is self-contained beside its HTML and validates the full input before replacing the snapshot atomically. Share only the HTML when you want a single file.

The bottom histogram compares absolute distances from 50% using the sign of `abs(TMM − 50) − abs(CP − 50)`. Every bar has value 1: green favors CP, blue favors TMM, and gray denotes a tie. The label names the winning metric for the hovered or last visible day. No bar appears before both model histories are available. It uses the same cumulative window as the middle pane.
