# Profitability

Profit and loss bands of the UTXO set: 25 bands from over 1000% in profit to
90-100% in loss, for all, short-term and long-term holders, at
`profitability.<filter>.<band>`. Each band
publishes its supply and capital, each with its share of the filter's total (the
bands partition the filter; a share is null while that total is zero).

The plugin is a consumer of the one walk of the UTXO Set's history
(`bitview_distribution::walk`), with block prices and monotonic timestamps.
It owns its database and the one resident price index. The index tracks
overlapping totals from which the long-term side is derived as all minus
short-term. The shared distribution library owns price-index algorithms; this
plugin owns their state and lifetime. From the same index it stores the seven
overlapping filters' cost-basis percentiles, density and price extremes, which
Holders publishes.

Append updates retain validated state. Reorgs, changed sources, failed updates,
and restarts rebuild from History's closest snapshot plus diffs. Only fixed-width
metric series are persisted here; History remains the source of unspent state.
Writes use the project exit guard. Crash recovery is outside the project contract.

`DefaultPlugins` computes it in the walk, with Age and the URPD history. Basic
profit/loss lives in Holders, which also publishes the cost-basis series.

Creation prices sit on the shared price index's grid, four significant digits of
the price in cents (exact under $100, at most 0.05% off above). Each band edge, the
spot scaled by an exact rational factor, rounds up to the next grid price (never to
whole cents first), and the spot's own grid price counts as profit: break-even
belongs to the profit side. Long-term totals are all minus short-term, before cents rounding.
