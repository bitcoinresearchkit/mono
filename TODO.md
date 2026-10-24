# TODO

Only what's left. An item done is removed: its history is in the commits and logs.

## Waiting on you

- Test Studio on a real phone. Its motion was measured in headless Chromium only.
  - Scrolling, on Sats per Dollar:
    - pinch and fling across 2011–2015, where the axis labels change width most;
    - do the same at the latest bar;
    - turn the phone and back.
  - Auto: pinch and fling through a few interval switches. Check that nothing shifts sideways, the time axis fades with the plot, and the switch comes quickly once the view rests.
- Decide whether Auto becomes the default interval again (it's opt-in now).

## Studio on the new server

Served by the server at DOMAIN/studio/ (bitview.space, next.bitview.space; studio.bitview.space redirects there).

- Deploy the MCP page only once `main` has `showcases/assets/fonts` (its fonts load from there through jsDelivr).
- Presets for the well-known indicators nothing shows yet, as formulas over published series (the server keeps
  formulas out): NVT (market cap over the 24-hour transfer volume in dollars), Thermocap multiple (market cap over
  `thermo_cap`) and RHODL (the 1-day-to-1-week band's realized cap over the 1-to-2-year bands', age-weighted).
- Alerts: being told when a series crosses a level (push, email or a webhook). Only the server can watch for it and
  deliver it (alerts that fire only while Studio's tab is open are a weak promise): a back-end feature, with its own
  design, before Studio can offer it.

## Showcases

The website is gone: the server serves the showcases, Bitview's front page (`showcases/index.html`) at `/`.

- Each showcase's production is standalone at `<name>.DOMAIN`: one build, like Studio's, that inlines the shared
  modules, fonts, pool logos, the logo and favicons into each page, and points its root links (`/studio/`, `/api/`)
  at bitview.space; the server keeps serving the sources.
- A home for what only the website had: block, transaction and address pages.

## Back end

- Indexer version 45 (2026-10-20: OP_RETURN markers, EPOBC) reindexes the Mac mini and production from scratch on first start; plan the downtime. This Mac is on 45 already: its regions were renamed in place for the 2026-10-23 renames (`_notes/tree-review-20261023/rename_regions.py`), which a fresh index doesn't need.

- Fill v1 block extras' `orphans` from the node's `getchaintips` (the non-active tips at the block's height, with their
  `prevhash`), as mempool.space does; it's always empty now. Additive, so after the release.

- Esplora routes mempool.space serves and Bitview doesn't (API.md), each only if it answers as fast as the address
  routes (no rate limit here, unlike mempool.space): `/api/scripthash/{hash}` with `/txs`,
  `/txs/chain[/{last_seen_txid}]`, `/txs/mempool` and `/utxo` (the address routes keyed by the Electrum script hash,
  SHA-256 of the script: needs a new script-hash index, as `addr_hash` isn't that hash), `/api/address-prefix/{prefix}`
  (up to 10 addresses by their text: needs a text-prefix index, as electrs's optional `--address-search`), and
  `POST /api/txs/package` (Bitcoin Core 28+ `submitpackage`, up to 25 transactions; cheap here, under `POST /api/tx`'s
  pending limit). `/fee-estimates` (deprecated upstream) and `/block-template` (not on mempool.space) stay out.

- mempool's electrs address extensions, live on mempool.space, each only if fast enough unthrottled:
  `/api/address/{address}/txs/summary[/{last_seen_txid}]` (up to 5,000 rows `{txid, height, value, time,
  tx_position}`, `max_txs` at most that), `POST /api/addresses/txs` and `POST /api/addresses/txs/summary` (several
  addresses' histories merged, body capped).

- Once next.bitview.space serves the dollar windows missing until priced (`bitview_vecs::PricedWindow`), drop Studio's Puell multiple guard (`lag`, in `showcases/studio/presets.js`): it hides its 10Y decade point and needs 4h+.
