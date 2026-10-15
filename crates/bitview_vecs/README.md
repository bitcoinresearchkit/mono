# bitview_vecs

Reusable metric vectors and views composed by Bitview plugins.

`vecdb` supplies storage, cache and reader primitives (including `LazyVec` and
`LazyDeltaVec`); `bitview_compute` supplies range and stateful algorithms. Neither
depends on this crate. Cohort shapes and cohort families belong to `bitview_cohort`
and `bitview_distribution`.

## Layout

- `shapes` (public): typed field groups (resolutions, windows, percentiles, lookback
  periods, percent views). They hold values; they do not import vectors or choose
  cache policy.
- `ops` (public): stateless, element-wise value transforms (currency conversion,
  ratios, arithmetic) implementing vecdb's unary, binary or delta interfaces with only
  the numeric bounds they need. Generic adapters (`Ident`, `Convert`, `MapOption`,
  `ReverseOperands`) belong to vecdb.
- `views`: lazy readers over one source and aligned metadata (`LazyIndexedVec`,
  `LazyLookbackVec`, `LazyPreviousDeltaVec`, `LazyRollingRatioVec`).
  Each read works through bounded input batches and retains no result.
- `sources`: index readers (`IndexSources`, `RangeMapVec`), window starts
  (`LazyWindowStartVec`), cached stored series (`CachedSeries`) and running totals
  (`CumulativeSource`).
- `resolutions`: a height source per time and chain resolution (`Resolutions`,
  `LazyAggVec` with its `AggFold` strategies, `LazyIndexes`, `DerivedResolutions`).
- `block`: per-block families, stored and lazy, with cumulative, rolling and delta
  views; percentile and density columns; per-type entry counts.
- `rolling`: rolling sums, averages, deltas and distributions from height sources.
- `value`, `fiat`, `percent`, `ratio`: unit-specific compositions (`Price`,
  `PriceWithRatio`, `PriceWithMvrv`, `CostBasisMeans`, `count_and_share`, ...).

Other modules are private; the crate root exposes the supported types. Files are named
relative to their parent module (`block/rolling.rs`, not `block/block_rolling.rs`) and
define at most one public struct. Implementation-only helpers stay private to their owner.

## Composition

Larger families retain their components: `PriceWithRatio` owns a price and its stored
ratio (`PriceWithMvrv` adds the ratio's MVRV id); `CostBasisMeans` adds a cohort's
percentile views, owned and computed elsewhere, to its means. Traversal flattening keeps
the public field layout without copying the components' fields into every wrapper.

`Price` is the shared USD-over-cents shape for stored, lazy and per-resolution price
views; its field types keep each sampling policy. The spot price, its candles and their
open/high/low/close views belong to `bitview_plugin_price`.

## Ownership

Source owners opt into budgeted range retention with `CachedSeries`. The application
initializes vecdb's process-wide budget once before importing sources; constructors do
not forward budget arguments. Transaction-index vectors stay uncached.

Views borrow cloneable readers at construction and keep read-only clones; they create no
cache of their own. Read retention is always evictable; computation working data belongs
to the plugin. Views read only the source ranges a request needs, in batches whose
scratch is bounded, and finish each batch's secondary reads before borrowing source
chunks, since a source may hold a non-reentrant publication lock while lending them.

Writer-only state such as a running total's last value uses `StorageMode::WriteOnly`, so
read-only handles carry none of it.
