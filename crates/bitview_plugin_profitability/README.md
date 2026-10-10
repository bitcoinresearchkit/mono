# Profitability

Profit and loss bands of the UTXO set: 25 bands from over 1000% in profit to
90-100% in loss, under each of seven age filters (all, STH, LTH, under 4m,
under 6m, over 4m, over 6m), at `profitability.<filter>.<band>`. Each band
publishes its supply and capital, each with its share of the filter's total (a
filter's bands add up to 100%), and its signed net unrealized profit or loss.

The plugin consumes the published UTXO Set, block prices, and monotonic timestamps.
It owns one complete `ComputePlugin::compute_state()`, its database, and a resident
price index. Four overlapping totals (all, STH, under 4m, under 6m) represent all
seven filters; complementary totals are derived. Only the three relevant age
cutoffs are advanced. The shared distribution library owns price-index algorithms;
this plugin owns their state and lifetime.

Append updates retain validated state. Reorgs, changed sources, failed updates,
and restarts rebuild from History's closest snapshot plus diffs. Only fixed-width
metric series are persisted here; History remains the source of unspent state.
Writes use the project exit guard. Crash recovery is outside the project contract.

`DefaultPlugins` composes it beside Entry, Coinflow and Cointime. Basic
profit/loss, cost-basis percentiles, and density live in Holders.

Creation prices sit on the shared price index's grid: whole dollars to five
significant digits, so bands are coarse while the spot price is under about $100
and collapse into one while it is under $1. Complementary filters (LTH, over 4m,
over 6m) are their totals minus the young side, before cents rounding.
