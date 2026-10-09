# Distribution

Shared accounting, price maps, price indexes and metric helpers used by Age,
Holders, UTXOs, Addresses, Entry and the optional Profitability plugin. This library
owns no database, producer or pipeline loop. Shared column readers and exact
scalar-checkpoint mechanics are composed into each plugin; every checkpoint
instance belongs to its plugin. Plugins own their derived state and recovery.
UTXOs and Addresses compile out optional price-map operations. Realized balances
live only in shared accounting state.

Holders and Profitability share one price-index implementation and canonical
history advancement helpers, choosing only the
filter totals their queries need. Bucketing, cumulative queries and percentiles
have one implementation; the indexes and their lifetimes belong to the plugins.
