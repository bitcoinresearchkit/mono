# TODO

Only what's left. An item done is removed: its history is in the commits and logs.

## Waiting on you

- Test Studio on a real phone. Its motion was measured in headless Chromium only.
  - Scrolling, on Sats per Dollar:
    - pinch and fling across 2011–2015, where the axis labels change width most;
    - do the same at the latest bar;
    - turn the phone and back.

## Studio: raised by you, still open

- The gradient editor and a heatmap's default colors (you, 2026-10-28: "think very hard, how would best designers in
  the world do it", "look for the best setup possible"): built, your look to confirm
  (_notes/overnight-20261028/heatmap-default-and-editor-before-after.png). A cell's strength to the 2.5th power (most
  cells were lit, the map one color), the weakest 30% fading into the chart, Inferno kept; the editor's bar as drawn,
  a value scale under it, Reverse, the color in hand named, your own gradient kept as "Your own" once a ready-made
  is picked (this session).

- Home's search: by its field alone (⌘K is the normal ⌘K), ⌘K's list, all of it, on the field, 10 from its every edge,
  unrolling from it, its corners rounding from the field's to its own (you, 2026-10-28/29: "we want the full power of
  cmd+k but in a tighter window", "cmd+k on home screen should open the normal command palette", "maybe double radii was
  right but we needed a smoother anim/motion ?"): your look to confirm.
- Bubbles: built (a layer type: Add a layer › Bubbles, a layer's ⋯ › As bubbles on…, its Type; On, Size, Hide below,
  Sized to; a preset per cohort, its day's net realized P&L on the price; you, 2026-10-28: "represent a series as bubbles
  attached to another layer"): your look to confirm, and the minis' (tiles, ⌘K's previews, Quick Look draw them now:
  _notes/overnight-20261029/bubbles-mini-before-after.png); the presets' name, for "bubbles" to find them
  (_notes/overnight-20261029/DECISIONS.md #4).
- Outline: built (a layer's switch; presets over a heatmap or a cloud, the price's), the line's width on each side
  (you, 2026-10-28: "outline doesn't feel think enough"; half of it, before): your look to confirm.
- A design system for all of Studio, then a polish and consistency pass applying it (you, 2026-10-28: "I think we
  really need to do a polish/consistency pass over the whole app, and make a proper system"): inventory
  (_notes/system-20261028/inventory.md: 9 font sizes, 20 radii, 22 gaps, 63 heights, 47 shadows, 56 transitions),
  then the system written down for your check (_notes/system-20261028/system.md; corners inside corners concentric,
  unless it makes no sense, you: "all radii inside radii follow the universal radii rule"); the pass applied area by
  area (2026-10-29: sidebar to dialogs and a phone's touch targets, see the system's notes); titles 18, ⌘K's rows 14
  (you: "maybe it should be 16 or 18", "judge too please"): your look to confirm. Body kept at 13 (you: "I think I
  agree for 13"); 15 → 16 for the big fields and titles left open (you: "not sure"; _notes/type-15-vs-16.png): if ⌘K's
  query reads too close to its 14 rows, its field alone at 16.
- The folders on the rail: "a ton of polish" needed; what you saw beyond what's fixed (depth, one menu, the flash,
  menu-bar switching, the toggle's speed and stale rows).

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
