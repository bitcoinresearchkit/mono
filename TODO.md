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

- Before going live, decide what happens to presets people edited on the old tree: their places are gone, so the edits match no preset (kept in the browser, shown nowhere, only in Download all), and old `/chart/…` addresses open the Charts page.
- Going to a date while an Auto switch is still loading (a ~130 ms window), at the interval still on screen, leaves the view where it was (`goToDate`'s Auto branch when the interval it asks for is the one drawn).

## Showcases

The website is gone: the server serves the showcases, Bitview's front page (`showcases/index.html`) at `/`.

- Each showcase's production is standalone at `<name>.DOMAIN`: one build, like Studio's, that inlines the shared
  modules, fonts, pool logos, the logo and favicons into each page, and points its root links (`/studio/`, `/api/`)
  at bitview.space; the server keeps serving the sources.
- A home for what only the website had: block, transaction and address pages.

## Back end

- Indexer version 45 (2026-10-20: OP_RETURN markers, EPOBC) reindexes every machine with an index (this Mac, the Mac mini, production) from scratch on first start; plan the downtime. The feature-region rename (`_notes/tree-review-20261020/rename_regions.py`) is no longer needed.

- Regenerate the client API docs (`scripts/js-docs.sh`, `scripts/python-docs.sh`; the release script runs them). They still show the removed URPD dates endpoint and the URPD's old shape.
- Hawk's 323 findings, all in back-end crates: bitview_client, bitview_cohort, bitview_primitives, bitview_types, brk_pco, brk_lsm_tree, brk_types, bitview_catalog, brk_fjall.
- Rebuild Hawk for rustc 1.99.1. It only runs as `cargo +1.98.1 hawk check` since the merge moved the toolchain.
