# bitview-charts

Time series charts drawn from columns: panes stacked over one time axis, each series a `Float64Array` on a shared
timeline (an index a time). Built for many series of thousands of points on a phone without its JIT: no object a
point, next to nothing made a frame, a pan moving the picture already drawn. Source modules, no build, typed by JSDoc.

```js
import { createChart, stack } from "/modules/bitview-charts/index.js";

const chart = createChart(element);              // element: any box; the chart fills it
chart.setTimes(times);                           // Float64Array of seconds, rising
const price = chart.addPane({ share: 2, scale: { mode: "log" } });
price.addLine({ values: close, color: "#f7931a", tag: true, priceLine: true });
const bands = chart.addPane({ format: (v) => `${v}%` });
bands.setRange(0, 100);                          // a fixed scale (none: fitted to what's in view)
const { tops, feet } = stack(members, { percent: true });
tops.forEach((upper, k) => bands.addBand({ lower: feet[k], upper, color: fills[k] }));
chart.show(365);                                 // the last 365 points in view
```

## Chart

- `createChart(element, { theme, axis, leftAxis, timeHeight, rightOffset, maxSpacing, dateFormat })`: `theme` is `DARK`
  (default) or `LIGHT`, or your own (`background`, `text`, `grid`, `font`, `crosshair` and `across`, its lines down
  and across, `badge`, `separator`); `axis`, whether there's an axis at the right (0, none) and its width till drawn:
  then as wide as its labels and tags need (`axisShown`: wider at once, never narrower by itself: `fitAxes()`, or a price axis' double-click, fits them again; every point kept where it is on the screen, the axis growing over the plot's edge);
  `leftAxis`, the left one's, where a pane has a left scale (as `axis`; `leftShown`); both fields too, then `layout()`; `rightOffset`, CSS pixels left empty
  past the last point (24); `dateFormat`, how the crosshair writes a point's time (seconds). `timeAxis.format`, a
  page's own time labels: `(t, kind)` (kind "year", "month", "day" or "time": what the label starts) to words, "" for
  none there, null for the library's.
- `setTimes(times)` (an end in view shifts by the change in point count, retaining its screen position on appends;
  a historical view stays put; callers replacing or rolling the timeline set their desired range explicitly),
  `addPane(options)`, `removePane(pane)`, `show(span)` (stops a flick when the view changes), `setRange(from, to, options?)` (logical indices,
  fractions allowed, within the zoom's bounds: narrower, centred at the closest; moved as a pan, by whole pixels, where
  as wide as the view; `{ exact: true }`, at its endpoints as given; stops a flick when the effective range changes),
  `range`, `onRange(listener)`,
  `onLayout(listener)` (its size, its panes' places: a page's own things kept on them; a separator's drag, at each move),
  `setCrosshair(index, y)`,
  `onCrosshair(listener)` (its point, its pane, the series a pointer's on there), `onClick(listener)` (a click or a
  tap: its point, its pane, the series hit there), `tagAt(x, y)` (the line whose tag is at a point on an axis),
  `y(series, value)` and `value(series, y)` (a value's height in its pane on its series' scale, a relative one read
  against its first value in view, and back; NaN where it has none),
  `time.firstVisible` and `time.lastVisible` (the first and last point whose centers are in view, for date captions and
  resting readings; geometry fitting includes connecting points beyond fractional edges),
  `lastIn(values, from, to)` (exported: the last index within inclusive bounds whose value isn't NaN, -1 if none: a
  reading kept from before the view, its date then said by its reader),
  `time.held(from, to)` (how far a view would move to keep its data's share in view, as the chart holds it: a page's
  glide aimed there ends where the chart stops it),
  `invalidate(whole)` (after changing a series through its methods, `invalidate()`: its pane sees its version change and
  draws it anew; `invalidate(true)`, everything drawn whole: after setting a field such as `hidden`), `destroy()`;
  `corner`, a page's own element in the corner beside the time axis, under the price axis (its buttons: it takes the
  pointer).
