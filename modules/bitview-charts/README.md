# bitview-charts

Time series charts drawn from columns: panes stacked over one time axis, each series a `Float64Array` on a shared
timeline (an index a time). Built for many series of thousands of points on a phone without its JIT: no object a
point, nothing made a frame, a pan moving the picture already drawn. Source modules, no build, typed by JSDoc.

```js
import { createChart, stack } from "/modules/bitview-charts/index.js";

const chart = createChart(element);              // element: any box; the chart fills it
chart.setTimes(times);                           // Float64Array of seconds, rising
const price = chart.addPane({ share: 2, scale: { mode: "log" } });
price.addLine({ values: close, color: "#f7931a", tag: true, priceLine: true });
const bands = chart.addPane({ format: (v) => `${v}%` });
bands.setRange(0, 100);                          // a fixed scale (none: fitted to what's in view)
const edges = stack(members, { percent: true });
edges.forEach((upper, k) => bands.addBand({ lower: k ? edges[k - 1] : undefined, upper, color: fills[k] }));
chart.show(365);                                 // the last 365 points in view
```

## Chart

- `createChart(element, { theme, axis, leftAxis, timeHeight, rightOffset, maxSpacing, dateFormat })`: `theme` is `DARK`
  (default) or `LIGHT`, or your own (`background`, `text`, `grid`, `font`, `crosshair` and `across`, its lines down
  and across, `badge`, `separator`); `axis`, the axis' width at the right (64; 0, none); `leftAxis`, the left one's,
  where a pane has a left scale (as `axis`); both fields too, then `layout()`; `rightOffset`, CSS pixels left empty
  past the last point (24); `dateFormat`, how the crosshair writes a point's time (seconds). `timeAxis.format`, a
  page's own time labels: `(t, kind)` (kind "year", "month", "day" or "time": what the label starts) to words, "" for
  none there, null for the library's.
- `setTimes(times)`, `addPane(options)`, `removePane(pane)`, `show(span)`, `setRange(from, to)` (logical indices,
  fractions allowed), `range`, `onRange(listener)`, `onLayout(listener)` (its size, its panes' places: a page's own
  things kept on them; as a separator's dragged, its two panes shown stretched, each pane's `shownTop` and
  `shownHeight` where it shows), `setCrosshair(index, y)`, `onCrosshair(listener)`, `onClick(listener)` (a click or a
  tap: its point, its pane, the series hit there), `tagAt(x, y)` (the line whose tag is at a point on an axis),
  `invalidate(whole)` (after changing a series through its methods, `invalidate()`: its pane sees its version change and
  draws it anew; `invalidate(true)`, everything drawn whole: after setting a field such as `hidden`), `destroy()`.
- Input: a drag pans (a flick glides on, as iOS's lists do), a pinch or the wheel zooms about that point (the newest
  point kept where it is while it's in view), a trackpad's sideways scroll pans. The crosshair: where a mouse is; a
  finger held still for 350 ms shows it, and moves it then; a finger pans once it's moved 6 pixels; none over a page's
  own element over the chart (a legend). Separators between panes drag to share their heights (the two panes shown
  stretched, laid out once let go). A drag on a price axis scales it about its middle (its range then set by hand; a
  double-click fits it again), on the time axis zooms about the plot's right edge.

## Panes

- `addPane({ share, scale: { mode, top, bottom }, left, format })`: `share`, its part of the height; `mode`:
  `"linear"`, `"log"`, `"percent"` (each series as its change from its first value in view) or `"indexed"` (100 at it);
  `top` and `bottom`, room kept as shares of its height; `left`, its left scale's options; `format`, how its values are
  written. `setFormat(format, ticks, side)`: a side's own (`ticks`, its ticks written all at once, their digits alike;
  an indexed scale's as its values); `setPercentFormat(format, ticks)`, a percent scale's; `setTagsTop(top, side)`, a
  side's tags kept under that height (a page's legend over the pane).
