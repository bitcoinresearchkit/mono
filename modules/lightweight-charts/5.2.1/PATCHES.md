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
its pane (11), an indexed scale's labels are its series' (19). A view held on screen as a right scale's width changes, an option (20). A view keeps a share of it on its data, an option (13). The rest (2–5, 8, 9, 14, 22, 23) draw the same, faster; a tag hides the mark labels under it (15). `tests.html` runs the patched file and 5.2.1 as published side by side through every
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

## 35. A tag's words in whichever reads more on it

A price axis's tag (a series' last value, its price line's, the crosshair's) writes its words black or white by the
tag's grey (`0.199 r + 0.687 g + 0.114 b` over 160: black): a mid green, an orange, a blue, a violet took white at 2.7
to 4:1 (Studio's MVRV green 2.7). Now the one of the two with the higher contrast, as WCAG measures it (the tag's
relative luminance against white's and black's): 4.5:1 or more on any color the axis shows but the mid greys.

- `generateContrastColors` (minified: the method returning `{Z:` the background, `X:` the foreground `}`): `X:` the
  luminance's pick instead of `S(i)>160?"black":"white"`.

Studio's own tags (a drawing's on the axis, a solid text's ground) pick alike (`textOn`).

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
