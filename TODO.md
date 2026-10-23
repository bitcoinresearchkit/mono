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

## Showcases

The website is gone: the server serves the showcases, Bitview's front page (`showcases/index.html`) at `/`.

- Each showcase's production is standalone at `<name>.DOMAIN`: one build, like Studio's, that inlines the shared
  modules, fonts, pool logos, the logo and favicons into each page, and points its root links (`/studio/`, `/api/`)
  at bitview.space; the server keeps serving the sources.
- A home for what only the website had: block, transaction and address pages.

## Back end

- Indexer version 45 (2026-10-20: OP_RETURN markers, EPOBC) reindexes the Mac mini and production from scratch on first start; plan the downtime. This Mac is on 45 already: its regions were renamed in place for the 2026-10-23 renames (`_notes/tree-review-20261023/rename_regions.py`), which a fresh index doesn't need.

- Once next.bitview.space serves the dollar windows missing until priced (`bitview_vecs::PricedWindow`), drop Studio's Puell multiple guard (`lag`, in `showcases/studio/presets.js`): it hides its 10Y decade point and needs 4h+.
