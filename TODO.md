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

Built against next.bitview.space; deployed to studio-next only, never studio.bitview.space until you say.

- Release together: the built Studio imports `/modules/bitview-client/index.js`, which only the new server serves, and
  the old Studio reads the website's import map, which the new server lacks. Upgrade the server and redeploy Studio at
  once. Deploy the MCP page only once `main` has `showcases/assets/fonts` (its fonts load from there through jsDelivr).

- Old `/chart/…` addresses (presets' places on the old tree) open the Charts page: decide whether they lead anywhere.

## Showcases

The website is gone: the server serves the showcases, Bitview's front page (`showcases/index.html`) at `/`.

- Each showcase's production is standalone at `<name>.DOMAIN`: one build, like Studio's, that inlines the shared
  modules, fonts, pool logos, the logo and favicons into each page, and points its root links (`/studio/`, `/api/`)
  at bitview.space; the server keeps serving the sources.
- A home for what only the website had: block, transaction and address pages.

## Back end

- Indexer version 45 (2026-10-20: OP_RETURN markers, EPOBC) reindexes every machine with an index (this Mac, the Mac mini, production) from scratch on first start; plan the downtime. The feature-region rename (`_notes/tree-review-20261020/rename_regions.py`) is no longer needed.

- OP_RETURN's transactions are one number computed twice: `op_return_tx_count`/`_tx_share` (the OP_RETURN plugin) and `op_return_output_tx_count`/`_tx_share` (outputs by type), same definition; and both plugins name their output count `op_return_output_count`. One plugin should take the other's. Studio pairs them meanwhile (`SAME` in `showcases/studio/presets.js`).

- Dollar flows are 0, not missing, before the first exchange price (block 74,641): a sum over a window reaching back past it reads short (Studio's Puell multiple guards for it with `lag`, which hides its 10Y decade point and needs 4h+). Serve them missing there, the windows' sums missing until wholly priced; then drop the guard in `showcases/studio/presets.js`.
