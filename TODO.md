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

Built against next.bitview.space, on studio.bitview.space (each build of a commit deploys it).

- Release together: the built Studio imports `/modules/bitview-client/index.js`, which only the new server serves, and
  the old Studio reads the website's import map, which the new server lacks. Upgrade the server and redeploy Studio at
  once. Deploy the MCP page only once `main` has `showcases/assets/fonts` (its fonts load from there through jsDelivr).

## Showcases

The website is gone: the server serves the showcases, Bitview's front page (`showcases/index.html`) at `/`.

- Each showcase's production is standalone at `<name>.DOMAIN`: one build, like Studio's, that inlines the shared
  modules, fonts, pool logos, the logo and favicons into each page, and points its root links (`/studio/`, `/api/`)
  at bitview.space; the server keeps serving the sources.
- A home for what only the website had: block, transaction and address pages.

## Back end

- Indexer version 45 (2026-10-20: OP_RETURN markers, EPOBC) reindexes every machine with an index (this Mac, the Mac mini, production) from scratch on first start; plan the downtime. The feature-region rename (`_notes/tree-review-20261020/rename_regions.py`) is no longer needed.

- OP_RETURN's transactions are one number computed twice: `op_return_tx_count`/`_tx_share` (the OP_RETURN plugin) and `op_return_output_tx_count`/`_tx_share` (outputs by type), same definition; and both plugins name their output count `op_return_output_count`. One plugin should take the other's. Studio pairs them meanwhile (`SAME` in `showcases/studio/presets.js`).

- Once next.bitview.space serves the dollar windows missing until priced (`bitview_vecs::PricedWindow`), drop Studio's Puell multiple guard (`lag`, in `showcases/studio/presets.js`): it hides its 10Y decade point and needs 4h+.
