# TODO

Only what's left. An item done is removed: its history is in the commits and logs.

## Waiting on you

- Test Studio on a real phone. Its motion was measured in headless Chromium only.
  - Scrolling, on Sats per Dollar:
    - pinch and fling across 2011–2015, where the axis labels change width most;
    - do the same at the latest bar;
    - turn the phone and back.

## Studio: raised by you, still open

- Thumbnails and previews of a chart with a heatmap draw its price alone (Codex, 2026-10-28): the heatmap's days
  aren't fetched for a tile. A sampled mini heatmap there, to decide.

- A gradient's range at an interval under a day (Codex, 2026-10-28): of the history loaded (it grows as you pan
  back, its colors then shifting), not all of it; all of it would mean loading every chunk, or the day interval's
  extremes read beside it. To decide.

- The rail's icons (you, 2026-10-27: do it): an icon per preset section; your folders' first letter by default, an icon
  settable for each in the Library (no colors: you, powerful but not complicated).
- Renaming or making a folder from the rail (you, 2026-10-27: do it): the compact editor beside its row, as mocked up,
  not ⌘K's page.
- Then a full pass over everything touched today and what it impacts (you, 2026-10-27): bugs, consistency, motion,
  speed, duplication.
- The folders on the rail: "a ton of polish" needed; what you saw beyond what's fixed (depth, one menu, the flash,
  menu-bar switching, the toggle's speed and stale rows).
- Shortcuts opening by itself during quick sidebar toggles: its button takes no press mid-slide now (5 presses on it as
  it slid: none opened it); tell me if you still see it.

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
