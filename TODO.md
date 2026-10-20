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

- Fix each showcase for the new server. Cohort ids are `age_<band>` and `amount_<band>` (were `utxos_<band>_old`, `utxos_<band>`) and URPD cohorts `age_under_4m`, `age_2y_to_3y` (Wave builds `utxos_${id}_old_*`, Trail's age filters send `utxos_*_old`). URPD weights are `none`, `wakefulness`, `mobility` (were `raw`, `cointime`, `coinflow`: Floor, Trail, the API page's default); Bedrock builds `bedrock_coinflow_floor_*` (now `bedrock_mobile_*`). Floor, for one, still reads the old series tree (`all.realized.price`…, `rarity.components.realizedPrice`, now `components.all.realizedPrice`), and most point at bitview.space. City, planet and calendar read the per-block pool as `pool` (now `block_pool`) and calendar a pool's blocks as `<slug>_blocks_mined_cumulative` (now `pool_<slug>_block_count_cumulative`).
- Add showcases for what only the website had.

## Back end

- Indexer version 45 (2026-10-20: OP_RETURN markers, EPOBC) reindexes every machine with an index (this Mac, the Mac mini, production) from scratch on first start; plan the downtime. The feature-region rename (`_notes/tree-review-20261020/rename_regions.py`) is no longer needed.

- Regenerate the client API docs (`scripts/js-docs.sh`, `scripts/python-docs.sh`; the release script runs them). They still show the removed URPD dates endpoint and the URPD's old shape.
- Hawk's 323 findings, all in back-end crates: bitview_client, bitview_cohort, bitview_primitives, bitview_types, brk_pco, brk_lsm_tree, brk_types, bitview_catalog, brk_fjall.
- Rebuild Hawk for rustc 1.99.1. It only runs as `cargo +1.98.1 hawk check` since the merge moved the toolchain.
