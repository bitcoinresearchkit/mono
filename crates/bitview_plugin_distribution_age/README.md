# Distribution Age

Consumes published UTXO History plus block prices and monotonic timestamps.
It owns disjoint age bands and fixed creation-year and epoch cohorts, including
their supply, activity, maturation and profit/loss accounting. Derived price maps remain in memory;
restart reconstructs them from canonical history. Append updates retain
validated state. Reorgs and failed updates rebuild it.

The overlapping all/STH/LTH and four/six-month filters, cost-basis percentiles,
density and aggregate-relative metrics belong to Distribution Aggregated.
Age publishes exact disjoint raw capital products so consumers can combine
bands before rounding weighted prices. These intermediate products stay out of
the public catalogue. Age exposes total realized cap, with no cap change/growth,
realized-price ratios, SOPR, MVRV, NUPL or presentation-only loss/supply aliases.

Age owns no address state and stores no URPD files. Block-based URPD models and
queries reconstruct distributions through `bitview_urpd`. Entry-price
classification and detailed percentage-profitability bands belong to their
separate optional plugins.
