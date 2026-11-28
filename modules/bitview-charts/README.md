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

- `createChart(element, { theme, axis, leftAxis, timeHeight, maxSpacing, dateFormat, embedded })`: `theme` is `DARK`
  (default) or `LIGHT`, or your own (`background`, `text`, `grid`, `font`, `crosshair` and `across`, its lines down
  and across, `badge`, `separator`; its `font`, every word on the chart in one face and one size: axes, tags, marks,
  the crosshair's, its legends', as `--bc-font` on `element`; `DARK` and `LIGHT`: `12px system-ui, sans-serif`);
  `axis`, whether there's an axis at the right (0, none) and its width till drawn:
  then as wide as its labels and tags need (`axisShown`: wider at once, never narrower by itself: `fitAxes()`, or a price axis' double-click, fits them again; every point kept where it is on the screen, the axis growing over the plot's edge);
  `leftAxis`, the left one's, where a pane has a left scale (as `axis`; `leftShown`); both fields too, then `layout()`; `dateFormat`, how the crosshair writes a point's time (seconds). `timeAxis.format`, a
  page's own time labels: `(t, kind)` (kind "year", "month", "day" or "time": what the label starts) to words, "" for
  none there, null for the library's. The canvases it lays in `element` are `aria-hidden`: the chart's name and
  words are the page's (a heading, its legends).
  `embedded`, a chart in a page that scrolls: a finger's mostly vertical swipe scrolls the page (the browser deciding:
  `touch-action: pan-y`), a sideways one pans, a pinch zooms, a held finger's crosshair holds the page still; a finger
  takes no separator or price axis there (their drags are vertical; a mouse's still do). A pinch starts with both
  fingers: a second one joining a scroll already running is the browser's. Else (a chart filling its screen) it takes
  every gesture.
- `setTimes(times)` (an end in view shifts by the change in point count, retaining its screen position on appends;
  a historical view stays put; callers replacing or rolling the timeline set their desired range explicitly),
  `addPane(options)`, `removePane(pane)`, `show(span)` (stops a flick when the view changes), `setRange(from, to, options?)` (logical indices,
  fractions allowed, within the zoom's bounds: narrower, centred at the closest; moved as a pan, by whole pixels, where
  as wide as the view; `{ exact: true }`, at its endpoints as given; stops a flick when the effective range changes),
  `onRange(listener)`,
  `onLayout(listener)` (its size, its panes' places: a page's own things kept on them; a separator's drag, at each move),
  `setCrosshair(index, y)` (its date written by `crosshair.setFormat`: written as nothing, no badge, as a value's),
  `onCrosshair(listener)` (its point, its pane, the series a pointer's on there), `onClick(listener)` (a click or a
  tap: its point, its pane, the series hit there; not a tap that caught a glide, which only stops it), `tagAt(x, y)` (the line whose tag is at a point on an axis),
  `y(series, value)` and `value(series, y)` (a value's height in its pane on its series' scale, a relative one read
  against its first value in view, and back; NaN where it has none),
  `time.rightOffset`, the blank after the last point in CSS pixels: 6% of the plot, 16 to 48 (`rightBlank(width)`, one rule for
  every chart), plus `time.reserve` where a page draws a column there (a profile: `columnWidth`);
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
  there stays there; a pinch let go with both fingers at once zooms on, slowing as a flick does), a trackpad's sideways scroll pans (a diagonal swipe pans and zooms at once). The crosshair: where a mouse or pen is; a
  finger held still for 240 ms shows it, and moves it then; a finger pans once it's moved 6 pixels; none over a page's
  own element over the chart (a legend); its value's badge over its axis' tags (each still read beside it), its date's
  covering the time axis' labels it touches (none shows cut beside it); a dot on each line at its point, the line
  pointed at's larger, ringed in its color (`crosshair.pointed`: a page's line, null none; left undefined, the one a
  pointer's on, as its listeners are told). Lines first, dots over them: its lines, a page's own that go with it (a
  `Custom`'s `crosshair`), then the dots its line down crosses (a mark's, a line's own, drawn again over it) and its
  dots on the lines; a page's layers over the panes (`over`, words) over it all. Separators between panes drag to share their heights (laid out and drawn at
  each move; the separator pointed at or dragged in the theme's `active` color). A drag on a price axis scales it about its middle (its range then set by hand; a
  double-click fits it again), on the time axis zooms holding the latest point where it is (in view; else the plot's
  right edge; a double-click there tells `onTimeReset`'s listeners: the page's view to come back to).
- `chart.controller`: how the view moves: the wheel, a flick, an eased `viewTo({ from, to }, duration)` (cubic, out; at
  once with no duration). `range` is the view, `target` where an eased move goes (null at rest), `halt()` ends a flick
  or a pinch's glide, `stop()` every motion. In an `embedded` chart a mostly vertical wheel scrolls the page, a pinch zooms, a sideways
  scroll pans. `time.first`, the first point with data (a page's series starting after its times do): the view's held
  on its data from there, as from a first point. `time.shown(span)` is the range `show(span)` sets (an eased move aimed
  there). A flick's glide and an eased move resolve at once under reduced motion. Its physics and clocks are
  `/modules/motion`'s (`glide`, `velocityTracker`, `reduced`; `motionClock()` and `motionTime()`).
- Presses: `press(pointerdown, { end, move?, captor?, signal? })` (exported) follows a press to its end however it
  ends, so the chart's presses and a page's own (a slider's drag, a long press, a list's reorder) end alike: `end` is
  told once, with the pointerup at a release, else with null at its pointercancel, its capture lost by `captor`, the
  window left, a mouse or pen moving with no button down, or a touch the browser forgot (below; then `lost` true);
  `move` hears its moves. Its events are heard on the window in the capture phase, before any element's listeners, so
  only a listener before it on the window itself could stop them. It returns its stop (`end` untold).
- Navigator (`navigator.js`): `new Navigator(strip, chart, hooks)` gives a strip of the whole history (the page paints
  it) its drags: the window dragged pans the view, an edge (within `grip` CSS pixels) moves that end (10 points at
  least), a press off it glides the view there (centred, held on the data) and a drag goes on from where the glide
  has got to; a change of the page's `data()` or of the strip's width mid-drag takes it up again from the view. A
  finger's mostly vertical swipe on the strip scrolls the page (`touch-action: pan-y`, set on it): a finger's press is
  taken once it moves more sideways than up or down, or lifts, so a scroll begun there neither jumps nor stops the view.
  Hooks: `overview()` (the points across the strip as painted now; null, nothing to navigate), `length()` (across it
  at rest: a drag's scale), `data()`, `within(range, keep)` (a view as long as the page lets one be), `glide(range)`,
  `window` (the element `dragging` while held), `least` (the window's least width, which `windowOn(left, right,
  width, least)` applies to what the page paints), `grip`, `pressed?()`, `dropped?()`, `signal?`. `dragging` says
  whether it's held.
- Forgotten touches: iOS WebKit can hand a touch to a native gesture and send it neither pointerup nor pointercancel
  (WebKit bug 239014). A press on such a touch ends (with null) once it's proved gone, and a press the chart took
  meanwhile as a pinch's second finger is pressed again alone. Only the browser's proof counts: a
  touchend or touchcancel with no finger left on the screen (every touch still down is gone), a touchstart whose fingers
  are all those it starts (every other is), or a press taking the id of one still down (that one, before the press is
  handled). A touch dropped while another finger stays down is held till then: until that finger lifts, a drag pinches
  about it. A press arriving while a lost touch still holds a drag (a separator's, a page's own) is spent freeing that
  drag; the next press works. This relies on a press's pointerdown coming before its touchstart and a lift's pointerup before its touchend
  (measured in Chromium, WebKit and iOS 27.2's WebKit).

Range synchronization should flow from the chart being interacted with to its followers. Avoid reflecting a follower's
rounded range back to the source: different widths can turn that echo into a real view change and stop its flick.
Before pending data is drawn, the latest setter decides how axis fitting keeps the view: `setTimes` keeps screen
positions; a later `show` or `setRange` keeps its normalized logical endpoints, including an explicit no-op range request.
Data queued by a render callback remains pending until it is drawn.
For pointer crosshair synchronization, forward `onCrosshair` events whose `input` argument is true (pointer placements and
clears); avoid reflecting programmatic calls. A programmatic
`setCrosshair` replaces the pointer's crosshair and clears its series hit, even at the same position.

## Panes

- `addPane({ share, scale: { mode, top, bottom }, left, format })`: `share`, its part of the height (0: no height, no separator, the panes either side of it one's neighbours); `mode`:
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
- `setLegend()`: its legend, over its plot's top left (made once, returned): `set(items, summary?)`, each
  `{ id, series, label, elements, hidden }` (the page's own `elements`, the first its key and name, the rest beside it;
  else made from its first series: a key of its color, its `label`, its value at the crosshair or its last in view, as
  its tag writes it; none in view, blank: a value from outside the view isn't shown as the view's). One row, scrolling
  sideways where its items don't fit (no scrollbar; the page makes it reachable and fades its edges); an optional
  `summary` caption before the items (words, or the page's own element: a color scale's key). The series pointed at is
  brought into view in it: the page's (`crosshair.pointed`, the crosshair on the plot or not: a tag, a row of its own),
  else the one the crosshair's pointer is on. Hidden ones left out. Its look the page's
  stylesheet's, over a few rules of its own (`.bc-legend`, `.bc-legend-item`, `-key`, `-name`, `-value`, `-summary`), in
  the theme's `font`.

## Series

- `Line({ values, color, width, colors, palette, breaks, tag, priceLine, stroke, format, outline, dots, marker })`:
  `colors` (a `Uint8Array` of indices into `palette`) colors each segment by its first point; `breaks` (a
  `Uint8Array`, 1 at a point) stops the line after it; `tag`, its value on the axis: its last in view (the crosshair's own tag says the pointer's; written by
  `format`, else as its side writes values, given its axis's step there; a side's tags laid top to bottom, each at its
  line or just under the one over it, the last cut off at the foot, one where lines read the same, none for a line out of the pane); `priceLine`, a dotted line across at it (drawn with it: the series added after it, their marks too, over it); `stroke: false`, no line (its tag and price
  line kept); `outline`, a color it's edged in (over a heatmap, a cloud, another line: it stands off them); `dots`, a
  dot at each point that radius; `marker`, the crosshair's dot on it (its radius, 3; 0: none); `steps`, drawn as steps
  (each value held flat to the next point, then straight up or down); `curve`, bowed through its points (zoomed in);
  `zones`, `[{ value, color }]` low to high: its colors by value, each from its value up, its own under them all, changing
  exactly where it crosses one (a line colored by levels, a baseline's by its base); `gradient`, a `Gradient`: its
  colors by value, its line by height in one stroke (one vertical gradient through its scale, log included, made again
  only as the gradient, the scale or the height change), its dots each by its own value; `dash`, its dashes; `partial`, its
  last point's period still running (a year so far): the segment to it dashed, where points have a pixel or more.
- `Band({ lower, upper, base, color, fade, below, belowFade, to, steps, zones, gradient, gradientFade })`: a fill between two columns (`steps`: its
  edges steps); without `lower`, an area down to `base` (`to: "top"` or `"foot"`: to the pane's edge instead); with
  `below`, a baseline (under its base in that color); `fade`, `belowFade`, the colors they fade to (gradients fixed to
  the pane: from its top to its foot, a baseline's to its base and from it); `curve`, its edges bowed as a curved line;
  `colors` and `palette` (`{ color, fade }` each), colored point by point, each segment in its first point's (zoomed in,
  where its edges cross on the way to a point in another color, changing color at the crossing: a cloud between two
  lines, its edges the two, colored by which one is over); `zones` (`{ value, color, fade }` low to high), an area
  filled by value, parted exactly where its line crosses a zone's value, each part fading as the whole would;
  `gradient`, a `Gradient`: an area filled by value, by height in one fill, each height's color fading to
  `gradientFade`'s there (a gradient as long) as far down the pane as it is (not a baseline's: its sides' colors
  say which side it's on).
- `Bars({ values, base, color, colors, palette, gradient, radius, partial })` (`gradient`: each bar by its value) (`partial`: its last bar at `PARTIAL` of its opacity), `Candles({ open, high, low, close, up, down, upWick,
  downWick, upBorder, downBorder, color, width, tag, priceLine, format, marker })` (a part's own color, else its body's: a hollow candle, its body the ground's):
  each with an `outline` (a color: each bar, each candle's body and wick, edged in it) and a `radius` (its corners', a
  number or one by its points' spacing). Under `BODIES_FROM` pixels a point (3), no candles: the close drawn as a
  line (a `Line`'s, in `color` and `width`; its scale fitted to the closes, marks on it, the crosshair's `marker` on it),
  candles from there. `tag` and `priceLine` (as a Line's) are the close's: its last value's badge on the axis and its line
  across, in the color of the candle they read while candles show (its border's, rising or falling), else in `color`.
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
- `Rows({ paint, below, outline, side })`: a set's members one under another, time across (a heatmap of members): `set(rows)`, each `{ values, name, color, id }` on the timeline, the first at the top, sharing evenly the room its scale keeps for values (its top margin, the legend's, left above); each point a cell colored by the page's `paint(value, row, pixels, at)` (none, left clear: a row draws nothing before its data starts). Painted once into an image (a pixel a point), drawn a row a `drawImage` each pass, smoothed only where its points are squeezed. `below(value)`: its region under the page's level, outlined in `outline`: by blocks of a power of two of points (a device pixel wide at least, each under where most of its points are: the same at any pan), then gaps and regions narrower than 4 CSS pixels merged away (a day-to-day flicker one region, not a comb of edges). Its names on `side`'s axis, each in its row's color: every row's where they fit, else as many as fit, evenly apart, the first and last always. Read on no scale; hit on a row with a value there (`rowAt(y, scale)`). `changed()` after a change of how it paints.
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
  each `{ at, position, color, text, textColor, size, priority }` at a point (its index), `"above"` it (a candle's
  high), `"below"` it (its low) or `"on"` it: a dot in its color (radius 3.5, times `size`; 0: none), ringed in the
  ground, resting on the point from its side (on it: centered), several at a point stacked away from it; its words just
  past its dot (on it: under it; at the timeline's ends, moved along it to stay a little within them), in a tag's pill
  of its color (or `textColor`) as it is, its words on it as a tag's (see `inkOn`, with a whisper of its hue where that
  still reads 4.5:1, else moved toward pure white or black as far as that takes); placed past its dot, over the series,
  outlined: a pill of the ground edged in that color, its words in it, kept at 4.5:1 or more. Where words crowd at the
  zoom, in the order of their `priority` (a move's size, say: the biggest first; at one, the latest first; those
  without, an event's name say, first, each counted only in another without), one whose words would run into those of
  one shown before it, in a row theirs can meet at some pan at that zoom (as far apart as the widest range its scale is
  fitted to in a view with both puts them: every series on it, hidden ones too; a relative or held scale's unknown, rows
  on one side always meeting, on two where the series crosses between them), is counted in the nearest of those ("2
  more", room kept for it on its left where words later in that order would run into its own; quieter than they are: in
  the theme's `text` color, kept at 4.5:1 or more, on a quiet tag of its `separator` color laid on its ground). The
  order is the data's: a pan regroups none, only a change of spacing does (a zoom, or the plot resized, as a wider price
  axis does), or of its scale (its mode, its height, its series' values). Its words over everything, on its pane's layer
  over it, drawn each frame, whole within the plot and clear of its other dots and of the words placed before them (in
  their order, the counts alone last): where its edge would cut them or they'd cover a dot, moved to the nearest spot
  that's clear (along their row, out from the series, or past their dot on its other side, over the series) where
  they're still nearer their own dot than any other, else a pill tried without its count, else left out; a count on the
  side of its pill away from its dot (right of it where the pill moves right or the count has no room on its left); one
  whose pill's dot is past an edge of the plot shown alone, where the words of the one it holds nearest that edge would
  be, while a mark it holds is in view. Their room kept at the scale's ends while its series has values in view (`room:
  false`, none); `set(marks)` after a change.
- `inkOn(color)`: white or black words on a CSS color, the chart's one rule for them (its tags', badges' and marks'
  pills'): APCA's pick, unless that falls short of WCAG 2's 4.5:1 there (its floor for small words), then the other
  (pure white's and black's ratios on a color multiply to 21, so one always reaches 4.58).
- `gradientTable(stops, steps = 256)`: a gradient's colors, `steps` of them evenly from 0% to 100%, as `oklch(…)` (a page
  may tone each for its theme): `stops` (`[place 0 to 100, any CSS color]`, in order) mixed in OKLCH, lightness and
  chroma straight and hue the shorter way round (a gray stop takes its neighbor's hue), so colors far apart stay vivid
  between (OKLab takes blue to yellow, or red to green, through gray); opacities straight; past the outer stops, theirs.
  The chart's fades are mixed by it too.
- `Gradient(colors, lo, hi)`: colors by value: `colors` evenly apart (a `gradientTable`'s, toned), a value's the one
  nearest its place from `lo` (0) to `hi` (1), past them the outer ones' (`lo` and `hi` the same, the middle one). One
  mechanism, applied two ways: continuous marks by height (a `Line`'s stroke, a `Band`'s fill), discrete marks each by
  its own value (`Bars`, a line's `dots`: a pass a color, so for them a table of about 64, not 256), so a mark's color is
  its value's wherever it's drawn. Never changed once made (made anew instead). `placeIndex(place, n)`,
  the index of a place (0 to 1) in `n` colors: what it and a page's own lookups (a heatmap's cells by strength) share.
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
- `Custom({ draw, extent, first, distance, axisTags, timeTags, margins, viewed }, { fixed, over, behind, crosshair })`: a page's own
  (bubbles, a heatmap, drawings, marks), drawn by its hooks as part of its pane's picture (see custom.js); its own tags
  on a price axis or the time axis (a drawing's value, its date), a price axis' over its lines'; `viewed`, told before each draw of its pane (what it
  draws found again from the view, once); `fixed`, held in place as the view moves (its pane drawn whole on a
  view or content change, kept while unchanged); `over`, over everything, the crosshair too, on a layer of its own drawn whole each frame (labels and handles
  placed in the plot); `behind`, behind every series; `crosshair`, drawn with the crosshair, each frame it shows, over
  its lines and under its dots (a line down its day); `changed()` after a change.
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

Its bench and its types: `website/_engine/` (`index.html`, `types.html`).
