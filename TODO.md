# TODO

Only what's left. An item done is removed: its history is in the commits and logs.

## Waiting on you

- Test Studio on a real phone by hand: its touch gestures (a finger's pan, pinch, fling) can't be driven from here.
  Measured on the iPhone (2026-10-29, Lockdown), the view moved by script a bar or more a frame: 60 fps at p90 on
  Sats per Dollar (2012–13, 2011–15, the latest bar) and the Bubbles chart (8Y, Max), 2-4 frames over 20 ms in 120.
  - Scrolling, on Sats per Dollar:
    - pinch and fling across 2011–2015, where the axis labels change width most;
    - do the same at the latest bar;
    - turn the phone and back.

- Right after a `fleet update` (2026-11-01, 232107e), live Bitcoin price stayed on "Loading 1 series" for over 40 s in
  Mullvad and Chromium, then loaded (later: 0.37 s; the API 30–60 ms). Not the server: locally, straight after a
  restart, the price answers in 2 ms and the preset opens in 0.54 s. Most likely a request caught on a server
  restarting behind the tunnel (fleet restarts them one at a time) hanging till the client's 30 s timeout
  (modules/bitview-client/index.js ~2164, the download included), then retried. To confirm during the next deploy
  (watch a page's requests as each server restarts); then either fleet drains a server's tunnel before stopping it,
  or the client gives a series request less time to start answering (not to finish).

- Next deploy (you, 2026-11-02): one, once the chip rule and Studio's heatmaps/profiles/stacks on the library are
  built and checked, with the line under the pointer (c34e6dd14); watched for the stall above.

## bitview-charts: raised by you, still open

- OKLCH throughout (2026-10-31): the library worked in OKLCH (its color math, tints and fades, from OKLCH colors), and
  every color in the app written and derived in OKLCH; contrast judged by APCA (the best perceptual formula there is:
  WCAG 3 has left its own undecided; CSS's contrast-color() uses WCAG 2's). Done in the library: the axis tags (tint()), heatmap gradients (gradientTable()), fades (Fill's stops); left:
  Studio's own colors written in OKLCH (its palettes, CSS tokens: ~30 hex and ~80 rgb() left, 2026-11-01).

## In progress (my own work, picked up from here)

- bitview-charts, Codex's review of the whole library (2026-11-01): done in two commits (b8f0aa256, then the
  second: Bars.distance gone, one firstFrom, stale comments).
- Studio's chart code, Codex's review (2026-11-01; full report: scratchpad codex/studio1/out). To fix, in batches
  (each: fresh test, type checks, 79-preset sweep, phone sweep, Codex on the diff, commit). Lines as of ac6dbc585:
  - Batch 1 committed (804a3cc8d); batch 2 committed after it ((3) every LWC geometry place, previewShown's
    ±0.5 for saved views; (4) a pinned crosshair put back after a live tick; (5) the close line colored candle by
    candle). Left: batch 3 (per-frame work, below), then going live.
  - Batch 1 as made: (1), (2), (7) (Shade `behind` by default in the library), (9), (10),
    (14); (8) kept, its comment fixed (a histogram's below-base palette is made with its data: a color edit needs
    it). Type checks clean, 79 presets 0 errors; Codex on the diff (scratchpad codex/studio2/out); then phone, commit.
  - Batch 1 as found: (1) stacksMade's key (~23502) lacks layer type and `from`: a layer made bubbles on
    another stays in the stack; (2) the timeline trim (~16159) slices every column but `closes` (heatmaps' closes off
    by `first` points); (7) Shading's Shade not `behind` (~18523): it can paint over series; (8) histograms excluded
    from kept data (~17232, ~17065: an LWC-era rule; Bars has columns now); (9) candles' `dated` unreachable (~17122,
    ~17203, ~16761); (10) LWC comments (~16319 Chart.#sync, ~17149 shading overlay, ~22275 field merge); (14)
    updateValues called twice (~17473 with updateLegends ~17910), dashFor (~18914) and dashesOf (~20443) the same
    five dashes.
  - Batch 2 plan for (3), read 2026-11-01: the library puts point i at (i - from) * spacing (LWC: (i - from + 0.5),
    a range one bar longer). Fix four: presetWindow (~17582: to = n-1 + gap/spacing, from = to - width/spacing; same
    intent, first point's middle `room` in, last's `gap` from the end), lastShown (~17667: Math.ceil(to), the
    library's tags' rule), firstInView (~17673: Math.floor(from), the library's #from), the span glide (~36085:
    first - 0.5 to last + 0.5, whole bars). previewShown's ±0.5 (~24115) is already right for the library.
  - Batch 2, to look at closely: (3) Studio's range maths (~17583, ~17670) use LWC's bar geometry (n-1, half a bar),
    the library's Timeline doesn't: lead and gap off by a few px, the resting legend reading point 10 where tags read
    11; (4) a live tick doesn't re-anchor a pinned crosshair after the scale refits (~39826: holdCross); (5) candles'
    close line tinted by the view's last candle (~16764), wrong when the tag reads the crosshair's candle; (6) bandAt
    (~17817) can override the library's hit with a nearer value at the rounded point: fixed (2026-11-02), each line's
    stroke measured against its drawing (Pane.distanceOf).
  - Batch 3 measured first (2026-11-01), left as is: a crosshair move costs 0.045 ms median in Chromium over 48
    presets (worst the profiles: Supply profile by age band 0.31, by holder group 0.24), on the phone 0.2 ms (the
    price) to 0.87 (the 24-band stack): what (15)–(17) could save is a part of that, a few percent of a frame
    (scratchpad n437/cross.mjs, phone-cross.mjs; cross-before.txt).
  - Batch 3 as found, per-frame work: (15) every render marks each Sketch stale (~20990 viewed): crosshair moves recompute
    drawings, trends' gradient colors (~21012); (16) Heat `fixed` (~18787) redraws its pane at every crosshair move,
    profile bins rebuilt each time (~22652); (17) profileShare summed twice a move (~22356, ~22598) though its day is
    the view's last.
  - Not to do now: Cloud, Heat, stackOf duplicate parts of the library's Band, Grid/Profile, stack (11–13): the
    question above (one should go).
- Studio's legend and layers-panel code, Codex's review (2026-11-01; full: scratchpad codex/legend/out).
  Committed (d1917fa27): the two leftovers gone, (2) a picture clears the spotlight, (3) a picture writes a value's
  as-of date. (1) committed after (focus restored after Legend.set). (5) committed after (every layer hidden: the
  legend wraps). A percent stack's readout reading the stacks once (not once a band; bandAt only for a stacked pane):
  committed. (4) committed (2026-11-02): the chip names the layer pointed at (its line or band, its axis tag), kept
  in its pane's empty space and on the chip. Left: the rest of the repeated work (measure first). As found:
  - Bugs: (1) a legend rebuilt loses keyboard focus (~17893 innerHTML then restoreFocus, then Legend.set appends
    every element again as the count element was dropped; the focus key carries data-pointed / data-lit, ~23578);
    (2) a picture keeps the spotlight's faint series and levels (export clears the legend's classes only, ~38492;
    fadeLayer ~16701); (3) a picture drops a resting value's as-of date (CSS content attr(data-as-of) ~6000, the
    picture reads textContent ~38464); (4) on a named chip, moving onto it loses its caption, and hovering an axis
    tag can't name it (the library names from crosshair.pointed/hit, cleared over page elements: index.js ~652, ~496;
    lightTag ~19257 lights classes only); (5) a folded pane's legend with all hidden lists every item on one clipped
    line (~2998 height 22px overflow hidden, ~17528).
  - Leftovers: the "pane-layers" click branch (~39264) has no producer; profile captions call fitSoon on legends
    (~17856), whose strip classes legends no longer have.
  - Repeated work (measure first): profileShare per crosshair move (~22350, ~22553); a percent stack's readout
    builds stackOf's key once a band (~18235, ~17832; bandAt calls stackOf before checking the pane is stacked,
    ~17807); figuresRoom each move (~19638); readouts found and written for hidden legends (~18207).
