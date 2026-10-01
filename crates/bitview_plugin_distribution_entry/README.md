# Distribution Entry

Optional price-defined UTXO cohorts, excluded from `bitview_default`.
Creation-block spot price at or below the previous block's all-chain capitalized
price selects discount (Veteran); a higher price selects premium (Rookie).
A zero anchor selects discount. Membership stays fixed for the output's lifetime.

Run after Aggregated, supplying its completed all-chain capitalized-price series plus
published UTXO history, block prices and monotonic timestamps. The plugin consumes
a borrowed `statedb::Reader`; it owns neither history nor the anchor series.
It owns its database, entry classifications, price maps, update and recovery.
Restart, reorg and failed updates reconstruct from canonical history and the
published anchor series. Only validated append updates reuse resident state.

Accounting, cost-basis maps and scalar metric views are shared libraries. Age
contains no entry classifications, entry state or entry recovery boundary.

The runnable `with_entry` example composes this plugin with `DefaultPlugins`,
computes it after Aggregated and publishes both under the existing update boundary:

```sh
cargo run -p bitview_plugin_distribution_entry --example with_entry -- --help
```

The optional catalog exposes `distribution_entry.cohorts.discount` and
`distribution_entry.cohorts.premium`. Existing `veteran_*` and `rookie_*` series
names are preserved. Default clients and website charts omit those series.
