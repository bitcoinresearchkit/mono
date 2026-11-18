# Entry

Price-defined UTXO cohorts, part of `bitview_default`.
Creation-block spot price at or below the previous block's all-chain capitalized
price selects discount (entered at or below it); a higher price selects premium.
An undefined anchor (the genesis block, or no priced capital yet) selects
discount. Membership stays fixed for the output's lifetime.

Run after Holders, supplying its completed all-chain capitalized-price series plus
the published UTXO set, and the block prices, their order and the monotonic timestamps
the completed walk of its history holds. The plugin consumes a borrowed `statedb::Reader`; it
owns neither history nor the anchor series, and keeps of the origins only the sats each holds
(the coins that change side when the spot passes their price).
It owns its database, entry classifications, update and recovery.
Restart, reorg and failed updates reconstruct from canonical history and the
published anchor series. Only validated append updates reuse resident state.

Accounting, the split at the spot and scalar metric views are shared libraries.

The catalog exposes `entry.discount` and `entry.premium`, as `entry_discount_*` and
`entry_premium_*` series.
