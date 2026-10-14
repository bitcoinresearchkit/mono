# bitview_vecs

Reusable metric vectors and views composed by Bitview plugins.

Plugins compose the vectors in this crate. `vecdb` supplies storage, cache,
and reader primitives; `bitview_compute` supplies stateful/range algorithms;
`bitview_cohort` supplies cohort shapes. None of those crates depends on this one.

## Layout

- `shapes` (public): typed field groups (resolutions, windows, percentiles,
  investment periods, percent views). They contain values; they do not
  import vectors or choose cache policy.
- `ops` (public): stateless, element-wise value transforms (currency conversion,
  ratios, arithmetic) implementing vecdb's unary or binary transform interfaces
  with only the numeric bounds they need. Generic adapters (`Ident`, `MapOption`,
  `ReverseOperands`) belong to vecdb; Bitcoin block target constants to `brk_types`.
- `views`: single-source readers with aligned metadata and no retained result cache.
- `sources`: shared index and window-start readers.
- `resolutions`: composition of a height source into time and chain resolutions.
- `block`: per-block metric families.
- `rolling`: rolling and delta compositions.
- `value`, `fiat`, `percent`, `ratio`: unit-specific compositions.

Other modules are private; the crate root exposes the supported types. Files are
named relative to their parent module (for example, `block/rolling.rs`, not
`block/block_rolling.rs`) and define at most one public struct.
Small implementation-only helpers stay private to their owner or family.

## Composition

Larger families retain their components: `PriceWithRatio` owns a price and
its stored ratio (`PriceWithMvrv` adds the ratio's MVRV id); cumulative families share `RollingTotals`/`RollingAmountTotals`;
`ValuePerBlockFull` adds distribution to the cumulative/rolling value family.
Traversal flattening preserves the public field layout without copying those
components' fields into every wrapper.

`Price` is the shared USD-over-cents shape for stored, lazy, and per-resolution
price views; its field types retain each sampling policy. The spot price, its
OHLC candles and their split views belong to `bitview_plugin_price`.
Investment-specific stacks retain the existing `LazySpotValuePerBlock` instead
of redeclaring its unit fields.

`TypeCounts` composes a typed breakdown with `CountTotal`. Plugins select the
total's adjustment and retention, including coinbase exclusion; vector assembly
does not infer those business rules. Daily percentile prices, mapping
readers, and integer-cent SMA readers also live here; their plugins retain
provider, calendar, percentile-selection, and period-selection policy.

## Ownership

Source owners select optional, budgeted range retention. The application initializes
vecdb's process-wide budget once before importing sources; constructors do not
forward budget arguments. `CachedSeries` makes the opt-in explicit. Stored height
sources and daily sources without a stored height counterpart can retain selected
ranges; transaction-index vectors stay uncached.

Views borrow cloneable readers at construction and retain read-only clones;
they do not create another cache for each resolution or metric. Read retention
is always evictable; computation working data belongs to the plugin and is
passed through its computation context, not held alive by a cache policy.

Date, first-height, and lookback readers derive their results directly from
monotonic stored sources. Period lookups seek to the requested boundary before
scanning. SMA readers share one stored cumulative-price source. Per-type input
and output counts store cumulative `u64` totals, with block counts derived by
subtraction. Neither needs reader-owned prefix checkpoints or revision tokens.
Rolling and ratio readers request only the needed source ranges, including on
a cold budgeted cache. Ordinary stored-vector recomputation and source-owned
invalidation handle appends and rewrites.

Writer-only cumulative checkpoints use `StorageMode::WriteOnly`, so read-only
handles contain no unused checkpoint payload. Exposing mutable storage invalidates
their writer checkpoint. Count-total and transformed-denominator views read
their sources directly and need no separate invalidation after rewrites.
