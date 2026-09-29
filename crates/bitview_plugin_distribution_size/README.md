# Distribution Size

Owns amount, output-type and address cohorts. Its loop consumes resolved input
values, indexed outputs and addresses, block boundaries and prices. It does not
need Transactions or the Age loop.

It retains balances, counts, activity and realized cost basis/profit/loss.
Advanced age-style analytics and full price maps are absent. Realized balances
have one owner in memory. A single 41-element vecdb state saves their exact
128-bit values alongside the mutable address vectors, with matching rollback
stamps. Successful updates retain validated in-memory state for the next append.
Global supply and UTXO counts are borrowed from UTXO History.