- Left from that review, not started: a transparent `under` half of a line still hit (recorder keeps a gradient's
  whole stroke: strokes would need the clip, line.js ~596, recorder.js ~186); marks ignore their host's overlay scale
  (marks.js ~145: Studio has none); tiny ranges (1e-310) overflow the zoomed-out slope in line.js ~430 and band.js
  ~301 (scale.js divides first); unused re-exports (kept: the library's API); the curve control points' one line (line, band: left).
- Decided (2026-11-02): Studio onto the library's Grid, Profile and stack. The stack done (committed after: the
  library's stack makes tops, feet and scale; Studio's stackOf calls it; every stacked preset's columns identical).
  Heatmaps onto Grid committed (2026-11-02). Profiles onto Profile committed after. Next: the cumulative
  view (mockups first). Also to look at: Studio's own gradientTable (~30978) beside the library's
  (color.js).
- Phone (connected 2026-11-01): after b8f0aa256, 48 presets 0 errors, 40 with no slow frame in 60 while panning, 8
  with one or two (scratchpad n437/phone-fix.txt; worst Supply by profit band 51 ms, Supply by age band 49). Probed
  (n437/phone-probe.mjs, each frame's gap and whole draws): the long ones are a single 56–61 ms frame with nothing
  drawn (most likely a WebKit full collection: ~1 a 60-frame pan now, 5–6 in 120 with Lightweight Charts), and the
  scale's refit at new extremes (a whole draw, 20–37 ms on the 24-band stack). Left as is unless a heap cut shows up.

## Studio: raised by you, still open

- Deploying (2026-11-02, you: "deploy once it's all checked"): next to prod, `fleet update`, then checked on Chromium,
  Mullvad and the phone, watched for the post-deploy stall (Bitcoin price loading 40+ s: a request caught by a
  restarting server?).
- The gradient editor and a heatmap's default colors (you, 2026-10-28: "think very hard, how would best designers in
  the world do it", "look for the best setup possible"): built, then reorganized (2026-10-29: Reverse and Add on the
  ready-made gradient's line, the color in hand under the bar heading its picker, its worth under its knob, the scale's
  values measured apart, the nearest color grabbed; _notes/overnight-20261029/gradient-editor-before-after.png). Your
  look to confirm.

- Bubbles: built, a kind of their own beside clouds, trends and signals (Add bubbles, a layer's ⋯ › As bubbles on…; not
  a series' Type), On, Size, Hide below, Sized to, one color or two; presets per cohort (Profit and loss bubbles, and on
  the unrealized P&L heatmap); a chart's mark shows them as dots; the minis (tiles, ⌘K's previews, Quick Look) read
  the chart's own days, each point its span's largest (_notes/overnight-20261029/bubbles-mini-days-before-after.png).
  Open: your look at the minis now.