- `addLine`, `addBand`, `addBars`, `addCandles`, `addCustom` (each with a side: `"right"`, else `"left"`: its own scale,
  its axis at the chart's left), `add(series, side)`, `remove(series)`, `order(list)` (its series drawn in that order,
  the last on top), `setRange(lo, hi, side)` (no range: fitted),
  `hit(time, index, x, y)` (the series nearest a point, within 6 pixels).

## Series

- `Line({ values, color, width, colors, palette, breaks, tag, priceLine, stroke, format, outline, dots, marker })`:
  `colors` (a `Uint8Array` of indices into `palette`) colors each segment by its first point; `breaks` (a
  `Uint8Array`, 1 at a point) stops the line after it; `tag`, its last value in view on the axis (written by `format`,
  else as its side writes values); `priceLine`, a dotted line across at it; `stroke: false`, no line (its tag and price
  line kept); `outline`, a color it's edged in (over a heatmap, a cloud, another line: it stands off them); `dots`, a
  dot at each point that radius; `marker`, the crosshair's dot on it (its radius, 4; 0: none); `steps`, drawn as steps
  (each value held flat to the next point, then straight up or down); `curve`, bowed through its points (zoomed in);
  `under`, `{ value, color }`: its stroke in that color below that value (a baseline's line); `dash`, its dashes.
- `Band({ lower, upper, base, color, fade, below, belowFade, to, steps })`: a fill between two columns (`steps`: its
  edges steps); without `lower`, an area down to `base` (`to: "top"` or `"foot"`: to the pane's edge instead); with
  `below`, a baseline (under its base in that color); `fade`, `belowFade`, the colors they fade to (gradients fixed to
  the pane: from its top to its foot, a baseline's to its base and from it).
- `Bars({ values, base, color, colors, palette, radius })`, `Candles({ open, high, low, close, up, down, upWick,
  downWick, upBorder, downBorder })` (a part's own color, else its body's: a hollow candle, its body the ground's):
  each with an `outline` (a color: each bar, each candle's body and wick, edged in it) and a `radius` (its corners', a
  number or one by its points' spacing).
- Any series' `fit`: a function `(a, b, positive, out)` taking its place in fitting its scale (as `extent`: its range
  from point `a` to `b` widening `out`'s low and high), its own `extent` there to call (a range from zero, a fixed one).
- `stack(members, { percent })`: a stack's edges from its members' columns.
- Lines, bars and candles: `setValues(...)`, `set(i, ...)` (a live point), `setStyle(...)`; bands: `set({ lower, upper,
  color, below })`; then `chart.invalidate()`.
- `Custom({ draw, extent, first, distance, axisTags, timeTags }, { fixed, over })`: a page's own (bubbles, a heatmap,
  drawings, marks), drawn by its hooks as part of its pane's picture (see custom.js); its own tags on a price axis or
  the time axis (a drawing's value, its date); `fixed`, held in place as the view moves (its pane drawn whole each
  frame); `over`, over everything, the crosshair too, on a layer of its own drawn whole each frame (labels and handles
  placed in the plot); `changed()` after a change.
- `picture(chart)`: the chart as one canvas (an Export, a card).

## How it draws

- A pan moves each pane's picture by whole pixels and draws only the strip come into view; drawn whole at the first
  frame nothing moves.
- A zoom draws whole, or, where a pane takes more than 8 ms to draw whole (a CPU-drawn canvas at a large size),
  stretches its last sharp picture till it's still.
- Zoomed out, a line's pixel column is drawn as its first, low, high and last values, read from its summary by blocks
  of 4, 8, 16… points (a draw's work by its columns, not its points); a band's, as its last point (a gap found by an
  index, not a scan); bars' and candles', from their points.
- Lines' ys on device pixels, a run at one y one segment; bevelled joins (half a round join's cost in Firefox).

Its bench and its types: `showcases/_engine/` (`index.html`, `types.html`).
