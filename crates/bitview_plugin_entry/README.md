# Entry

Price-defined UTXO cohorts, part of `bitview_default`.
Creation-block spot price at or below the previous block's all-chain capitalized
price selects discount (Veteran); a higher price selects premium (Rookie).
A zero anchor selects discount. Membership stays fixed for the output's lifetime.

Run after Holders, supplying its completed all-chain capitalized-price series plus
the published UTXO set, block prices and monotonic timestamps. The plugin consumes
a borrowed `statedb::Reader`; it owns neither history nor the anchor series.
It owns its database, entry classifications, price maps, update and recovery.
Restart, reorg and failed updates reconstruct from canonical history and the
published anchor series. Only validated append updates reuse resident state.

Accounting, cost-basis maps and scalar metric views are shared libraries. Age
contains no entry classifications, entry state or entry recovery boundary.

The catalog exposes `entry.discount` and `entry.premium`, as `veteran_*` and
`rookie_*` series.
