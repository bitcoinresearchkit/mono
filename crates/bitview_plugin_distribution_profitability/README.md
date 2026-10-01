# Distribution Profitability

Optional percentage-profit/loss bands for all, STH, LTH, under 4m, under 6m,
over 4m, and over 6m. Each of the 25 bands exposes supply, creation-value cap,
unrealized profit/loss, and NUPL through the same layout.

The plugin consumes published UTXO History, block prices, and monotonic timestamps.
It owns one complete `ComputePlugin::compute()`, its database, and a resident
price index. Four overlapping totals (all, STH, under 4m, under 6m) represent all
seven filters; complementary totals are derived. Only the three relevant age
cutoffs are advanced. The shared distribution library owns price-index algorithms;
this plugin owns their state and lifetime.

Append updates retain validated state. Reorgs, changed sources, failed updates,
and restarts rebuild from History's closest snapshot plus diffs. Only fixed-width
metric series are persisted here; History remains the source of unspent state.
Writes use the project exit guard. Crash recovery is outside the project contract.

This crate is absent from `DefaultPlugins`. Compose it explicitly as in
`examples/with_profitability.rs`. Its series are registered under this plugin's
identity and are absent when the plugin is omitted. Basic profit/loss, cost-basis
percentiles, and density remain with Distribution Age. Age no longer creates or maintains detailed profitability-band series.

The existing price-grid precision and all/STH/LTH band rounding are preserved.
The additional filters use the same grid, cutoff definitions, and complementary
cap rounding. NUPL is exposed for every filter, including zero-supply buckets.
