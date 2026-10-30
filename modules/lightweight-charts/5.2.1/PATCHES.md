# Patches to lightweight-charts 5.2.1 (Bitview)

Applied by hand to `dist/lightweight-charts.standalone.production.mjs`, the only file Studio loads (the dev page imports
it, the build inlines it). A fresh download loses them: apply each again at its anchor (minified names are 5.2.1's),
then open `tests.html` (served from the repo) and check it says 0 failing and every step the same. (Its two charts, the
published library's and the patched one's, one under the other at the same left edge: side by side, the second sat at a
fraction of a pixel and drew its bars and dotted lines a pixel apart, 175 steps differing between two copies of the
same published file. At a device pixel ratio of 2, two corner checks fail as they should not: a radius of 2 or more at a
3.5 px spacing, bars too thin for a corner to show; at 1 they aren't asked.)

The plots draw exactly what the library does, but for the log marks (1, 12) and the rounded bars (7, only where a radius
is set); the axes' tags are rounded (6), the time axis's labels fade before its ends (10), a price axis's tags stay inside
its pane (11), an indexed scale's labels are its series' (19), a curve's ends at the view's edges are drawn to its
neighbors where they are, not where the frame before had them (43: three pan steps, 6 to 27 values). A view held on screen as a right scale's width changes, an option (20). A view keeps a share of it on its data, an option (13). The rest (2–5, 8, 9, 14, 22, 23) draw the same, faster; a tag hides the mark labels under it (15). `tests.html` runs the patched file and 5.2.1 as published side by side through every
series type, every price scale mode (inverted too), pans, jumps, zooms, the crosshair, restyles, live updates, new data,
resizes, margins, moves between scales, a held price range and conflation, and compares their plots pixel for pixel, the
axes left out (713 steps).

## 1. Log scales: marks on round values

The library steps a log scale's marks down from the top of the range, each step a share of the value it's at: the marks
land on values like 6.31 (written "6") and shift as the chart pans. Its log marks are now round values, as many as fit
`tickMarkDensity` apart: every decade (every few, where they don't fit); in each decade 3, or 2 and 5, or more of 1 to 9;
then each stretch between two marks parted evenly (by 5, else 2) while all of its parts fit (a stretch only partly in
view, when at least two marks' height of it shows; zoomed into one, down to where marks show), a decade's stretches
alike (as long) parted alike, a finer step only where it fits in each of them in view (a log scale's lower stretches
are taller: parted alone, 63K to 88K was marked every 1K under 72K and every 2K over it). None finer than the
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
it leaves them to the library at once; 63K to 88K over 760 px, its marks evenly stepped; a range of subnormal
numbers (5e-324 to 1e-320, its steps underflowing to 0) ends.

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
axis covers it: on a price axis, a series' last value's and a price line's (within half a tag, half the text's height
and 4 px of the tag's place, after the spread and patch 11: a label touching a tag read as cramped against it); on the
time axis, a primitive's (a drawing's dates; within
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

## 18. A month's label not taken by a day's, nor its room

The time axis's marks are placed by weight (years, then months, then days…), each where it's as far from those placed as
the widest label takes, in whole bars (rounded up). Months aren't as long as each other: at some zooms February's 28
days fall a bar short of that, so March's mark isn't placed, and a day's, the 2nd, a bar further, is, in its place
(the axis read "Feb 2 Apr"). Marks of a month or more (weight 60 up: months, years, of calendar lengths) now need nine
tenths of that room: 28 days of 31 are more than nine tenths; days and finer, all alike, need it all, as before.

A month's mark left out for room, months dense at the zoom (shown every two), no day's mark goes in that month: its
number alone read as the month before's ("Sep 8", for 8 October, at a year of daily data). A year's mark left out is
its January's: the same. Months whose marks show, their days fill in as before ("Oct 20" after "Oct"); a month's mark
left out where the data jumps (a bar or two after the mark before it: a lone point a year earlier, a month missing),
its days as before (else a view inside it had none); finer marks (hours, minutes) as before, their labels whole times.

- `class` of the tick marks (`Vu`, its marks by weight), its build's loop over weights: `bvNeed=i>=60?.9*t:t` with the
  weight's marks, and their test `_-o>=bvNeed&&o-u>=bvNeed` for `_-o>=t&&o-u>=t`; a month's or year's mark not placed
  (in reach, at least half its own month's bars from the mark before it: a month's mark a bar or two after the one
  before, its month a few dozen, is the data jumping), its month (to the next mark of either: `bvB`, their places,
  sorted once a build, searched; or the data's end) kept (`bvSkipped`); a day's mark in one of them not placed (the
  months sorted, walked along with the days: a build O(N + B log B), N points, B months).

`tests.html` leaves the axes out of its pictures. Checked in Studio: daily data from 7 Sep 2017 to 4 May 2018, its
axis "Feb Mar Apr" (was "Feb 2 Apr"); other zooms and intervals read as before. A year of the price: "2026 Mar May Jul
Sep" (was "… Sep 8"); 3 months, 6 months, hourly: as before. Codex's reviews (a harness of the builder against the
published one): 390 data jumps of one to five bars at rooms 2-40, weekly and monthly data as before; left: a jump into
a last month only a few days long, at rooms 3-5, its days left out (a view of that month alone blank), and a view
narrower than one label inside a month left out (one label before, none now).

## 19. An indexed scale's labels as its series writes them

A scale indexed to 100 wrote its labels with a formatter of its own, two decimals and no bound ("10000000000.00" for a
series a hundred million times its first value in view), not its series' format as the linear, log and percentage
scales do: a wide axis, as wide as its longest label, its width changing as the view moves. Its labels are now written
by its first series' formatter (its `priceFormat`), the library's own still where it has no series.

- `class Mi` (the price scale), `vl()` (its formatter chosen): `this.No()?(this.el=null!==this.So?this.So.sl():new X(100,1),i=100)`
  for `this.No()?(this.el=new X(100,1),i=100)`.

The scale's marks still step from a base of 100 (its `minMove` its own): a series' format coarser than its steps (Studio's
ticks, three digits) writes neighbours alike, 100.05 and 100.1 both "100"; Studio's tick formatter parts them. As the
linear branch, the formatter follows a left or right scale's first series as the order changes, not an overlay's.

`tests.html` compares plots with the axes left out. Checked in Studio: an indexed pane over a series rising from cents to
billions, its labels short ("1.5B"), its axis as wide as the other scales'; zoomed in close, its ticks apart ("100.05").

## 20. The view held on screen as a price scale's width changes (an option)

A price scale is as wide as its labels (or its `minimumWidth`), so it widens and narrows as the view moves ("885.7M" to
"1.621B"), and the plot with it. The library keeps the bars' spacing and the right offset in bars as the plot's width
changes: a left scale's change leaves every bar where it was on screen, a right scale's moves them all by its change (the
plot's right edge where the bars are counted from). `bvHoldOnAxes` on the time scale (unset, as before) holds them on
screen there too: the chart's own width the same, a right scale's width changing by d pixels takes d / spacing bars off
the right offset, as the plot's width is set; but at the latest (the right offset 0 or more: the last bar in view, its
room after it in pixels, `rightOffsetPixels`), the view is the latest's, kept as the library keeps it (a wider scale
would cover the room and the last bar). A drag (its start's offset) and an animated scroll (a fling, an animated
`scrollToPosition`: their frames' positions, by the total moved since each started) carry it, so their next frame doesn't
undo it. The chart's own width changing (a resize), it's the library's as before.

- `class` of the chart widget, its size laid out (`zg()`): before the model's width is set (`this.sn.H_(r)`),
  `this.yn.timeScale.bvHoldOnAxes&&this.bvWidth===s&&this.hg!==n&&(this.sn.ia.bvPending=n-this.hg),this.bvWidth=s,`.
- `class zi` (the time scale): `bvShift=0` (the total moved so) and `bvPending=0` in its constructor; `bvHold(t)` after
  `$c()`: the offset, the drag's start's (`this.nc.Oc`) and `bvShift` less `t/this.oc` (none where `this.lc>=0`); its width set (`H_(t)`) takes the
  pending change with the width, after the view as it was is read (`this.Sc()`, for `fixLeftEdge`: read after the offset
  moved and before the width, a view half moved, told to the page as the view); the width the same (the scales' changes
  even), the offset corrected at once (`this.Rc()`); `bvHoldOnAxes:!1` in its defaults.
- The chart widget's frame (`yg(t,i)`): a scale's width changed as it's drawn, the library lays it out again and applies
  the frame's time scale changes again, at the new width (a range asked for, shown exactly at it: its bars narrower or
  wider than the hold keeps them). Held, only the changes asked for since are applied there, the frame's merged in after
  (`this.bvHeld||this.ag.Ss(t)` before, `this.bvHeld&&this.ag.Ss(t)` after `this.Fg(this.ag,i)`).
- A range asked for (`setVisibleLogicalRange`) is applied a frame later, its scale perhaps of another width by then (a
  range asked for as the chart's box resizes, its labels laid out again after): it carries the right scale's width and
  the box's as the page had them (`bvRight`, `bvBox`, the time scale's, set as each layout starts, `zg()`, before the
  size change is told), and applied (`nd(t)`) in the same box with that scale's width changed by d, it's applied at the
  plot's width then (spacing `(this.k_+n)/i.Iu()`, its right edge `i.bi()`) and held by d (`bvHold(n)`), before the panes
  are told (`this.sn.zc()`, `this.sn.mr()`: their price ranges by the bars then in view); else as before (`this.Qc(i)`).
- The animation's frame (`case 5` of the invalidations applied): `n.Ms(t.Wt.Gc(i)+(n.bvShift-(t.Wt.bvFrom??=n.bvShift)))`
  for `n.Ms(t.Wt.Gc(i))`; its start as it's asked for (the model's `ps(t)`: `t.bvFrom??=this.ia.bvShift`), not its first
  frame: a scale's change before it, held, isn't undone by it.

`tests.html` leaves it unset. Checked in Studio (no `minimumWidth`, each scale as wide as its labels): pans, flings, wheel
and pinch zooms with a right scale widening and narrowing as they go, every bar where the move puts it, never a frame
off; resizes as before.

## 21. A price scale's width before it's laid out: 0

`priceScale(side).width()` of a scale made visible (`visible: true`) before the chart lays it out (its axis drawn next)
threw ("Value is null": the pane's axis widget not made yet). It's 0 until then, as the room it takes: the plot's
width and the scales' then still add up to the chart's.

- `class` of the chart widget, the price axis's width (`Dg(t)`): `return("left"===t?this.ng[0].yw():this.ng[0].Pw())?.Tm()??0`
  for `return a("left"===t?this.ng[0].yw():this.ng[0].Pw()).Tm()`.

`tests.html` doesn't read a scale's width then. Checked in Studio: a layer on a pane's left scale, the first there, the
chart's width read as it's added.

## 22. A gradient asked for by its values

An area's fill, a baseline's fill and its line ask the gradient cache (`class Fe`, `Nb`) for each point's style: each
call built an object of its seven values for the cache to compare with the last one's, an object a point in view, each
frame. `Nb(t, ...)` now takes the seven values themselves and keeps the last ones in fields: the same comparison, the
same gradient made when one changes, no object.

- `class Fe`: `Nb(t,s,e,r,h,a,l,o)` (was `Nb(t,i)` with `i` the values' object, kept as `Fb`).
- Its callers: the baseline's `Ob` (`class We`) and `kb` (`class He`), the area's `Ob`.

## 23. One style for every point: none asked per point

A line's or an area's path asked its style function for each point in view, each frame (`fe`: `r(t, v)`), to start a
new stroke where a point's style differs, though a series without points of their own colors has one style for all. Its
points now say so: building them (`zM()`, the three: the colorer's styles, each compared key by key with the first's,
`bvSame`, in `bvOwn()`'s `Sh`) and restyling them (`BM()`) mark the points array (`bvU`), and `fe` takes the first
point in view's style for every one when it's set (`u=i.bvU?c:r(t,v)`).

`tests.html` runs a line with points of their own colors beside ones without, restyled mid-pan. A baseline whose
price crosses its base has two line colors (`sh`), so isn't one style: drawn as before.

Measured (Chromium, 1440×900 at 2x, a pan and a wheel zoom at Max, 25 layers; CPU time in script, before → after
22 and 23): stacked areas (HODL waves) 1,698–1,725 → 1,325–1,352 ms (−22%; the paths' own functions 904–931 →
523–535); lines (realized price by age band) 1,620–1,653 → 1,253–1,298 ms (−22%).

## 24. A crosshair set on a series, at its own scale's price

`setCrosshairPosition(price, time, series)` placed the crosshair through its pane's default price scale (`Gd`:
`n.kn()`), whatever the series' own: a price given for a series on the other side (a ratio on the left, the price on
the right) landed at that price on the default scale, off its line, often off the plot. The series is passed on, and
`Gd(t,i,n,o)` converts through its scale (`o.Ft()`), the default's only without one.

- The API's `setCrosshairPosition`: `Gd(t,i,e,s)` (`s` the series' model).
- `Gd(t,i,n,o)`: `const s=o?o.Ft():n.kn()`, its base the series' own first value as `priceToCoordinate` takes it
  (`o.zt().Wt`: on a percentage or indexed scale shared with another series, the scale's first value can be that
  one's), the default scale's without a series; none (nothing in view) places nothing (it asserted, a throw).

Studio's keyboard crosshair walks the selected layer, which can be on either side. `tests.html`: 713/713 the same (it
sets no crosshair).

## 25. A custom series' bars kept between frames

A custom series' view handed its renderer every bar as a new object each frame (`NM`: `bars: this.kM.map(Ls)`, each
`{x, time, originalData, barColor}`), all of them, in view or not: a chart of 30 custom series (Studio's clouds) at
Max made some 180,000 objects a frame. The bars are kept (`bvBars`) and only their `x` and `barColor` written each
frame; they're made anew when a point isn't the one its bar was made from (its time or its row: new data, older data
put before it, a point updated), or the count differs.

- `zs.NM()`: the kept bars checked point by point against `kM` (`time` is `wt`, `originalData` is `HM`), updated
  (`x` from `_t`, `barColor` from `sh`), else `this.bvBars = kM.map(Ls)`.

A renderer reading `bars` gets the same values as before; one keeping the array, or a bar, across frames now sees it
change (Studio's cloud reads them each draw, keeping nothing). Measured (Chromium at 4x, a pan and a wheel zoom at Max
of "Bedrock landscape, awake", 33 layers, most clouds): median frame 50 → 33 ms, script −16%; the plot's pixels
identical with and without it (a screenshot compared, live price held); `tests.html` 713/713.

## 26. A custom series' prices placed from its first value as updated

A custom series' renderer turns prices into coordinates through the view (`WM`), and each call asked the series for
its first value in view afresh (`this.ae.zt()`: the visible range, a search for its first point, a new object):
Studio's clouds ask it for each pixel column's high and low, tens of thousands of times a frame. The view's update
(`LM`, which computes that first value to place its points) keeps it (`bvZ`, cleared when it starts), and `WM` uses it
(the series' own, asked, without one: an update that stopped early).

- `Es.LM()`: `this.bvZ=null` first; `const s=this.bvZ=this.ae.zt()`.
- `zs.WM(t)`: `const i=this.bvZ??this.ae.zt()`.

The same frame's first value either way: it changes with the data or the visible range, which run the update again.
Measured (as 25): `WM` 675 → 467 ms, script −9%; the plot's pixels the same; `tests.html` 713/713.

## 27. A frame's changes drawn at once, asked

The chart draws what's asked of it (a range set, options) on its next animation frame. A page that needs it drawn now
(Studio: a view put back is shown, drawn so its price scales are laid out for it, and shown again, see 28) had no way to.
`chart.bvFlush()` draws the changes waiting at once, as a forced resize does (`wg` with its third argument), without
redrawing what hasn't changed; nothing waiting, nothing's done.

- The chart widget: `bvFlush(){if(!this.lg||null===this.ag)return;window.cancelAnimationFrame(this.eg),this.lg=!1,
  this.eg=0;const i=this.ag,t=performance.now();this.ag=null,this.yg(i,t);for(const n of i.bs())if(5===n.ds&&!n.Wt.Kc(t))
  {this.Qt().ps(n.Wt);break}}` before `Md(t)` (the frame's own callback, the same steps).
- The chart's API: `bvFlush(){this.hv.bvFlush()}` after `resize(t,i,n)`.

`tests.html` doesn't call it: nothing else changes. Checked in Studio: a chart opened and reloaded, its view as kept.

## 28. A scale never laid out isn't held against

Patch 20 holds the bars on screen as the right scale's width changes, a range asked for included (applied over the
plot as it was when asked, then held). A chart's first draw asks for its range before its scale is laid out (its width
0 then, see 21): the scale then took its room from the range's left edge, the view narrower by the scale's width than
asked. Studio, which puts a view back as it was left, lost that much of it at each reload. Until the right scale has
been laid out with a width once, nothing's held: the range is shown over the plot as it is, as without the option. (A
scale hidden and shown again was laid out: held both ways, as before. One shown for the first time after the chart's
first draw (only left scales till then) isn't held either: the view shown over the plot as it is, a scale's width less.)

- `class zi` (the time scale): `bvLaid=!1` in its constructor.
- `nd(t)` (a range applied): `t.bvLaid` for `void 0!==t.bvRight`; `setVisibleLogicalRange` carries `bvLaid:i.bvLaid` with
  `bvRight` and `bvBox`.
- The chart widget's layout (`zg()`): `this.sn.ia.bvLaid&&` before `this.hg!==n&&(this.sn.ia.bvPending=n-this.hg)`,
  and `n>0&&(this.sn.ia.bvLaid=!0)` after it.

`tests.html` leaves `bvHoldOnAxes` unset. Checked in Studio: a view left zoomed out at the latest, reloaded three times,
the same each time; pans and zooms with a scale widening as before.

## 29. A line's points it'd pass within a quarter of a pixel of, left out (an option)

Firefox strokes a canvas's paths on the CPU where WebGL is off (Mullvad Browser, Tor Browser): Studio's busy charts (24
lines of ~2,700 points in view) took 45 ms to draw at 2x (Chromium: 2 ms), all of it the strokes, by their points
(half the points, 27 ms; their width no matter). A series with `bvSimplify: true` draws its line through fewer: a point
is left out where the segment drawn from the last point drawn passes close enough to it, and to every one left out
since: within a quarter of a CSS pixel (half a device pixel at 2x) while bars are a pixel or more apart, more as they
crowd (a quarter of a pixel by the bars to a pixel), up to a whole pixel (`bvR`: zoomed out, a little less exact; zoomed
in, as exact as ever), by the smaller of the two pixel ratios. Kept by a cone of the directions from the last
point drawn (its anchor) that pass that close to each point left out (the tangents to a circle of that radius around
it: exact; kept as two directions, each a point's own turned by the tangent's angle, by its sine and cosine: square
roots alone, no trigonometry, a point in the cone by two cross products' signs); a point joins it where its direction is in the cone and it's no nearer the anchor than those left out
before it (their nearest place on the segment, then, not past its end), and within a million pixels of it (an infinite
or far point never joins: the drawing goes on as the canvas has it); one within the radius of the anchor joins while
those left out are all as near. Any other ends the segment at the one before, drawn, the cone started again from it. A
line's path only (`fe`, a simple line, no dash, one style for all its points, see 23): stepped, curved and dashed
lines, and those whose points have their own styles, as before.

- The points' arrays: `this.kM.bvS=!!this.ae.N().bvSimplify` with `bvU` where they're built (`zM()`, the three), and
  `s.bvS` where they're restyled (`BM()`, an option changed).
- Walked again in the same paint (an area's fill, then its line: see 30), its points aren't simplified again: the kept
  ones are recorded on the points' array (`bvKeep`: the paint, `bvPaint`, counted by the pane's `bvViews`; the range;
  the radius in CSS pixels; their indices) and drawn as they are (`bvRp`): its fill's edge and its line the same points.
- `fe`: `bvZ` (a simple line, no dash, `bvU` and `bvS`), `bvR` (`Math.min(o,_)` times a quarter of the bars a CSS pixel, a span not a number the quarter alone,
  in view, from `.25` to `1`) and `bvR2` (its square); in its case 0, the point checked
  against the cone (`bvLx`/`bvLy` and `bvUx`/`bvUy`, its two directions; `bvF`, none yet: all), kept pending, or the pending one drawn (`lineTo`) and the cone started again from
  it; the last pending one drawn before the path's end (the stroke, an area's fill, by the path's own end).

`tests.html` leaves `bvSimplify` unset: its pictures the same as before the patch. Checked on synthetic paths (the
loop as the file has it: random walks with spikes, a smooth curve, steps of 0.05 to 2 device pixels, spikes and turns
at a pixel's width, NaN, infinities and points a long way off): no point left out further than the radius from the
path drawn; 97% of a smooth curve's points left out, half a noisy one's. Measured (Mullvad Browser, headless, 2x, one
redraw for a 5-bar pan, 8 years in view): "Realized price by age band" (24 lines) 44.7 → 11.9 ms, panning ~25 → ~57
fps; HODL waves (24 stacked areas) 68.9 → 40.0 ms; Chromium 2.1 → 3.1 ms. Its pixels (Mullvad, 2x): 3.4% of the busy plot's differ, by a
little (antialiasing), 0.4% by more than a quarter, none to be told apart by eye.

## 30. A pane's fills drawn at half resolution, under its series (an option)

Firefox fills on the CPU where WebGL is off (see 29): Studio's stacked areas (HODL waves, 24 translucent gradients each
to the pane's floor) spent 22 ms of a 40 ms frame filling at 2x, half of it the gradients. With the chart's
`bvHalfFills: true`, each pane's area and baseline fills (soft gradients, their edges under their lines) are drawn
first, at half the pane's bitmap size, then laid on it at its size (only where one was drawn), and its series drawn on
as before, their fills left out. The fills look the same (their gradients soft, their top edges under their lines);
every series of a pane is drawn over every fill now, so a line or a series under an area stacked over it isn't tinted
by that area (a touch crisper).

- `bvFilled`, a module-level flag: whether the pane being drawn has its fills drawn already; `bvDrew`, whether any was.
- `Ne` (the fills' renderer, area and baseline): `st` skipped while `bvFilled`; `bvFills(t,i,n)`, its own `st` (`bvDrew`
  set where it has something to draw).
- `C` (a composite renderer) and `Ue` (a series' fill and line): `bvFills` passed on (to all; to the fill).
- The pane widget: `bvHalf(t)`, with the option: its own canvas at half the bitmap's size (`bvCanvas`, kept, in the
  chart's color space, made again if that changes; cleared when the last frame drew on it), a rendering target on it
  (`fn`, the pane's media size: its pixel ratios halved), every source's views drawn with `bvFills` alone (`Dw`, as `Sw`
  walks them), the canvas laid on the pane's (`drawImage`, scaled) where a fill was drawn; `bvViews(t)`, in its paint
  (`Rm`) after the background, the bottom views and the grid (`xw`): `bvHalf`, then the views (`Sw(t,Un)`) with
  `bvFilled` set, the flag as it was after (another chart painted meanwhile, an exception).
- The gradients' cache (`Fe.Nb`, 22): keyed by the target's vertical pixel ratio and bitmap height too (the half
  target's), its bottom given as asked (media) or none (the bitmap's: the three callers pass `n.qb`, or nothing), no
  longer told apart by its number.

`tests.html` leaves `bvHalfFills` unset: its pictures the same as before the patch. Measured (Mullvad Browser,
headless, 2x): HODL waves, a redraw 40.0 → 25.6 ms, panning ~25 → ~36 fps; a chart without fills (24 lines) as before
(10.6 ms, 60 fps). Its pixels (Mullvad, 2x, HODL waves): 11.8% of the plot's differ, by 37 of 255 at most.

## 31. A chart drawn again under the pointer only where its drawing can change

As the pointer moves from one series to another (a band to the next in a stacked chart, every few pixels), the model
(`Rd`, the hovered object set) asked for the series left and the one hovered (`Ta`, a light update: `new Y(2)`), and so
drew the whole chart again (every pane, its price scales autoscaled again, both price axes, the time axis): for the
hovered series drawn last (`hoveredSeriesOnTop`) and for a custom series' renderer, which is told whether it's hovered.
Nothing else drawn depends on it: a built-in series' renderers aren't told (their `et` takes the target alone), a
primitive's are given a constant (`Vt`, `setLineStyle`), the crosshair (its marker too) is drawn on the pane's top
canvas (`Ya`, asked just before), and the pointer's cursor is set apart (`pg`). Studio's charts set
`hoveredSeriesOnTop` false, so a sweep over HODL waves (24 bands) drew the chart again on every band crossed.

- `Rd`: `bvH(t)`, `hoveredSeriesOnTop` or the series a custom one (`"Custom"===t?.wa`; a pane, `Si`, has none); the
  series left (`Ta(i.uu)`) and the one hovered (`Ta(t.uu)`) asked for only where `bvH` holds of it.
- Still drawn again: a hover onto or off a custom series (Studio's clouds draw the same either way; the library can't
  know). No longer: a primitive that keeps the crosshair's `hoveredObjectId` and draws by it without `requestUpdate()`
  (it was drawn again by this, by chance; Studio has none: its primitives have no `hitTest`).

`tests.html` as before. Measured: Chromium, 65 moves over HODL waves, 104k `lineTo`s → 12.7k (one frame, the live price's;
as many as with the pointer still); Mullvad Browser (headless, 2x), the crosshair swept over HODL waves, 102 → 110-119
frames in 2 s.

## 32. A series set again once it's been alone: its points kept

A bug of the published library. A series alone set again makes its points anew (the map of them cleared and filled);
with the same times, the time scale saw nothing changed and the sorted points stayed the old ones. The next series set
again took its rows off those, held by no one, so every point was let go: the time scale emptied, a view set then was
set against no points (Studio: undoing a step that left one layer, the view lands a day wide, shifted by all its bars
once they're back), and one set again with fewer let go of another series' own. The sorted points are now the ones made,
whether or not their times changed.

- `class ms` (the data layer), `cM()` (the sorted points replaced): `this.lM=t` before its `return -1` too.
- `wM()` (a series' latest rows taken off): emptied, its points sent as emptied (`Math.max(0, …)`, as `setData([])` sends
  them), not left to the time scale holding the same array (it may hold the old one now).

`tests.html`: "Points kept" (the patched library alone: a series alone set again, another added, the first set again,
then with fewer; the view and the other's latest point there). Studio: an undo of a formula in a pane of its own over
one layer, at Max, keeps the view (−258 to 6,309 bars, as before the step).

## 33. A pan anchored where it was pressed

The library tells a drag from a click by 5 px of travel (`Rp`), then began the pan from where the pointer was at its
next move: the chart trailed the pointer by that travel for the whole drag (8 px on a slow drag, 24 on a fast one). The
pan now starts from where the press was, so its first frame catches up and it stays under the pointer.

- The pane's press handlers, `Fp` (mouse) and `Op` (touch): the press kept (`bvPress`: its client and local x, y).
- `cw()` (a pressed move): its anchor (`Hm`) taken from the press, not the move; the scroll begun at the press's local
  x, y (`Ud`, `nu`), not the move's. `dw()` (the drag's end) lets the press go, as does `$p()` (a pinch's start): a
  finger dragging on after a pinch pans from where it is then, as before.

Measured: Mullvad Browser (headless, 2x), a 60 px drag in 2 px steps: the chart's travel equals the pointer's from the
first frame past the threshold (it was 8 px short throughout).

## 34. A fling that would pass the view's end eases into it

A fling glides on from where the finger lifts, its speed falling by a fixed friction (`class In`: its travel, as it
starts, `-speed / ln(friction)`), and the time scale holds every frame to its ends (patch 13): a fling that would carry
the view past one was cut off there at its speed, a stop in one frame (Studio on an iPhone: 47 to 129 bars a frame, then
none). Now, as it starts, a fling that would pass an end has its friction raised to end there: the same speed as the
finger left it, falling faster, the view coming to rest against the end.

- `class In`: `bvEndBy(at, least, most)` after `Kc()`: the travel as it starts against the room to `least` or `most`
  from `at` (bar offsets, as `Oc()`); past one, the friction set to stop there (`exp(-speed / room)`), its duration as
  the constructor's; none left (or the room the other way), it ends at once.
- The pane's press handler (`dw()`, a drag let go): after `this.Xm.me(n.Oc(),t)`, `this.Xm.bvEndBy(n.Oc(),n.od()??-1/0,n._d())`.

Studio, the iPhone simulator, a fling at the latest bar's end: 121, 74, 37, 21, 11, 5, 3, 1.5 bars a frame, at rest in
150 ms (before, the same fling: 131 … 47, then held).

## 35. A tag's words white or black by its lightness

A price axis's tag (a series' last value, its price line's, the crosshair's) writes its words black or white by the
tag's grey (`0.199 r + 0.687 g + 0.114 b` over 160: black): a mid green, an orange, a blue, a violet took white at 2.7
to 4:1 (Studio's MVRV green 2.7). Then by APCA (its Lc, white's against black's): better, but by luminance, which at one
lightness swings with hue: a palette of one lightness (Studio's HODL waves' 23 bands, OKLab L 0.72) split by hue, its
pinks and oranges white, its blues to yellows black, the lighter-looking ones taking the white. Now by the tag's
lightness as OKLab measures it: black from L 0.7157, where APCA flips for a grey (its luminance, `v^2.4` of the
sRGB value, 0.342: `#a4a4a4` and lighter, the two reading alike), so a grey goes as APCA has it and a hue as a grey of its lightness. Bitview's palette as before but amber (black
now) and lime (white); the HODL bands black, all of them. Not WCAG 2's ratio, which overrates black on a vivid color: it gave a
red, a blue, a pink black (white reads twice as much there).

- `generateContrastColors` (minified: the method returning `{Z:` the background, `X:` the foreground `}`): `X:` white
  where the color's OKLab L (sRGB linearized, its LMS cube-rooted) is under 0.7157, else black; instead of
  `S(i)>160?"black":"white"`.

Studio's own tags (a drawing's on the axis, a solid text's ground) pick alike, from the same hex (`textOn`).

## 36. A pan's line drawn from the frame before's points

A pan moves every point the same way, sideways: the same bars a pixel, the same prices to the same heights. Yet each
frame simplified each line again (29), point by point over all of them in view: 24 stacked areas of ~3,100 points,
~77,000 a frame. Where JavaScript runs without its JIT (iOS's Lockdown Mode: Studio's HODL waves on the phone), that
walk was most of a frame (pans at 40 ms). Now a line's points kept in the frame before are drawn again where only the
view moved: the same points array (new data is a new one), the same price mapping (its `bvMap`, see 4), the same bar
spacing, pixel ratios and radius.

After any point it keeps, the walk goes on exactly as one started at that point would: the point after it, a cone from
it, nothing else carried. So the new frame's walk, from its own first point, joins the last frame's at the first point
both keep; from there it's the last frame's points, drawn (but its last, kept only as the end of that frame's path),
then the walk again from the last of them on to this frame's end. A frame's walk is its first points, until the two
meet, and the ones newly in view; their coordinates are this frame's, each point within the radius of its line as
before (the same points, moved by a fraction of a pixel's rounding).

- `fe`: `bvB` (its bar spacing, `e`, as the `else` that shadows it starts); the kept record (`bvKeep`) holds the price
  mapping (`y`, the points' `bvY`), `b`, `o`, `v` (pixel ratios) beside the paint, range and radius; a new paint with them
  all the same (the radius to a billionth: its span's division rounds) takes the last list (`bvO`) and starts a new one.
  As a point is kept (`lineTo` of the pending one), `bvJ` walks `bvO` (sorted) to it: the same, its later points before
  this frame's end are drawn and recorded, the walk set back to a start at the last of them (`h`, `bvAx`/`bvAy`,
  `bvP`, `bvF`, `bvD`), `bvO` let go.
- Line-like `OM()`: the price mapping's key (`t.bvMap(n)`, already made for `bvRun`) set on the points as `bvY`.

`tests.html`, 7: a simplified line and an area panned a step a frame (12 steps, back and forth), their price range held
(the points kept) then free (made again), fills at half resolution (30), each frame's strokes and fills within the radius
of every point in view, at their own canvas's pixel ratios (the pane's; the fills' own, half), a frame drawing no line
(or an area no fill) failing: 144 checks, 0 failing, at 2x and at 1.25x; with the join taken at a later point than the
one both keep, 78 failing.

Measured, HODL waves (24 areas, 8.5 years in view), panning as a drag does (the right offset alone): the iPhone (Lockdown
Mode), 2 s: 49 → 86 frames, median 40 → 22 ms, lost 1.2 → 0.6 s; a pan step's redraw, Chromium 2x 2.3–3.2 → 1.0–1.3 ms,
Mullvad 2x 22.8 → 21.8 ms (its frame is the canvas's), windowed 24.4 → 22.5. "Realized price by age band" (24 lines):
Chromium 2.2–2.8 → 0.9–1.0, Mullvad 11.1 → 10.0. A zoom (its bar spacing changing) walks all as before.

## 37. A point's x with its constants read once

`Ic` maps each point in view to x: it read the time scale's right offset, width and bar spacing again at each point, and
the point from its array twice. A JIT keeps them in registers; without one (iOS's Lockdown Mode), each read is a
property lookup, at every point of every series, every frame of a pan (HODL waves: 25 × ~3,100). Now each is read once
before the loop, the point once, the same arithmetic in the same order (`n + lc`, less the point's index, then as
before): the same values, bit for bit.

- `Ic(t,i)`: `n` is `Dc() + lc` (its sum as the loop made it), `h` the width (`k_`), `o` the bar spacing (`oc`).

Measured on the iPhone (Lockdown Mode), x for 25 × 3,105 points: 7 → 4 ms, every value the same; a pan step's redraw in
Chromium (1.0-1.2 ms) and Mullvad (22-23 ms), as before. `tests.html`: its pictures, 713 of 713 the same.

## 38. A line's simplification with one root a point

Patch 29's walk narrows its cone, at each point it leaves out past the radius, to the two tangents from the anchor to
the circle around that point: as unit vectors, two roots and four divisions a point (`d = √q`, `c = √(1 − (R/d)²)`, the
direction `x/d`, `y/d`). Every test made with them is a sign (a cross product's, a direction in the cone or not, one
edge left of another), and a vector scaled by any positive amount keeps its signs: the tangents are kept unnormalized,
`(x·c ∓ y·R, y·c ± x·R)` with `c = √(q − R²)` (the unit ones times `d²`), one root and no division a point; the same
decisions in exact arithmetic (a point exactly on a cone's edge, rounding may tip it the other way than before: either
way within the radius; 10,000 random paths, 475,846 points, kept the same). Its running greatest distance (`bvD`) by a
comparison, not `Math.max`'s call.

- `fe`, the cone's narrowing (a point joining past the radius) and its start (a new anchor): `c=Math.sqrt(q-bvR2)`, the
  tangents from `x`, `y`, `c` and `bvR`; `q>bvD&&(bvD=q)`.

Measured on the iPhone (Lockdown Mode, no JIT): the walk over 25 × 3,100 points (HODL waves' size) 26 → 19 ms, the
points kept the same there (3,718 of 77,500); HODL waves zoomed as a pinch does, 2 s: median 29-32 → 26-29 ms a frame, lost
0.95 → 0.83 s, its pan 60 fps (median 16 ms). Chromium, a zoom step's redraw 2.3-2.8 → 2.1-2.4 ms; Mullvad 24-25 → 24 ms
(its frame the canvas's); a pan's as before. `tests.html`: pictures 713 of 713 the same, a pan's lines within their radius
(7) 144 checks, 0 failing.

## 39. A zoom's line from each pixel's lowest and highest

Zoomed, every point's x moves: patch 36 has nothing to keep, and patch 29's walk visits every point in view, on a
phone's dense chart (HODL waves, 26 × 3,000 points, 11 a CSS pixel) most of a zoom's frame without a JIT. While the bar
spacing differs from the paint before's (a zoom's frame; the same paint drawn again keeps its first drawing's choice),
at four points or more a CSS pixel, the walk visits only each block's lowest and highest point, in order: blocks of
2^k points, the largest at most a CSS pixel wide, aligned on the series' own indices (still as a view pans); the view's
ends, part blocks, scanned point by point; its last point always; a block wider than a CSS pixel (its dates apart, a
gap in the series), each of its points. A block's two found once per data (`bvLevel`, on the
series' points, an array new with each data: its levels built in turn, each from the one below, by value, `gt`, which a
price scale maps to y in order or reversed, either way the two the same points; a value not a number never chosen
over one, in a level or a part block). Every point in view within the radius and its block's width (a CSS pixel) of the line drawn, its spikes
whole; the next paint at that spacing walks every point as before (no timer: the two the same to the eye). Under four
a CSS pixel (a block of two or three points saves nothing), as before.

- `bvLevel(t,k)` before `fe`: the 2^k blocks' lowest and highest indices (`{lo,hi}`, `Int32Array`s), kept as `t.bvMM`.
- `fe`: `bvKp` read only in a paint counted (`0!==bvPaint`); `bvG` the points a CSS pixel; `bvC` the spacing's change
  (kept in the record as `c`); with it, the indices walked put in `bvIx` (reused, `bvK.ix`), and the loop over them
  (`h` the next of `bvIx`, else `h+1`). Patch 36's join refuses such a paint's points (`!bvK.c`).

Measured on the iPhone (Lockdown Mode, no JIT), HODL waves zoomed as a pinch does, 2 s, each against the library before
it in the same sitting: median 32-36 → 20-23 ms a frame, lost 1.00-1.13 → 0.44-0.60 s; again 35-39 → 24-26 ms,
1.08-1.18 → 0.66-0.73 s. Chromium (at 1440 wide, 3 points a pixel: not taken), a zoom step's redraw 1.8-2.8 → 1.8-2.4
ms; Mullvad 21-23 ms both. A zoom's frame at a phone's size pictured against the same view at rest: the same to the eye.
`tests.html`: pictures 713 of 713 the same; a pan's lines (7) 144 checks, 0 failing; a zoom's (8) 42 checks, 0 failing
(the block's highest dropped: 30 failing).

## 40. A pinch is no tap

A touch's first finger lets go: a tap (the chart's click) unless it moved, but a pinch's moves are the pinch's, not
its first finger's (`xp` returns while one is on), so a pinch, its fingers let go, tapped where its first one was:
Studio opened the layer under it. A pinch begun marks its first finger moved: let go, it taps nothing (nor counts to a
double tap); a tap alone, as before.

- `Gp` (a pinch's start: its middle and its fingers' distance): `this.Wf=!0` first.

`tests.html`: a pinch out and in, either finger let go first, no click; a tap, one (9).

## 41. A stacked area's fill a band, from its line to the fills under it (an option)

An area fills from its line to the pane's floor. Stacked (Studio's HODL waves, 24), each band showed every band over it
too, their tints mixed (all the lower bands' blue-violet), and a fill fading with height read lighter the lower it sat.
With `bvBandFill` set (to its stack's name: any value, the same for each of a stack's areas; and the chart's
`bvHalfFills`, 30), its fill is a band: from its line back along the outline of its stack's fills drawn under it, the
pane's floor where there are none, each band its own tint alone, nothing filled twice.

- `bvBanding` (module): while a pane's fills are drawn at half resolution (`bvHalf`), the list of band fills met, not
  drawn then; after the others, in a bitmap scope of the same target, drawn from the front back (the foot first, as
  Studio orders a stack), an outline a stack (`S`, by its name).
- `Ne.et` (the fills' renderer): a band fill (`rt.bvBand`, not `Lb`, inverted) put on the list. `Ne.bvBand(t,S)`: its
  line walked as ever (`fe`, so patches 29, 36, 39 alike), drawn through a recorder that draws and keeps each vertex
  (`bvRec`, 7 numbers: x, y, curved, a curve's two controls; a new path, a new run: a color's change, each its own
  band), each run then closed by `bvBandFill`.
- `bvBandFill(c,S,col)`: a run of no width (a step's color changing at its last point: a vertical alone) none; else,
  from the run's last vertex down to the outline at its x (`at`, by halving: from the left, the
  first vertex there; from the right, the last), back along it to its first's (a curve reversed, its controls swapped;
  an end inside a segment, a straight piece to it), closed and filled; then the outline takes the run in: its vertices
  before the run, a vertical step down at its start, the run, a step at its end, the rest (two buffers, swapped).
- The area's view: `bvBand:t.bvBandFill` in its fill's data.

Not where its line is drawn twice (`invertFilledArea`) or without `bvHalfFills`: filled as before. Two stacks in one
pane (a scale's each), each its own outline.
Measured: Mullvad (headless), HODL waves, a pan step's redraw 21-24 ms as before (fills cleared and filled again, a
first try, 24-26); the iPhone, its zoom median 17 → 16 ms, its pan 19 → 20 ms a frame (each band's floor traced).
`tests.html`: pictures 713 of 713 the same; a band its own (10), straight, curved, the foot starting later, in steps
with a color changing at the last point, all in view, 9 checks, 0 failing (without the patch, 4 failing; without the
rule for a run of no width, the page hangs).

## 42. A pan's line to its end as the frame before's, where that end is the same

Patch 36 joins the frame before's points, draws them, and walks again from the last of them (its end aside) to this
frame's end: a pan with the latest point in view (Studio's usual) walked the same stretch each frame, the frame before's
last kept point to the end, the same walk on the same points, moved alike, finding again that none is kept there (a
smooth band's last stretch: hundreds of points). Where the frame before ended at this frame's end, its end is taken as
it was: the walk stops at the join's last point, the end drawn after it.

- `fe`, the join (36): its points drawn, then, the list's end this frame's (`bvO[m]===bvT-1`), the end the pending point
  (`bvPx`/`bvPy`, `bvPi`, `v`) and the walk done (`h=bvT-1`).

## 43. A line's x's written as its walk reads them

Each frame a view moved (a pan, a zoom), every line's every point in view had its x written again (`Ic`): HODL waves at
Max on the iPhone, 155,000 a frame, a quarter of it; a simplified line's walk (29) reads only the points it visits
(with 36, its first ones and those newly in view), and nothing else reads them while the pointer's away. A line's, an
area's, a baseline's x's are now kept as a mapping (the time scale's base, width and spacing, and the range asked for)
and written as they're read: by its walk, each point it visits (the formula `Ic`'s); by anything else, all of them
first (the hit tests, a line's dots, a walk not simplified).

- `bvLater(t,i,s)`: the mapping and range kept on the points (`t.bvXm`), a range grown where the mapping's the same.
  `bvXsure(t)`: every point of its range written (`Ic`'s formula), and one either side (a curve's control points
  read its neighbors, `we`: the library's own `Ic` left them as they were, a frame or more behind), the mapping let go.
- `ye.bvX(t,i)` (the series' x's: `Ic`, as before; a histogram's bars read every x) and `Pe.bvX` (`bvLater`), called
  where `OM` called `Ic` through `bvRun` (4).
- `fe`: a simplified line (patch 29's `bvZ` conditions) writes each point's x as it reads it: its first (`u`) and last,
  each visited, each joined (36), a block's span (39) by its points' bar indices; a replay reads them as written by the
  first drawing of its paint. Any other line, all of them first.
- `Rs` and the line's hit test, and the line's dots (`Me.et`): all of them first.

Measured on the iPhone (Lockdown Mode), HODL waves, 2 s, each against the library before it: panned at Max, median
37-39 → 31-33 ms a frame, lost 1.16-1.19 → 1.01-1.03 s; at 4Y, lost 0.17-0.19 → 0.13-0.16 s; zoomed, as before (lost
0.78-0.84 s both). Chromium, a pan step 1.0-1.2 → 0.9-1.1 ms, a zoom step as before; Mullvad, a pan step 22.2-23.6 →
20.3-20.6 ms, a zoom step 22.5-24.2 → 20.8-22.1 ms. `tests.html`: pictures 710 of 713 the same (the three: a curved line's ends at the view's edges, its neighbors where
they are now, the published library's a frame behind; 6 to 27 values); 7, 8 and 10 as before;
a simplified line's x's read later (11), its dots over an area panned against the published library and the pointer on
them, then a line straight and curved panned and the pointer alone on it (mid-view; a curve's at the view's end), 11
checks, 0 failing (without bringing them up to date first, the dots' 9 failing; the pointer's alone pass either way:
those x's are there to read on these lines, a case where a pan's walk skips some of those near the pointer is Codex's,
reproduced outside the page). Codex's review: a curve's neighbors, outside the range (fixed).

## 44. A pan's line from its start as the frame before's, where that start is the same

Patch 36 joins the frame before's points at the first point both walks keep; till then this frame walks. With a
series' first point in view (Studio's Max: all of it), both walks start there, on the same points moved alike: the
same walk, which found the frame before's first kept point where this one finds it (HODL waves' early years, flat:
1,500 points each band before it, 35,000 a frame). Where the frame before started at this frame's start, its points
are taken from the start: drawn and kept, then the end as 42 has it (the same end, nothing walked at all), else the
walk from the last of them on.

- `fe`: the frame before's start kept as its record's is replaced (`bvOf`); before the walk, the list taken from the
  start (each point's x written, 43), the end pending where it's the same, else the walk set to start at the last of
  them (`bvH`, its anchor `bvAx`/`bvAy`), `bvO` let go.

Measured on the iPhone (Lockdown Mode), HODL waves at Max panned 2 s: median 31-32 → 21 ms a frame, lost 1.02-1.04 →
0.54 s; at 4Y as before (its first point out of view). Chromium and Mullvad as before (their bench's view at 8.5 years).
`tests.html`: a pan's line (7) with the whole series in view too (its first and last point there each step), 288
checks, 0 failing.

## 45. A thin simplified line's joins cut, not rounded

A line's segments meet in round joins: a CPU canvas (Firefox's, Mullvad's) rasterizes each round join's arc, most of a
thin line's stroke where its segments are short (a simplified line's: one every pixel or two). At 2 px or less, a cut
join (`bevel`) differs from a round one by at most half the width at a sharp turn: the same to the eye (HODL waves at
2x: a mean difference of 0.2 of 255 a pixel, its spikes alike). A simplified line (29) 2 px wide or less is drawn with
bevel joins; wider, or not simplified, round as before.

- `Me.et` (a line's renderer): `lineJoin` `bevel` where the points are simplified (`i.bvS`) and its width (`h`) is 2
  or less.

Measured, Mullvad (headless, 2x): HODL waves, a pan step 20.3-20.6 → 18.1 ms, a zoom step 21.3-22.1 → 18.8-19.6;
"Realized price by age band" (24 lines), a pan step 7.5-8.1 → 6.4-6.9, a zoom step 9.2 → 8.3. Chromium as before.
`tests.html` as before (its pictures not simplified; 7 to 11 by distance).

## 46. A series' options set again, its points restyled only where their style changes

Every `applyOptions` on a series restyled each of its points (`Pt("options")`: `BM`, its colorer asked for each point,
the style assigned onto it), whatever the options were: Studio's axis tags shown again for 25 lines as a zoom takes their
names' room (`lastValueVisible`), 25 × 5,000 points restyled in one frame. Now the style the options give a point of no
colors of its own, rising and falling (a probe row each: a bar's, a candle's up and down colors; a baseline's top and
bottom), is compared before and after: the same, the series drawn again (`"look"`: its views updated, `CM`), its points
not restyled; else as before.

- The series (`vr`, its `applyOptions`): `bvLook()`, its colorer's (`Sa().Mh`) styles for the two probe rows (values
  ±1e308 from open to close) its `bvSimplify` (the restyle sets the points' `bvS`, 29) and its `baseValue` (a baseline's points' side of it), as
  JSON; before and after
  the options are merged, `Pt("look")` where they're the same.

Measured on the iPhone (Lockdown Mode): 25 lines' axis tags switched and drawn, 115-124 → 39-53 ms. `tests.html`:
pictures 710 of 713 the same as before (restyles in every series type, mid-pan too: never restyling, 576 of 713).
Codex's review: a baseline's base moved, its points' sides changed with no probe seeing it (its base now compared).

## 47. Marks' words edged in a color of the chart's

A series marker's text (`createSeriesMarkers`, a mark's `text`) is filled alone: over a line or the data it's crossed by
them (Studio's signals' results, "+46.27%", read through the price line). A chart option, `layout.bvHalo` (a color,
none by default), now edges each mark's words in it first (a 3 px stroke, round joins, under the fill), as Studio edges
its legends' words in its card's color: the lines under them stop short of them. None set, as published.

- The markers' view (`Or.renderer`): the chart's `layout.bvHalo` handed to its renderer (`Er`), each frame (a theme's
  change, applied to the chart, in its next).
- The marker drawn (`Br`): its text stroked in it before it's filled, where one is given.

`tests.html` 12: a mark's words on a line, edged in blue (blue about them), and without a halo, none.

## 48. A pan moves the picture, only what comes into view drawn (an option)

A pan redrew each pane whole: on the iPhone (no JIT) HODL waves' stack (24 bands) took ~8.5 ms a frame to draw, in
Mullvad (canvases on the CPU) a pan step ~25 ms. With the chart's `bvPanShift: true`, a frame whose only change since
the pane's last is the time scale's offset (the same pane, bitmap size, bar spacing, width, every price scale's mapping
and the first value it's read from, the series hovered and what it was hit at, the last bar in view where a series'
price line reads it, no option, data, price line, order or primitive changed; never where something can't move with the
picture: a relative gradient (it spans the view), a curved line (it bows past its points), a dashed line (its dashes
from where its path starts, or from the pane's edge: a series', a price line, the last price's, the base's, the grid's),
a title or a percent at the pane's edge) moves the pane's main canvas by the offset's whole device pixels (its own
pixels, drawn on it as a copy) and draws only the strip that came into view: under a clip, the pane as before
(background, grid, views, fills), each series' points those in the strip alone, and those as far either side as the
widest a series draws past its point (6 CSS pixels at least: joins, the half-resolution fills' sampling; half a bar, a
point marker's radius, a line's width). It then asks for one more frame: the frame after the motion stops draws the pane
whole, so at rest the picture is exactly a full draw's. Mid-motion the moved part can be up to half a device pixel from
where a full draw puts it (the offset's rest, carried to the next frame) and antialiased a little apart (half-resolution
fills on another grid); what's kept inside the plot's edge by a primitive (a label pinned to it) moves with the picture
till the frame at rest.

- `bvStrip` (module level): the strip being drawn, its device pixels (`a`, `b`) and its logical range (`Na`, `bi`), as a
  visible range reads; null otherwise (set aside and put back around a pane's paint: one chart drawn inside another's
  doesn't inherit it). `bvEdits`: a count of edits that change a picture without moving it, raised by a series' views
  told of new data or options (`Es.Pt` with a reason), a primitive attached, detached or asking for an update (both
  `attachPrimitive`s' `requestUpdate`), a price line made, removed or set (`createPriceLine`, `removePriceLine`, `ee`'s
  `applyOptions`), a series' order (`setSeriesOrder`), the chart's options (`vr`), the time scale's and a price
  scale's (`applyOptions`).
- `Es` (a series' pane view): its visible range (`TM`) read from `bvStrip` where one is set (`LM`), and made again once
  the strip's gone (`IM`: `bvSt`, the strip it was made for).
- The pane widget: `bvMain(t)`, its main canvas's paint (in `Rm`); `bvFixed(t)`, what can't move; `bvShift(t)`, with the
  option, the key above (`bvAt.k`) and where its picture shows (`bvAt.c`, in bars: moved by whole pixels, the rest
  carried), the canvas moved (`drawImage` of itself, `copy`), the next frame asked for (`ka`), the strip returned;
  `bvStripOf`, its logical range; `bvHalf` (30): only the strip's part of its half-resolution canvas cleared (`bvDirty`
  kept: what's outside is old).

`tests.html` 13: an area and a line panned by whole and part pixels, both ways, their scale held: strips drawn, and once
still the published library's picture.

## 49. A series' last-value tag made again only once what it reads changes

A pan invalidates every view, and each series' last-value tag (`ut`) was made again each time it was read, many times a
frame: the last bar in view found and its value formatted three ways (price, percent, indexed), shown or not. HODL
waves' 23 series spent ~3.3 ms a frame on the iPhone. Now a tag is made again only once something's been asked of the
chart since (`bvAsked`, raised by every invalidation, `kd`) and, then, once what it reads differs: `bvEdits` (48), the
last bar in view (its index and its values), the price scale's mapping (`bvMap`, with the first value it's read from),
the series its scale formats by, the first value in view where the tag shows a percentage.

- `ut.Yi`: `bvA` (the `bvAsked` it was made at), `bvK` (what it read).

`tests.html` 14: three series' tags, a view off their end panned so its last bar changes: each step, the price scale
as drawn once every tag's made again.

## 50. A series' tags under the legend, the axis as wide all through a pan

A price scale's `bvTagsTop` keeps its tags under that height (Studio's legend over the pane). A price scale's
`bvTagsWidth` is the widest its series' tags can be written (Studio's, from each series' values): the axis at least as
wide all through a
pan, not resized each frame by the values passing (a tag reads the last bar in view; across a stack's 23 bands the
widest changed every few frames, the plot jumping 8 px); a tag wider than that (one Studio's sample of the values
missed) widens it once, as wide as any has been since it was given. A tag's pane side (a title) is drawn on the pane's
top canvas alone, not under it too: it stays where it is as the picture moves (48), so titles no longer keep a pane
from moving. (Its tag once followed its series' end into the plot, `bvAtEnd`, named, then not: it clashed with
drawings and stacked where values are close; gone, the tags on the axis alone. And its place and words once came from
the series, `bvTagOf`, a stacked band's tag at its middle with its own value: gone, a tag reads the axis where it sits,
a stacked band's its sum there, as any line's.)

- `bvAbove(t,i)`: the top views but a tag's pane side (`st`), in `bvMain`'s main canvas paint; `bvInside`: its upward
  pass from `bvTagsTop`; `Cm`, the axis's width: at least `bvTagsWidth`, and its series' tags' (`ut`) widest since
  (`bvSeen`, again from none when it's given anew or the axis shows another scale).

## 51. Whether a point is on a series' tag

Studio lights a layer's legend item as the pointer's on its line (the chart's hovered series), but not on its tag: on
the axis, the chart says nothing (the crosshair's off the plot). Now each tag remembers where it was last drawn, and a
series says whether a point is on its tag.

- `L.st`: `bvHit` on its view (`qt`), its canvas and its box in that canvas's CSS pixels, as drawn; none while it's
  not drawn.
- `bvTagHit(x, y)` on a series (the API's, beside `priceToCoordinate`): a point in the client's coordinates on either
  of its last value's tags (`ut`'s `zi`, the axis's; `Li`, the pane's), as last drawn and as far as its canvas shows
  it (one spread past its pane's foot, not over the next); false while hidden.

`tests.html` 15: a series' tag's middle on its tag, a corner of the pane not.

## 52. A custom series drawn to the bar past each edge of the view, as a line is

A line, an area and a baseline are drawn from the bar before the view's first to the bar after its last (their
segments run on to the edges); a custom series was given the bars in view alone, so a fill drawn by its points (Studio's
clouds) stopped at its last bar in view while its lines ran on to the edge, up to a bar short of it (zoomed in, a
bar's spacing). Now a custom series' renderer is given the same range as a line's, the canvas clipping what's past.

- `SeriesCustomPaneView`'s `super(t,i,!1)` (the base view's `extendedVisibleRange`): `!0`.

## 53. A series outlined in the chart's background (an option)

A line over a heatmap, a cloud or another line is crossed by what's under it: read through, it's lost in it. A series
option, `bvOutline`, now edges what the series draws in the chart's `layout.bvHalo` (47), drawn under it, as a map's
road is cased: where two lines cross, the one on top cuts a clean gap in the one under; over a fill it stands off it.
Unset, or no halo set, nothing is drawn: as published.

- The series' view (`Es.Tt`, each frame): its renderer handed the color (`bvO`, the halo where the series asks for
  one; a theme's change, applied to the chart, in its next frame); an area's and a baseline's, their line's (`bvL`).
- A line's renderer (`Me.et`: a line's, an area's, a baseline's): the line's path stroked first in it, three times as
  wide (the line's width each side: half of it read as too thin), round joins, dashed as the line is (its dashes set by
  its own width, a dashed band is dashed with it), none where a point hides the line (`transparent`: a trend's flips);
  the line's own stroke after it walks the points kept by the first (29). Point markers (Studio's dots): each ringed,
  the line's width wider (1 px at least), its dots drawn by `bvDots` (the renderer's own, named).
- A histogram's (`sr.et`) and candles' (`tr.bvHalo`): 1.5 px (a device pixel at least) about each bar, each candle's
  body and wick (to its last pixel, as drawn), one fill under them all (their corners' radii as much more).

`tests.html` 16: a line over a band of another color, outlined (the halo's color beside the line), and not (none).

## 54. A series drawn fixed in the view (an option)

A pan (48) moves a pane's picture with the time scale and draws only the strip that comes into view: right for what's
drawn at its bars, wrong for what a series draws fixed in the view (Studio's heatmap profile, bars from the plot's right
edge), moved with the picture and cut at the strip. A series option, `bvFixed`, now counts among what a pan can't move
(`bvFixed`): its pane drawn whole, as with a relative gradient. Unset: as published (and as 48).

## 55. A series updated in place: its pane alone drawn again

A series' data applied (`Mb`, the data layer's answer) drew every pane again, whatever changed: the series' own change
(`ht`) asked for its pane's scales (`Ea`, the pane's `ou`, which ends asking for every pane: `mr()`), its pane (`Ta`,
whose mask, `Nd`, also sets every pane's level), the crosshair (`Ra`) and every pane (`mr()`); then `Mb` asked for every
pane's scales and every pane (`zc`). When the answer carries no time scale points (`SM` undefined: no time added or
taken, only that series changed) and the same base index, nothing another pane draws can change: its data, its scales,
the time scale and its marks. (But with `ignoreWhitespaceIndices`: a point filled in where only whitespace was is a
new mark, every pane's grid line.) Studio's live price moves the price's last point each second it changes: a stacked
chart's 23 bands, in the pane below, were drawn again with it.

- `Mb`: `bvPane`, `SM` undefined, the answer's base index (`Dc`) the time scale's (`Bt().Dc()`) and the time scale's
  `ignoreWhitespaceIndices` off; passed to each series' `ht`, then `bvZcIn(series)` instead of `zc()`.
- A series' `ht(t, i, bvPane)`: with it, none of its four asks (`bvZcIn` asks for its pane).
- The model's `bvZcIn(series)`: each series' pane's scales again (the pane's `ou(!0)`, quiet: without its `mr()`), a mask
  with those panes alone at the light level (`new Y(0)`, each `{rs:2}`), then `Ra()`, as `zc` does.
- The time axis drawn at the most any pane is asked for (the mask's `bvMost()`, in the widget's `Rm`), not the mask's
  own level: as published whenever a pane is drawn (its `Nd` masks set both), and for a pane asked for alone, a
  primitive's time axis views drawn again with it.
- Unchanged: an axis that the update widens lays every pane out again as published, the update drawn first at the old
  width, the layout a frame on (traced frame by frame, both libraries: the same canvases' sizes and pixels each frame).

`tests.html`: a picture is taken with the crosshair hidden, an option set, and setting one draws every pane again first:
a pane left undrawn never showed. Scenarios now say whether theirs are taken as drawn (`asDrawn`: the canvases as they
are, the crosshair's layer left out); not all, as a pane drawn again isn't always the same to the pixel (a candle's
edges, a shade apart). As drawn: the two-pane candles scenarios, ending with updates in place (one widening the axis
both panes share, on a linear scale; one in the pane below under the crosshair; the axis narrow again), and points
filled in on whitespace left out of the marks (798 values apart without the `ignoreWhitespaceIndices` check). Measured,
40 updates of the price's last point on the 23-band stack, drawn at once (`bvFlush`): Chromium at 4x CPU 7.9 → 2.4 ms
an update; Mullvad Browser (headless, 2x) 11-12 → ~1.8 ms; the iPhone without its JIT ~19 → ~7 ms.

## 56. A pan by under a pixel: the picture kept

A pan's frame (48) whose offset, carried, rounds to no whole device pixel drew its pane whole: the same rule (`0===o`)
told the frame after the motion stops, at rest, to draw it whole. A slow pan, or one over bars under a pixel apart,
moves less than a pixel a frame: Studio's 100% stack, 3 bars a frame at 0.26 px a bar, every third frame drawn whole
(~60 ms in Mullvad Browser). Now the two are told apart by whether the time scale moved since the pane's last frame.

- `bvShift`: `bvAt.h`, the time scale's offset (`Dc()+Oc()`) at the pane's last frame. The same: at rest, null (drawn
  whole, as before). Moved by under a pixel: its picture kept (`bvAt.c` where it was, the rest carried), the next frame
  asked for (`ka`, as after a strip: a stop is then drawn whole), and an empty strip returned (`[0,0]`).
- `bvMain`: an empty strip, nothing drawn.
- A picture (`Tg`, `takeScreenshot`) takes what's waiting before drawing it, as `bvFlush` does: drawn while still
  waiting, a frame asked for meanwhile (the next frame after a strip (48) or a picture kept) merged into it and was
  dropped with it, the pane left mid-motion till something else asked for it.

`tests.html` 13 compares its pictures once still as drawn (patch 55's `asDrawn`; before, a picture hid the crosshair,
drawing every pane again first), and pans by under a device pixel too: frames keeping the picture counted (none
before), the picture once still the published library's; and once with a picture taken as drawn after a step, before
its frame (4071 values apart once still without `Tg`'s change). Measured, a pan of 3 bars a step drawn at once (`bvFlush`): Mullvad
Browser, the 100% stack 21-22 → ~3 ms a step, the 23-band stack 8-10 → ~3.5 ms; the iPhone (60 frames by rAF), the
100% stack p50 24-32 → 16 ms, p90 50 → 18-21 ms.

## 57. A series' data as columns (`bvSetColumns`)

A series' data came as rows alone: an object a value (`{ time, value }`), each one's time converted (the library's form
and its key), made into the library's own row and put on its time's point. A page holding its data as columns (Studio:
one times array a chart, a `Float64Array` of values a series) made ~156,000 objects to open a 23-band stack, and the
library converted the same times again for every band. `series.bvSetColumns(times, values)` takes them as they are, for
a series of values alone (Line, Area, Baseline, Histogram; another, or lengths that differ, an error): a finite value a
row, any other (NaN, an infinity) none (whitespace), as the same rows with no colors of their own set by `setData`
(`{ time, value }`, `{ time }`) are. A times array's conversion is made once (held on the array, weakly: an array once
given is taken as never changing) and shared by every series set on it. Set again on the same array as its last columns, nothing else set between (`setData`,
`update`, `pop`, `bvPrepend` let it go), a series' rows aren't taken off every point first: each of its points gets its
new row in place. One difference with `setData` there, as with `bvPrepend` (9): a point's `originalTime` stays the form
of the first series holding it (`setData` would put the series after the others); they differ only when series give the
same moment in different forms (a business day or a date string in one, a timestamp in another).

- `class ms` (the data layer): `bvColumns(t, i, v)`, beside `oM` (setData), and `bvTail`, setData's end shared by both
  (the rows put on the series, points no series holds dropped, the time scale made again if points were made or
  dropped); `bvConverted(i)`, a times array's conversion (`bvConv`); `bvTimesOf`, each series' times array as last set
  as columns, let go of by `oM`, `pM` (update), `wM` (pop), `bvPrepend` and a reset (`m()`).
- The chart (`JM`'s class): `bvJMc(t, i, n)`, as `JM` hands its result on.
- The series' API (`setData`'s class): `bvSetColumns(times, values)`, as `setData` (a full data change to its listeners).

`tests.html`: "data as columns" (as published, the same rows set by `setData`): a line, an area, a histogram and a
baseline on one times array (gaps, every 37th value none, an infinity); one set again on it with other values, pans; one
on another array; a live update, then the same array again (the update's point let go of: kept, a view asked to end at
its time ended a day further, 32,566 values apart without `pM`'s letting go); emptied, and back. Measured, an interval
opened again on Studio's age bands' 100% stack (23 bands): the iPhone without its JIT, setData's samples 215 → 129
(`bvSetColumns` 88, `setData` 41 for the layers Studio still gives rows: candles, a close, colored bars, breaks, a fine
interval's), the chart's build 388 → 334; Chromium at 4x CPU, the library's intake 53-54 → 44-56 ms (a JIT makes rows
cheap: no gain there, no loss).

## Typings

`dist/typings.d.ts` declares what the patches add to the API (the chart's `bvFlush`; a scale's `bvRange`, `bvSetRange`,
`bvAutoRange`; a series' `bvPrepend`, `bvTagHit`, `bvSetColumns`; the options `bvHalfFills`, `bvPanShift`, `layout.bvHalo`,
`minVisibleDataRatio`, `bvHoldOnAxes`, `bvTagsTop`, `bvTagsWidth`, `bvSimplify`, `bvBandFill`, `bvOutline`, a candlestick's and a histogram's
`cornerRadius`), in one block at its end, merged into the
library's own interfaces: a fresh download's typings take it again as it is.
