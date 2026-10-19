# Distribution

Shared accounting, price maps, price indexes and metric helpers used by Age,
Holders, UTXOs, Addresses, Entry, Profitability, Cointime and Coinflow. This library
owns no database, producer or pipeline loop: plugins compose its column readers and
checkpoint mechanics and own their checkpoints, derived state and recovery.
UTXOs and Addresses compile out optional price-map operations. Realized balances
live only in shared accounting state.

Shared families keep one implementation per shape: the creation-cohort bundle
(Age, Entry), the spent/unspent output counts, the weighted-model sources (Cointime,
Coinflow) and the replay resume (`replay::run` for Addresses and UTXOs). Each plugin
keeps its classification, model and chunk loop.

Holders and Profitability share price-index bucketing, cumulative queries,
percentiles and history advancement, choosing the filter totals their queries need;
the indexes and their lifetimes belong to the plugins.
