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

- Decisions from the nights' work (2026-10-30 to 11-01; each with its numbers in
  _notes/overnight-20261030/DECISIONS.md), mine noted:
  - Charts saved with lag(…) now say "unknown function lag" (shift replaced it): Studio could rewrite lag( to shift(
    in stored formulas as it reads them (one line). Mine: yes.
  - Opening Studio at a preset's address: its data is asked for only once the presets are read (~80 ms after them in
    Chromium, ~130 in Mullvad, more on the phone). Option: the last preset opened kept with its chart, its data
    fetched at once, checked once the presets are read. It changes how Studio starts.
  - Mullvad, the layers panel toggled on the 24-band stack: one frame of 50–67 ms at each toggle (2026-11-01; was
    83–133 plus one of 50–67 with Lightweight Charts), the chart redrawn at its new width at the motion's end. Drawn
    at its start instead, the long frame comes before the panel moves: a motion choice.
  - Mullvad, the 24-band stack panned: ~20 ms a step (a whole draw each, the scale refitting: decision 12, as now),
    ~12 of it the 24 lines' strokes, rasterized by Firefox's CPU canvas (~0.9 µs a segment at 1.5 px; their paths'
    making is near nothing). Lines one device pixel wide take its hairline path and cost almost nothing, but that's
    a third as wide as today on a 2x screen; strokes split in pieces gain ~10% and change the joins. Mine: leave it.

## bitview-charts: raised by you, still open

- OKLCH throughout (2026-10-31): the library worked in OKLCH (its color math, tints and fades, from OKLCH colors), and
  every color in the app written and derived in OKLCH; contrast judged by APCA (the best perceptual formula there is:
  WCAG 3 has left its own undecided; CSS's contrast-color() uses WCAG 2's). Done in the library: the axis tags (tint()), heatmap gradients (gradientTable()), fades (Fill's stops); left:
  Studio's own colors written in OKLCH (its palettes, CSS tokens: ~30 hex and ~80 rgb() left, 2026-11-01).

## Studio: raised by you, still open

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