- Input: a drag pans (a finger's flick glides on, slowing as a list does, a little sooner; a mouse or pen's drag stops where it's
  let go), a pinch or the wheel zooms about that point (the point
  there stays there), a trackpad's sideways scroll pans (a diagonal swipe pans and zooms at once). The crosshair: where a mouse or pen is; a
  finger held still for 240 ms shows it, and moves it then; a finger pans once it's moved 6 pixels; none over a page's
  own element over the chart (a legend); its value's badge over its axis' tags (each still read beside it), its date's
  covering the time axis' labels it touches (none shows cut beside it); a dot on each line at its point, the line
  pointed at's larger, ringed in its color (`crosshair.pointed`: a page's line, null none; left undefined, the one a
  pointer's on, as its listeners are told). Separators between panes drag to share their heights (laid out and drawn at
  each move; the separator pointed at or dragged in the theme's `active` color). A drag on a price axis scales it about its middle (its range then set by hand; a
  double-click fits it again), on the time axis zooms holding the latest point where it is (in view; else the plot's
  right edge; a double-click there tells `onTimeReset`'s listeners: the page's view to come back to).

Range synchronization should flow from the chart being interacted with to its followers. Avoid reflecting a follower's
rounded range back to the source: different widths can turn that echo into a real view change and stop its flick.
Before pending data is drawn, the latest setter decides how axis fitting keeps the view: `setTimes` keeps screen
positions; a later `show` or `setRange` keeps its normalized logical endpoints, including an explicit no-op range request.
Data queued by a render callback remains pending until it is drawn.
For pointer crosshair synchronization, forward `onCrosshair` events whose `input` argument is true (pointer placements and
clears); avoid reflecting programmatic calls. A programmatic
`setCrosshair` replaces the pointer's crosshair and clears its series hit, even at the same position.

## Panes

