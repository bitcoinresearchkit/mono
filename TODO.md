# TODO

Only what's left. An item done is removed: its history is in the commits and logs.

## Waiting on you

- Test Studio's scrolling on a real phone. The jump fixes were measured in headless Chromium only. On Sats per Dollar:
  - pinch and fling across 2011–2015, where the axis labels change width most;
  - do the same at the latest bar;
  - turn the phone and back.

## Studio on the new server

Built against local.bitview.space; deployed to studio-next only, never studio.bitview.space until you say.

- Before going live, decide what happens to presets people edited on the old tree: their places are gone, so the edits match no preset (kept in the browser, shown nowhere, only in Download all), and old `/chart/…` addresses open the Charts page.

## Showcases

The website is going away: showcases become what's served by default.

- Fix each showcase for the new server.
- Add showcases for what only the website had.

## Back end

- Hawk's 323 findings, all in back-end crates: bitview_client, bitview_cohort, bitview_primitives, bitview_types, brk_pco, brk_lsm_tree, brk_types, bitview_catalog, brk_fjall.
- Rebuild Hawk for rustc 1.99.1. It only runs as `cargo +1.98.1 hawk check` since the merge moved the toolchain.
