# URPD query ownership

Resolve cohort, date, aggregation, weight, publication availability and packed
source bounds before matching conditionals. Matching tags do not skip missing
source or malformed-input errors. Raw and weighted queries share checked
decoding/aggregation; never silently truncate oversized or incomplete sources.

There is no retained URPD JSON cache. Matching requests validate captured source
entries without constructing buckets/JSON. Separate work and encoded-response
permits bound in-flight capture/build and slow transmission respectively. Body
ownership survives compression and frames retained after the HTTP future ends.
These are endpoint resource limits, not a total process-memory limit.

## Age filters

The aggregate filters are `all`, `sth`, `lth`, `under_4m`, `under_6m`,
`over_4m`, and `over_6m`. The boundaries are 120, 150 and 180 days;
`under` is strict and `over` includes the boundary. Individual disjoint age
ranges remain available under names such as `utxos_4m_to_5m_old`.
All filters support raw, Cointime and Coinflow weights through the same history
reconstruction. For example:

`/api/urpd/under_4m/900000?weight=cointime&agg=raw`

Cointime and Coinflow expose the same per-block URPD metric layout for each
aggregate: cost-basis percentiles per coin/per dollar, capitalized price and
supply density. Each plugin projects all requested cohorts in one pass; the
age filters do not require separate copies of UTXO history or saved URPDs.
