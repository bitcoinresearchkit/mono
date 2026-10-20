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

Built against local.bitview.space; deployed to studio-next only, never studio.bitview.space until you say.

- Before going live, decide what happens to presets people edited on the old tree: their places are gone, so the edits match no preset (kept in the browser, shown nowhere, only in Download all), and old `/chart/…` addresses open the Charts page.
- Going to a date while an Auto switch is still loading (a ~130 ms window), at the interval still on screen, leaves the view where it was (`goToDate`'s Auto branch when the interval it asks for is the one drawn).

## Showcases

The website is going away: showcases become what's served by default.

- Fix each showcase for the new server. Floor, for one, still reads the old series tree (`all.realized.price`…, `rarity.components.realizedPrice`, now `components.all.realizedPrice`), and most point at bitview.space. City, planet and calendar read the per-block pool as `pool` (now `block_pool`) and calendar a pool's blocks as `<slug>_blocks_mined_cumulative` (now `pool_<slug>_…`).
- Add showcases for what only the website had.

## Back end

- Before running this build on another machine with an index (the Mac mini, production), rename the indexer's empty and unknown feature regions, or the indexer stops with "drop and re-import it": with bitviewd stopped, `python3 -I _notes/tree-review-20261020/rename_regions.py ~/.bitview/plugins/indexer/vecs/regions --apply` (a dry run without `--apply`; it keeps a backup beside the file). Done on this Mac on 2026-10-20.

- Regenerate the client API docs (`scripts/js-docs.sh`, `scripts/python-docs.sh`; the release script runs them). They still show the removed URPD dates endpoint and the URPD's old shape.
- Hawk's 323 findings, all in back-end crates: bitview_client, bitview_cohort, bitview_primitives, bitview_types, brk_pco, brk_lsm_tree, brk_types, bitview_catalog, brk_fjall.
- Rebuild Hawk for rustc 1.99.1. It only runs as `cargo +1.98.1 hawk check` since the merge moved the toolchain.
