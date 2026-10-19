# TODO

Only what's left. An item done is removed: its history is in the commits and logs.

## Waiting on you

- Test Studio's scrolling on a real phone. The jump fixes were measured in headless Chromium only. On Sats per Dollar:
  - pinch and fling across 2011–2015, where the axis labels change width most;
  - do the same at the latest bar;
  - turn the phone and back.
- Decide when Studio goes live. Recommended: wait for the new server, because the live server lacks a few series the merged Studio names (`hashrate`, `sats_per_dollar`, `*_avg_24h`).

## Studio, next

- Units for the remaining value types: Years, Seconds/SecondsFract, PerSecond, PerDay, Bytes/BytesFract, Weight/WeightFract, VSize/VSizeFract.
- Format menu (`FORMATS`): offer cents, basis points, ppm, hash rate, fee rate and days by hand, not only through Automatic.
- Auto interval polish, so it could become the default again:
  - a 2–4px sideways shift during the crossfade;
  - time-axis labels that change instantly;
  - a 0.7s delay before the switch.

## Once the new server is live

- Check Studio against it, then deploy Studio live (`node showcases/studio/deploy.mjs live`):
  - presets with the new ids;
  - hash rate in H/s, days and difficulty in their units;
  - by-block series' `_avg_24h` days found.
- Remove the compatibility code the old server needed:
  - `unitOf`'s name rules for `StoredF32`/`StoredF64`;
  - the ppm, price-ratio, cents and bps unit handling;
  - the build's copy of this repo's date helpers when the served client differs (`showcases/studio/build.mjs`).

## Back end

- Hawk's 323 findings, all in back-end crates: bitview_client, bitview_cohort, bitview_primitives, bitview_types, brk_pco, brk_lsm_tree, brk_types, bitview_catalog, brk_fjall.
- Rebuild Hawk for rustc 1.99.1. It only runs as `cargo +1.98.1 hawk check` since the merge moved the toolchain.