- Profile, a layer of its own (2026-10-29, 31734f9a8): bars as tall as their rows, solid (97f69ee60); on the view's last
  day (your call, as a visible range's profile), its key in the legend and a bar pointed at read there, its group lit
  (your pick, A and the key: _notes/overnight-20261029/profile-point-A-legend.png); its groups all deselectable; in the
  minis too (tiles, ⌘K's previews, Quick Look: its real last day, room left after the line). Open: your look on a phone.
  Presets to try it: each group's Cost basis › Profile, Holders › Compare › Profile, Coin age › Age bands › Compare ›
  Profile; one palette for the holder groups everywhere (palette.js, 6c01ddb92).
- Next, asked: one system for every size, width, height, margin and padding (see the design-system item); then the
  right panel and the legends reviewed (the least worked on).
- A design system for all of Studio, then a polish and consistency pass applying it (you, 2026-10-28: "I think we
  really need to do a polish/consistency pass over the whole app, and make a proper system"): inventory
  (_notes/system-20261028/inventory.md: 9 font sizes, 20 radii, 22 gaps, 63 heights, 47 shadows, 56 transitions),
  then the system written down for your check (_notes/system-20261028/system.md; corners inside corners concentric,
  unless it makes no sense, you: "all radii inside radii follow the universal radii rule"); the pass applied area by
  area (2026-10-29: sidebar to dialogs and a phone's touch targets, see the system's notes); titles 15 (2026-10-29: from 18, you), ⌘K's rows 14,
  their asides 14, its preview titles 14 semibold: fine for now, to tweak later. Body kept at 13 (you: "I think I agree for
  13").

## Studio on the new server

Served by the server at DOMAIN/studio/ (bitview.space, next.bitview.space; studio.bitview.space redirects there).

- Deploy the MCP page only once `main` has `showcases/assets/fonts` (its fonts load from there through jsDelivr).
- Alerts: being told when a series crosses a level (push, email or a webhook). Only the server can watch for it and
  deliver it (alerts that fire only while Studio's tab is open are a weak promise): a back-end feature, with its own
  design, before Studio can offer it.

## Showcases

The website is gone: the server serves the showcases, Bitview's front page (`showcases/index.html`) at `/`.

- A home for what only the website had: block, transaction and address pages.

## Back end

- Fill v1 block extras' `orphans` from the node's `getchaintips` (the non-active tips at the block's height, with their
  `prevhash`), as mempool.space does; it's always empty now. Additive, so after the release.

- Esplora routes mempool.space serves and Bitview doesn't (API.md), each only if it answers as fast as the address
  routes (no rate limit here, unlike mempool.space): `/api/scripthash/{hash}` with `/txs`,
  `/txs/chain[/{last_seen_txid}]`, `/txs/mempool` and `/utxo` (the address routes keyed by the Electrum script hash,
  SHA-256 of the script: needs a new script-hash index, as `addr_hash` isn't that hash), `/api/address-prefix/{prefix}`
  (up to 10 addresses by their text: needs a text-prefix index, as electrs's optional `--address-search`), and
  `POST /api/txs/package` (Bitcoin Core 28+ `submitpackage`, up to 25 transactions; cheap here, under `POST /api/tx`'s
  pending limit). `/fee-estimates` (deprecated upstream) and `/block-template` (not on mempool.space) stay out.

- More mempool/electrs routes live on mempool.space (rbitcoin serves them too), each only if fast enough unthrottled:
  `GET /api/txs/outspends?txids=` (several transactions' outspends), `/api/mempool/txids/page[/{last_txid}]`,
  `POST /api/txs/test` (Core's `testmempoolaccept`, under `POST /api/tx`'s pending limit) and `GET /api/broadcast?tx=`
  (a GET twin of `POST /api/tx`). Additive, so after the release.

- `GET /api/v1/mining/blocks/fees?from=&to=`, mempool.space's block fees over a timestamp range: the period routes'
  grouping, its divisor from the span (24 h past 4 years down to 1 s within a day, `getTimeRangeFromTimespan`).
  Additive, so after the release.

- mempool's electrs address extensions, live on mempool.space, each only if fast enough unthrottled:
  `/api/address/{address}/txs/summary[/{last_seen_txid}]` (up to 5,000 rows `{txid, height, value, time,
  tx_position}`, `max_txs` at most that), `POST /api/addresses/txs` and `POST /api/addresses/txs/summary` (several
  addresses' histories merged, body capped).

- Series additions and precision fixes found by the 2026-10-24 review (`_notes/tree-review-20261024/FULL-REPORT.md`,
  sections 2 and 4), all additive: raw joins (`tx_height`, `output_tx_index`), pool fee totals, unspent coinbase
  supply, change-adjusted transfer volume, spent-output lifespan (ASOL, MSOL), UTXO and address counts in profit and
  loss, UTXO-set age statistics, address concentration, USD balance thresholds, a spent-output price distribution;
  coin days destroyed summed in satoshi-seconds before converting, and the awake/mobile distributions' per-bucket
  rounding and grid-price dollar weights.
