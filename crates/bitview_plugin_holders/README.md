# Holders

Owns the overlapping all, STH, LTH, under-four-month, under-six-month,
over-four-month and over-six-month cohorts. Every filter has the same
constructor and computation: supply, counts, activity, realized and unrealized
accounting and cost-basis statistics, typed by its kind where the layout
differs: all has no share of all (100% throughout), and LTH and the over-four-
and over-six-month cohorts no aSOPR (they hold no output under an hour old, so
it equals their SOPR).

Accounting totals are collected in batches from Age's completed disjoint bands.
Exact raw capital products are summed before computing weighted prices. They
are consumed during calculation rather than copied into this database.
Percentiles, density and price extremes for all seven filters are read from the
Profitability plugin, which computes them from the one price index.

The plugin consumes read-only Age, Profitability and price dependencies and keeps
no state between updates: every series derives from their completed series, so a
reopen, a reorg or a source-version change needs no recovery of its own.

No URPD distributions or address state are stored here. Detailed profitability
bands live in the Profitability plugin. Downstream plugins, including Rarity
Meter, read these completed aggregate metrics instead of computing their own
raw threshold prices.

The realized price and MVRV are second ids for `cost_basis.by_coin.avg` and its spot ratio. NUPL
has one stored fixed-point source, computed alongside the block's P&L from net
unrealized profit/loss divided by the cohort market cap. Zero market cap leaves it
undefined (null).