- `addPane({ share, scale: { mode, top, bottom }, left, format })`: `share`, its part of the height; `mode`:
  `"linear"`, `"log"`, `"percent"` (each series as its change from its first value in view) or `"indexed"` (100 at it);
  `top` and `bottom`, room kept as shares of its height; `left`, its left scale's options; `format`, how its values are
  written: `format(v, step)`, a value on its axis (a tag, a level's, the crosshair's) given `step`, the step between its
  ticks around it (`stepAt(v, scale)`: a decimal past theirs, in the unit it's written in: 89.0 by ticks 20 apart, 1.053
  by 0.02; the default writer's rule), elsewhere none. `setFormat(format, ticks, side)`: a side's own (`ticks`, its
  ticks written all at once, their digits alike; an indexed scale's as its values); `setPercentFormat(format, ticks)`, a percent scale's; `frame`, a page's own element
  over its plot (a legend, a highlight), laid out with it, under the crosshair, taking no pointer (its children may).
- `addLine`, `addBand`, `addBars`, `addCandles`, `addCustom` (each with a side: `"right"`, else `"left"`: its own scale,
  its axis at the chart's left), `add(series, side)`, `remove(series)`, `order(list)` (its series drawn in that order,
  the last on top), `setRange(lo, hi, side)` (no range: fitted), `fittedRange(time, side)` (the range a real fit gives
  that side's scale, a held one left held at its range: the fit's margins, bases, other side and overlays stay as it
  left them; nothing to fit, the range as it is), `setScale(side, { mode, top, bottom })` (a side's
  scale's look, now and a left one made later), `hit(time, index, x, y)` (the series nearest a point, within 6 pixels).
- `setLegend({ onPress })`: its legend, over its plot's top left (made once, returned): `set(items, summary?)`, each
  `{ id, series, label, elements, hidden }` (the page's own `elements`, the first its key and name, the rest beside it;
  else made from its first series: a key of its color, its `label`, its value at the crosshair or its last in view, as
  its tag writes it; none in view, blank: a value from outside the view isn't shown as the view's). One shown, named; more, a chip (their first four keys and how many; a press on it told
  `onPress`); an optional `summary` caption stays beside the folded count. The series pointed at named alone in it: the page's (`crosshair.pointed`, the crosshair on the plot or
  not: a tag, a row of its own, the pointer moved onto the legend), else the one the crosshair's pointer is on. Hidden ones
  left out (none shown: each named). `setWhole(true)`: every item named (a picture's). Its look the page's stylesheet's,
  over a few rules of its own (`.bc-legend`, `.bc-legend-item`, `-key`, `-name`, `-value`, `-count`; `data-chip`,
  `data-named`, an item's `data-pointed`).

## Series

- `Line({ values, color, width, colors, palette, breaks, tag, priceLine, stroke, format, outline, dots, marker })`:
  `colors` (a `Uint8Array` of indices into `palette`) colors each segment by its first point; `breaks` (a
  `Uint8Array`, 1 at a point) stops the line after it; `tag`, its value on the axis: its last in view (the crosshair's own tag says the pointer's; written by
  `format`, else as its side writes values, given its axis's step there; a side's tags laid top to bottom, each at its
  line or just under the one over it, the last cut off at the foot, one where lines read the same, none for a line out of the pane); `priceLine`, a dotted line across at it; `stroke: false`, no line (its tag and price
  line kept); `outline`, a color it's edged in (over a heatmap, a cloud, another line: it stands off them); `dots`, a
  dot at each point that radius; `marker`, the crosshair's dot on it (its radius, 3; 0: none); `steps`, drawn as steps
  (each value held flat to the next point, then straight up or down); `curve`, bowed through its points (zoomed in);
  `zones`, `[{ value, color }]` low to high: its colors by value, each from its value up, its own under them all, changing
  exactly where it crosses one (a line colored by levels, a baseline's by its base); `dash`, its dashes; `partial`, its
  last point's period still running (a year so far): the segment to it dashed, where points have a pixel or more.
- `Band({ lower, upper, base, color, fade, below, belowFade, to, steps, zones })`: a fill between two columns (`steps`: its
  edges steps); without `lower`, an area down to `base` (`to: "top"` or `"foot"`: to the pane's edge instead); with
  `below`, a baseline (under its base in that color); `fade`, `belowFade`, the colors they fade to (gradients fixed to
  the pane: from its top to its foot, a baseline's to its base and from it); `curve`, its edges bowed as a curved line;
  `colors` and `palette` (`{ color, fade }` each), colored point by point, each segment in its first point's (zoomed in,
  where its edges cross on the way to a point in another color, changing color at the crossing: a cloud between two
  lines, its edges the two, colored by which one is over); `zones` (`{ value, color, fade }` low to high), an area
  filled by value, parted exactly where its line crosses a zone's value, each part fading as the whole would.
- `Bars({ values, base, color, colors, palette, radius, partial })` (`partial`: its last bar at `PARTIAL` of its opacity), `Candles({ open, high, low, close, up, down, upWick,
  downWick, upBorder, downBorder })` (a part's own color, else its body's: a hollow candle, its body the ground's):
  each with an `outline` (a color: each bar, each candle's body and wick, edged in it) and a `radius` (its corners', a
  number or one by its points' spacing). Under `BODIES_FROM` pixels a point (3), candles are drawn as their wicks
  alone (a page may show their close as a line there instead).
- `Bubbles({ values, sizes, up, upEdge, down, downEdge, size, hide, scaleTo, halo })`: a dot a point where `values`
  put it, its area its size's share of the largest (of those in view, kept up as the view moves, or `scaleTo: "all"`),
  `size` pixels across at the largest (of every size, placed or not), those under `hide` of it left out; above zero in
  `up`, below in `down`, each edged; the largest drawn first; `halo`, ringed in it first. `set(i, value, size)`, one
  point's in place (a live one).
- `Shade({ values, color, colors, palette })`: stretches of the pane's whole height behind what's drawn (a signal's
  where it holds), each from half a point's room before its first point to half after its last (a pixel at least: a
  point alone zoomed out, a sliver), its color changing halfway between two points, a NaN value none; never fitted or hit.
- `Grid({ columns, column, edge, paint, look, width, viewed, fit })`: a heatmap's cells, its columns the page's
  (`columns`, each point's: a day, say; -1 none), zoomed out a block of a power of two of them (`width` CSS pixels at
  least, a block drawn as its last: the same blocks at any pan); a block's cells `column(owner, step)` (its rows,
  rising, their values and a key; null: none or not yet, then `changed()`), a row's floor `edge(row)` (rows even on a
  log scale). Each block a strip made once (a pixel a row on a log scale, a step of values on a linear one, each pixel
  its strongest cell, colored by the page's `paint(value, pixels, at)`, its look's key `look`: `setLook`), stretched
  onto device pixels, smoothed only where it's squeezed; `forget(key)`, a column's strips let go (its cells anew).
  `viewed({ step, owners })`, told the blocks in view before each draw (what the page fetches and keeps). Fitted by
  the page's `fit`, else not; never hit.
- `Profile({ groups, edge, cell, at, lit, bin, backdrop, closeLine, cumulative, viewed, fit })`: groups' amounts by value
  rows (each `{ id, rows, values, gain, loss, dimGain, dimLoss }`; a row's floor `edge(row)`, a value's row
  `cell(value)`) as bars from the plot's right edge, stacked, as long as a bin's total against the longest in view
  (`columnWidth(plot)` pixels at most: a fifth of the plot from 72 to 200, exported for a page keeping room for it); rows gathered into bins `bin` pixels tall at least, on whole multiples of a row (the
  same at any pan), a pixel of `backdrop` between two; split at `at` (a close), a group's `gain` under it and `loss` over
  it, a group `lit`, the others dim; `closeLine`, a close's dashed line across it (an older day's). Held at the plot's
  edge; an unchanged view keeps its picture and bins. The page is told before each render (`viewed`); `set(...)` its groups and the rest; `covers(x)`, whether a point's in its column as
  drawn (bars or cumulative, its left edge as painted; nothing drawn, or hidden, none); `readAt(y, group?)`, what its
  bars hold at a height: `{ from, to, sum }`, the bin's rows (`to` not in it) and its signed sum, its groups' or
  `group`'s (an empty bin's, 0), null where no bin is, hidden or cumulative; its bins found by the device pixel, as painted; `hit(x, y)`, the bar under a point, its ends as
  painted (its group, its bin's rows; none out of its column, or cumulative). `cumulative`: in place of the bars, the amounts summed from the close outward (upward over
  it, downward under it), the groups stacked, each filled in its colors to the edge, as long as the longer side's
  whole; `sumsAt(y)`, each group's sum from the close to a height. Fitted by the page's `fit`, else not.
- `Marks({ series, marks, room })`: a series' marks (a signal's turns), added to its pane on its series' side after it:
  each `{ at, position, shape, color, text, textColor, size }` at a point (its index), `"above"` it (a candle's high),
  `"below"` it (its low) or `"on"` it; an arrow (`"up"`, `"down"`), a `"circle"` or a `"square"` as tall as a point is
  wide (12 to 30 pixels, times `size`; 0: none), several at a point stacked away from it; its words in a pill of the
  ground edged in its color (or `textColor`), only where they have room at the zoom (from the latest back, one running
  into the next one's goes without them). Their room kept at the scale's ends while its series has values in view
  (`room: false`, none); `set(marks)` after a change.
- `channels(color)`: a CSS color's sRGB channels and opacity (0 to 255), read from the canvas's own writing of it (any
  CSS color, OKLCH and wide-gamut ones too), never a pixel: a browser resisting fingerprinting (Mullvad) gives read
  pixels noise.
- A series' `viewed(time)`: brought up to the view before each draw, its `version` changed where what it draws hangs on
  the view (bubbles' largest): its pane drawn whole then, its picture moved otherwise.
- `Levels({ lines, side, format })`: lines across a pane at values (thresholds, a baseline's base), behind every
  series, each `{ value, color, width, dash, tag, hidden, faint }`, its value tagged on its side's axis where it asks
  (written by `format`, else as a scale writes values: by its axis's ticks); `set({ lines })` or, a line changed in
  place, `changed()`; then `chart.invalidate(true)`.
- Any series' `behind`: drawn behind every series, whatever their order (a backdrop: levels).
- Any series' `faint`: drawn at a fifth of its opacity (`FAINT`, for a page's own layers to match), its tag and
  price line gone (another lit over it); then `chart.invalidate(true)`.
- Any series' `overlay`: a `Scale` of its own (unseen, the pane's height, linear or log: a volume along a price pane's
  foot, a page's drawing on a range of its own). Each shared overlay is fitted once per pass to its visible series'
  combined extent and margins, by the same rules as a side scale; no extent, its range stays as set. An overlay that
  references a pane's side scale follows that side's auto/manual fit.
  An independent relative overlay is rejected; use a side scale for relative series.
- Any series' `fit`: a function `(a, b, positive, out)` taking its place in fitting its scale (as `extent`: its range
  from point `a` to `b` widening `out`'s low and high), its own `extent` there to call (a range from zero, a fixed one).
- `stack(members, { percent })`: a stack's columns from its members' (the first at the foot): `tops` (each member's value
  plus those under it, NaN where it has none), `feet` (the sum under it); with `percent`, shares of each point's whole
  (a negative member counted as none), `scale` the factor taken (100 over the whole).
- Lines, bars and candles: `setValues(...)`, `set(i, ...)` (a live point), `setStyle(...)`; bands: `set({ lower, upper,
  color, below })`; then `chart.invalidate()`.
- `Custom({ draw, extent, first, distance, axisTags, timeTags, margins, viewed }, { fixed, over, behind })`: a page's own
  (bubbles, a heatmap, drawings, marks), drawn by its hooks as part of its pane's picture (see custom.js); its own tags
  on a price axis or the time axis (a drawing's value, its date), a price axis' over its lines'; `viewed`, told before each draw of its pane (what it
  draws found again from the view, once); `fixed`, held in place as the view moves (its pane drawn whole on a
  view or content change, kept while unchanged); `over`, over everything, the crosshair too, on a layer of its own drawn whole each frame (labels and handles
  placed in the plot); `behind`, behind every series; `changed()` after a change.
- `picture(chart)`: the chart as one canvas (an Export, a card).
- `compact(v, step)`: a value as a scale writes it unless told otherwise (four significant digits at most, thousands as
  K, M, B, T; given `step`, its ticks' around it, a decimal past theirs).
- Hits (`Pane.hit`, the crosshair's and clicks' series) are measured against each series' own drawing near the pointer
  (recorder.js: a canvas stand-in keeping its strokes, fills and dots), so they match the picture at any zoom.
  Strokes and fills honor rectangular clips and a library gradient's painted rows; transparent parts are not hit.

## How it draws

- A pan moves each pane's picture by whole pixels and draws only the strip come into view; drawn whole at the first
  frame nothing moves.
- A zoom draws whole, or, where a pane takes more than 8 ms to draw whole (a CPU-drawn canvas at a large size),
  stretches its last sharp picture till it's still (not where it has a dashed line: its dashes would stretch).
- A dashed line's dashes are fixed to it: each subpath its own stroke, its dashes as far along as its length from the
  line's dash origin (measured from the nearer end of its last draw), a long one split in a gap every 128 pixels or so
  (the canvas measures a subpath in single precision); so a pan's strip draws the dashes its moved picture has. A
  curved one's draws whole as the view moves, as does one a whole draw found unsplit over 2,048 pixels (its segments
  each about a multiple of its dashes' period: no gap to split in).
- Zoomed out, a line's pixel column is drawn as its low and high in the order they came, read from its summary by blocks
  of 4, 8, 16… points (a draw's work by its columns, not its points); a band's, as its last point (a gap found by an
  index, not a scan); bars' and candles', from their points.
- Lines' ys on device pixels, a run at one y one segment; bevelled joins (half a round join's cost in Firefox).

Its bench and its types: `showcases/_engine/` (`index.html`, `types.html`).
