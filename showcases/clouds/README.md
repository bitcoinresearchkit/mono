# Bitcoin Clouds

Open `index.html` directly. The snapshot, CSS, and JavaScript are embedded; fonts and the chart library load from a CDN.

One chart combines the existing weighted clouds: STH (12 prices), Holders (4), and LTH (12). The draw order is LTH, Holders, STH, using soft fills with all boundary lines drawn above the fills. Each cloud name toggles its fill and both boundaries together, with a muted active/inactive status. Cloud ranges always show the hovered date, or the rightmost visible observation when not hovering. The three trend values follow the same selected date and remain visible while disabled. A discreet Trends toggle with active/inactive status (off by default) shows the STH 4m yellow sparse-dashed, 5m orange dashed, and 6m red solid lines with endpoint dots at the same 60% opacity as the cloud boundaries. Bitcoin is in front, with candles when zoomed in. Colors match `website/`.

Run `node generate.mjs` from this folder to refresh from `http://localhost:3110/api`, including today's partial data. The generator is self-contained and validates all inputs before atomically replacing the snapshot. Trend selection uses the existing expanding-bound rule and is calculated before cropping history.

Optional arguments: `--api http://localhost:3110/api` and `--start 2011-03-22`. This start is the earliest date with positive prices for all three complete clouds.

The four readouts share one row. Prices below 10,000 are shown in full; larger values use three significant digits with a decimal comma and k/m suffix. Exact dollar amounts are available on hover.
