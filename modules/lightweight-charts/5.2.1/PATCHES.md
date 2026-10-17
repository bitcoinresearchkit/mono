# Patches to lightweight-charts 5.2.1 (Bitview)

Applied by hand to `dist/lightweight-charts.standalone.production.mjs`, the only file Studio loads (the dev page imports
it, the build inlines it). A fresh download loses them: apply each again at its anchor (minified names are 5.2.1's),
then open `tests.html` (served from the repo) in a window wide enough for its two charts side by side and check it says
0 failing and every step the same.

The plots draw exactly what the library does, but for the log marks (1, 12) and the rounded bars (7, only where a radius
is set); the axes' tags are rounded (6), the time axis's labels fade before its ends (10), a price axis's tags stay inside
its pane (11). A view keeps a share of it on its data, an option (13). The rest (2–5, 8, 9, 14) draw the same, faster; a tag hides the mark labels under it (15). `tests.html` runs the patched file and 5.2.1 as published side by side through every
series type, every price scale mode (inverted too), pans, jumps, zooms, the crosshair, restyles, live updates, new data,
resizes, margins, moves between scales, a held price range and conflation, and compares their plots pixel for pixel, the
axes left out (713 steps).

## 1. Log scales: marks on round values

The library steps a log scale's marks down from the top of the range, each step a share of the value it's at: the marks
land on values like 6.31 (written "6") and shift as the chart pans. Its log marks are now round values, as many as fit
`tickMarkDensity` apart: every decade (every few, where they don't fit); in each decade 3, or 2 and 5, or more of 1 to 9;
then each stretch between two marks parted evenly (by 5, else 2) while all of its parts fit (a stretch only partly in
view, when at least two marks' height of it shows; zoomed into one, down to where marks show). None finer than the
price's least move (the format's `minMove`), no two written alike by the format. Ranges it can't mark (zero or below,
not finite, 60 decades or more, no room between marks: a density of 0), or where no round value fits, keep the
library's marks. Edge marks (`ensureEdgeTickMarksVisible`) over these: each mark closer to one than a mark's height goes
(the library drops one at each end by a linear span's measure).

- In `class pi` (the price scale's tick mark builder), `Zl(t,i,n,s,e,r)` (its marks between two logicals: in log mode,
  prices) starts with `if(a.ho()&&this.bvLogMarks(t,n,s,e,r))return;`.
- `bvLogMarks(t,n,s,e,r)` added to `class pi`, after `Gl()`; `Zl` keeps whether it marked (`bvUsed`), and `io()` (the
  edge marks) drops ours near them when it did.

`tests.html` runs it over 20,000 random ranges (10^-12 to 10^22, a millionth of a decade to 30, 30 to 2,000 px) and
hand-picked edges, each written three ways (unformatted; two decimals, a least move of 0.01, the library's default;
three significant figures, Studio's): it ends (well under a millisecond), its marks are in the range and its margins, a
mark's height apart, round at their step, none under the least move, none written alike; with no room between marks,
it leaves them to the library at once.

## 2. A point's style from its row, not a search for it

Building a series' points (each data change) and restyling them (each `applyOptions`: a color, a spotlight) asked the
bar colorer for every point's style by its index, a binary search each, with a copy of the row found. The row is now
handed over where it's known to be that point's: the series' own rows (not conflated ones) at the point's place, with
the point's index.

- `class Es` (a series' pane view): `bvOwn()` (the colorer, handing over the row being built from), used by the three
  `zM()` (line-like, bar-like, custom: `t.Wt=i` before each point is built).
- `Es.BM()` (the restyle): the row at the point's place, when its index is the point's.

## 3. Restyles in place

`BM()` made every point anew (`{...point, ...style}`): a restyle allocated as many objects as points, and new points
made the coordinates (patch 4) start over. It now assigns each point's style onto it (`Object.assign`): the same values,
the same points. Nothing else in the library holds or compares the points array (`kM` is only replaced, by `zM()`, never
changed in place; the renderers are handed it on each update).

## 4. Coordinates kept while their mapping holds

Every update mapped every point in view to x (index to pixel) and y (price to pixel), though a crosshair's move or a
restyle changes neither mapping, and a pan whose price range holds changes only x. Each is now kept, per series, with what it depends
on (`bvMap()`: the time scale's base index, right offset, width and bar spacing; the price scale's range, margins,
height, inversion, mode, log formula, and the base value of percent and indexed scales) and the stretch of points
mapped: the same mapping and the same points, only points newly in view are mapped; otherwise all of them, as before. A
view that jumps past the stretch starts a new one (a gap would hold stale coordinates).

- `Es.bvRun(slot, key, map)`; line-like and bar-like `OM()` call it for x (`Ic`) and y (`Jo`, `t_`). New points
  (`bvOwn()`, as `zM()` starts) let go of what was kept for the old ones.
- `bvMap()` on the time scale (before `Ic`) and on the price scale (before `Jo`).

## 5. Points built without spreads

`Rb(t,i,n)` (a point from a row: its fields, then its style) spread two fresh objects into a third for each point. It
now assigns the style onto the fields' object: the same keys in the same order, one object fewer per point.

## 6. Axis tags rounded at both ends

A price axis's tags (a series' last value, a price line's, the crosshair's) were rounded on their outer side only, square
where they meet the plot; the time axis's (the crosshair's date) at its bottom only. Each is now rounded at every corner,
by the same radius (2 px).

- `class L` (a price axis tag), `st()`: one `A(...)` call with `[l.ft,l.ft,l.ft,l.ft]` for either side (it chose
  `[l.ft,0,0,l.ft]` on a left scale, `[0,l.ft,l.ft,0]` on a right one).
- `class W` (the time axis's tag), `st()`: its path (square top, rounded bottom) is now `B(t,r,h,l-r,c-h,[d,d,d,d])`,
  the library's own rounded box.

`tests.html` leaves the axes out of its pictures (it compares the plots).

## 7. Candles' and histogram bars' corners rounded (an option)

`cornerRadius` on a candlestick or histogram series: a radius in CSS pixels, or a function of the bar spacing giving
one (TradingView's rounded candles plugin takes the same). Unset, 0, negative or not a number, the bars are drawn as the
library draws them. Set, one rule for both: a bar is rounded when it's 3 device pixels wide or more and a device pixel
or more from the next (touching, rounded bars would overlap), by at most half its width, where that comes to half a
device pixel or more; others keep the library's rectangles. A candle's body and its border are rounded (the border a
ring: the box less its inside, the inside's radius the border's less, a plain box when that's none); a histogram bar's
far end (from its base). The rounded bars of each color are one path, filled once a frame (they never overlap: the
order isn't seen).

- `bvRadius(option, spacing)`, `bvCorner(radius, ratio, width, gap)`, `bvPath(map, color)`, `bvFill(context, map,
  rule)` added (module level, before `B`).
- Candles: the pane view's `NM()` hands `cornerRadius` over (`bvR`); `class tr`'s `Vm()` (borders) and `wS()` (bodies).
- Histogram: its pane view's `NM()` the same; `class sr`'s `et()`, which also sets the fill color only when it changes
  (it set it for every bar: the same pixels, less work).

`tests.html`: with no radius (0, negative, NaN, a function giving 0) the pictures are 5.2.1's, step by step; with one
(Studio's rule, a third of the spacing, 2, 1000), at spacings from 0.5 to 60, rounding only takes from the bars (no
pixel painted where they left the background), changes no more than their corners hold, changes nothing where bars
touch (a device pixel apart or less), and changes something from 6 device pixels apart. Run at a true 2× too (Chromium with `--force-device-scale-factor=2`: an emulated ratio
gives 1× canvases, and Mullvad's fingerprinting protection scrambles a canvas read back).

Measured (Chromium at a true 2×, the library's time over a 2.5 s pan, medians of 6, before → rounded with Studio's
rule): candles 60 → 56 ms at a spacing of 6, 50 → 45.5 at 12; a two-color histogram 44.5 → 51 at 6 (+15%), 41 → 41 at
12. Not rounded (spacing 3), as before or less.

## 8. A series' data set again: the time scale rebuilt only when its points change

A series' `setData` took its old rows off every point and marked the data changed whenever it had any, so the time
scale was rebuilt (every point sorted again, every point's index and every series' row on it rewritten) for each series
set, though the points were the same: setting 40 series' data rebuilt it 40 times, each over all of them. It's now
rebuilt when points are made (a time no series had) or dropped (one no series holds any more), as the library already
does for new points; when neither, its rebuild found nothing changed and returned what it now returns at once. (A
series alone on the chart is still cleared and rebuilt whole, as before.)

- `class ms` (the data layer), `oM()` (a series' data set): the loop taking its rows off the points no longer marks a
  change; `_M()` (dropping points no series holds) returns whether it dropped any, and that marks one.

`tests.html`: "a series set again among others" sets one series' data again among others: the same times, fewer (the
others holding them), its own points let go, gaps over a stretch, more past every end, emptied, and back; with the
setData already in the lines scenarios (one series of four, fewer points).

Measured (Chromium, 39 layers, a layer added, every series set again): a day's 5,900 points 231 → 107 ms; ten
minutes' 14,800 points 484 → 225 ms; a chunk reached back (22,800 points) 721 → 328 ms.

## 9. Older data put before a series' own (`bvPrepend`)

A chart that holds a stretch of history and reaches further back (Studio at a fine interval: 8,000 older points a
series at a time) could only set each series' whole data again: every point it already had converted, matched to its
time and rowed once more, for every series, to add the older ones. `series.bvPrepend(items)` takes only the items
before the series' first: they alone are converted and put on their times (made where no series had them), placed
before the series' own rows, and the time scale is made again once if they added points (every point's index moves). The
same update then goes to the chart as `setData`'s would: what's drawn is what setting the whole data again draws. Items
not older than the first time the series holds (its gaps too) are refused (an error, as `update` refuses data older than
its last); none, nothing. One difference with setting the whole data again: a point's `originalTime` (the time as given,
which callbacks and formatters see) is the form of the first series holding it; a series' data set whole again puts it
after the others there, prepended it keeps its place. They differ only when series give the same moment in different
forms (a date string in one, a timestamp in another).

- `class ms` (the data layer): `bvPrepend(t, i)`, beside `oM` (setData).
- The chart (`JM`'s class): `bvPrepend(t, i)`, as `JM` hands its result on.
- The series' API (`setData`'s class): `bvPrepend(t)`, as `setData` (a full data change to its listeners).

`tests.html`: "older data prepended": a line with gaps and colors, candles, a colored histogram and an area, prepended
(as published, set whole again: they must draw alike): one series, then the others; gaps only; points no other
series held; then a live update, the whole set again, and an emptied series given older data.

Measured (Chromium, a chunk of 8,000 points reached back at ten minutes, with Studio's build only giving the new
points): 39 layers 721 → 162 ms (patch 8 alone: 328); 10 layers with 11 chunks held (87,000 points) 317 → 161 ms.
Mullvad, 10 layers: 250 → 133 ms at 3 chunks, 583 → 250 at 7.

## 10. The time axis's labels never cut at its ends

A date label centered on its tick near either end of the time axis was drawn across it, cut in half ("20" for 2020 at
the plot's left, after a pan). Each label now fades out over the last 12 px before either end (its alpha the share of
that distance left) and isn't drawn once it would reach one: as the chart pans, a label fades away at an end instead of
being cut there, and fades in as it comes back.

- `class` of the time axis widget (the one with `tg()`, the library's label nudge for fixed edges): `bvLabel(t,l,x,y)`
  added before `tg()`; its two label loops (`weight<s`, `weight>=s`) call it instead of `t.fillText(e.label,n,i)`,
  but for a label the library moves inside an end itself (`needAlignCoordinate`: `fixLeftEdge`/`fixRightEdge`, or
  scrolling and scaling off), drawn as it was.

`tests.html` leaves the axes out of its pictures. Checked in Studio by recording every label the time axis draws through
40 pans (264 labels: none crossing an end, 9 faded near one).

## 11. A price axis's tags inside their pane

The library pulls a tag in when its value sits within half a tag of the pane's edge, then spreads overlapping tags
apart, which can push one back across the edge with nothing to shift the rest by (a short pane: MVRV's base-line tag
"1" pushed under its last value's, cut in half by the pane's foot). After the spread, the tags of values in view (within
half a tag of the pane) and those the spread put in it are now moved inside where one crosses an edge, from the foot up (each above the one under it,
none past the foot), then from the top down: those beside one moved with it, none overlapping where they fit.

- `class Wn` (a price axis widget), the overlap fix (it calls `Fn` twice): `,this.bvInside(t,i)` after them;
  `bvInside(t,i)` added after it.

`tests.html` leaves the axes out of its pictures. Checked in Studio on the phone at its side (MVRV's tags in a short
pane) and squeezed log scales.

## 12. No log mark under the price's least move

Where patch 1 leaves a log scale to the library (a range reaching zero or below: a log axis dragged short, its foot past
nothing), the library steps it linearly, and its marks there read "0" and "−0.02" on a log scale. A log scale's marks
under the price's least move (the format's `minMove`), zero or below, now go, as patch 1's never go there.

- `class pi`, `Kl()` (rebuilding the marks): after the edge marks, `if(this.Ki.ho()){…filter…}`.

`tests.html`'s log marks are patch 1's (its own never go there). Checked in Studio: a log axis dragged short by 80 and
200 px (the phone review's repro, its "0" and "−0.02" gone), and a scale of values under one squeezed by 120 to 300 px.

## Measured (Chromium, CPU time in script; scratch benchmarks, before → after)

- 40 layers: a crosshair sweep −45% (Max) and −56% (1Y); a pan −18%, a wheel zoom −19% (Max).
- A load of 96,000 points × 3 series (a fine interval's history, at its most): 108 → 88 ms.
- A restyle (a spotlight's): 14.5 → 5 ms.
- Mullvad (Firefox), 40 layers at Max: the longest frame of a pan 167–200 → 83–100 ms.

## 13. The view kept on its data (an option)

`minVisibleDataRatio` on the time scale: the least share of the view's width its data keeps, 0 to 1 (unset or 0, the
library's two bars). The library holds every move to its least bars as it makes it (a pan, a fling's frames, a zoom, a
resize, new data, a view the page sets), so a view never shows past it, not even a frame; all of the data, where it
holds fewer bars than that share. (A page can't: a view it sets from its range-change handler is applied a frame later,
and a fling's frame, applied as the chart paints, shows past it first.)

- `class zi` (the time scale): `od()` (the least right offset) and `_d()` (the most) take `bvLeast()` for the library's
  two bars (`Math.min(2,this.Yu.length)`), not where `fixLeftEdge` holds the whole view, as before; `bvLeast()` added
  after `_d()`; `minVisibleDataRatio:0` in the time scale's defaults; its options applied (`vr()`), a share given holds
  the view to it at once (`this.Rc()`, the offset's correction).

`tests.html`: unset, the pictures (views put past either end among them) are 5.2.1's; set (a sixth, a half, the whole
view; 3,000 points, 40, 5), views put far past either end, offsets past them and a zoom out at the start keep that share
on the data (all of it, 5 points), a view on the data stays as put, and one past the end with none set takes the share as
it's given. Checked in Studio (a phone): flings and drags against either end never paint past it.

## 14. Canvases measured together

A canvas resized (`resizeCanvasElement`: a pane's, an axis's, a corner's; each widget has one or two) wrote its size and
read its box back at once (`getClientRects`, for its bitmap's device pixels), each read after another canvas's write: a
chart resized as the page around it changes laid the page out again for every canvas (Studio's two panes: 8 layouts;
Chromium at 4x CPU, a phone's sheet opened over the chart, a frame of 83 ms). The reads now wait for the writes: all are
read together once the script that resized them is done (a microtask), or at once as a canvas takes its new bitmap size
to be drawn (`applySuggestedBitmapSize`: a chart resized and drawn at once, `resize(w, h, true)`, draws at its new
size). One layout; the same frame, 33 ms.

- Before `var cn=function(){…}` (fancy-canvas's binding): `bvSizing` (the canvases waiting to be read) and `bvMeasure()`
  (reads them, `_invalidateBitmapSize()` each).
- `cn.prototype.resizeCanvasElement`: `bvSizing.size||queueMicrotask(bvMeasure),bvSizing.add(this)` for its
  `this._invalidateBitmapSize()`.
- `cn.prototype.applySuggestedBitmapSize`: `bvSizing.size&&bvMeasure()` first.

`tests.html`: the pictures (resizes among their steps, drawn on the next frame) are 5.2.1's; a chart of two panes resized
and drawn at once, then pictured in the same task (down, up, sizes of fractions, back), draws what 5.2.1 does, its
canvases' bitmaps as true to their boxes.

## 15. No mark's label under a tag

An axis's tags are drawn over its marks' labels, a label half under one showing at its edge ("100K" peeking above a
last value just under it; "2021" beside a drawing's date). A mark's label is now left out where a tag shown on that
axis covers it: on a price axis, a series' last value's and a price line's (within half a tag and half the text's
height of the tag's place, after the spread and patch 11); on the time axis, a primitive's (a drawing's dates; within
both halves' widths and 4 px, each where it's drawn: a tag pulled in from an end, a label moved in from one). The mark's tick stays; the crosshair's tags (drawn over, on their own canvas, moving)
hide none.

- `class Wn` (a price axis), `Bm()` (the marks): the tags (`this.Pm()`, those shown) read before the labels are
  written, a label written only where none covers it.
- `class Jn` (the time axis), `Bm()`: the same, the tags the panes' sources' (`Jn()`, each `dn()`, as `Jw()` draws
  them), the crosshair's (`Vd()`) left out.

`tests.html` leaves the axes out of its pictures. Checked in Studio: a last value at a mark, near one, a price line's
tag, a drawing's dates; on a phone.

## 16. A scale's range, read and set as it is

`getVisibleRange()` and `setVisibleRange()` don't meet on a log scale: the one reads prices rounded to the scale's
`minMove` (a small ratio's range read as 0 to 0), the other writes its numbers as the scale holds them (on a log scale,
logarithms, read as prices), so a range read and written back isn't the one read. And a scale auto scaled finds its
range only as it's drawn next (a view set is applied only then too): the range it'll take isn't known before it shows.
Studio glides a scale from one range to another (an interval changed by Auto: the old range to the new, frame by
frame), on any scale: three methods.

- `class` of `IPriceScaleApi` (`setAutoScale(t){…}`): `bvRange()` (the range shown, `{ from, to }`, or `null`: prices,
  unrounded, a log scale's converted with its formula as it is now, `ci()`), `bvSetRange(t)` (a range of prices held,
  auto scale off: a log scale's converted with its formula as it is now, `ui()`) and `bvAutoRange()` (auto scale on, its
  range found now, as its pane finds it when drawn, `hu()`, for the bars the view has as last applied: returned as
  `bvRange()` says it; a log scale's formula may change with it, and prices stay prices).

`tests.html` doesn't use them. Checked in Studio: a log scale, a linear one, a percentage one, glided between ranges.

## 17. Markers let go of their series

The series markers plugin (`createSeriesMarkers`) listens to its series' data as it's attached, by a callback of its own
made there and not kept, while its `detached()` lets go of one it keeps (`this.lx`, never set): detached, it listens on,
the series holding it and its markers (Studio detaches one each time a signal's marks move to another series). The
callback is now kept as it's subscribed, and let go of on detach.

- `class Fr` (the markers primitive), `attached(t)`: `this.lx=t=>this.QM(t),this.Te.subscribeDataChanged(this.lx)` for
  `this.Te.subscribeDataChanged((t=>this.QM(t)))`.

`tests.html` doesn't use markers. Checked in Studio: marks moved from series to series, a series' data set after.

## 18. A month's label not taken by a day's

The time axis's marks are placed by weight (years, then months, then days…), each where it's as far from those placed as
the widest label takes, in whole bars (rounded up). Months aren't as long as each other: at some zooms February's 28
days fall a bar short of that, so March's mark isn't placed, and a day's, the 2nd, a bar further, is, in its place
(the axis read "Feb 2 Apr"). Marks of a month or more (weight 60 up: months, years, of calendar lengths) now need nine
tenths of that room: 28 days of 31 are more than nine tenths; days and finer, all alike, need it all, as before.

- `class` of the tick marks (`Vu`, its marks by weight), its build's loop over weights: `bvNeed=i>=60?.9*t:t` with the
  weight's marks, and their test `_-o>=bvNeed&&o-u>=bvNeed` for `_-o>=t&&o-u>=t`.

`tests.html` leaves the axes out of its pictures. Checked in Studio: daily data from 7 Sep 2017 to 4 May 2018, its
axis "Feb Mar Apr" (was "Feb 2 Apr"); other zooms and intervals read as before.
