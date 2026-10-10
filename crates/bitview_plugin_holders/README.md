# Holders

Owns the overlapping all, STH, LTH, under-four-month, under-six-month,
over-four-month and over-six-month cohorts. Every filter has the same metric
layout, constructor and computation: supply, counts, activity, realized and
unrealized accounting, cost-basis statistics and relative metrics.

Accounting totals are collected in batches from Age's completed disjoint bands.
Exact raw capital products are summed before computing weighted prices. They
are consumed during calculation rather than copied into this database.
A resident price index advances from the canonical UTXO Set and supplies
percentiles and density for all seven filters. Shared price-index algorithms
live in Distribution; this plugin owns its index, database and recovery.

The plugin implements one complete `ComputePlugin::compute_state()`. It consumes
read-only Age, History, price and mapping dependencies. Append updates retain
validated state; reopen, reorg, source-version changes and partial accounting
writes restore the canonical state. Incomplete derived ratios are repaired
without replaying the history index.

No URPD distributions or address state are stored here. Detailed profitability
bands live in the Profitability plugin. Downstream plugins, including Rarity
Meter, read these completed aggregate metrics instead of computing their own
raw threshold prices.

The realized price and MVRV are second ids for `cost_basis.per_coin.avg` and its spot ratio. NUPL
has one stored fixed-point source, computed alongside the block's P&L from net
unrealized profit/loss divided by the cohort market cap. Zero market cap produces
zero. Negative-loss and half-supply presentation aliases are omitted.
