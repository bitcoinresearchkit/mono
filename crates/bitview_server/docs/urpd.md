# URPD query ownership

Resolve cohort, date, aggregation, weight, publication availability and packed
source bounds before matching conditionals. Matching tags do not skip missing
source or malformed-input errors. Unweighted and weighted queries share checked
decoding/aggregation; never silently truncate oversized or incomplete sources.

There is no retained URPD JSON cache. Matching requests validate captured source
entries without constructing buckets/JSON. Separate work and encoded-response
permits bound in-flight capture/build and slow transmission respectively. Bodies
of at least 256 KiB keep their response permit through compression and the final
frame, including frames retained after the HTTP future ends.
These are endpoint resource limits, not a total process-memory limit.

## Age filters

The aggregate filters are `all`, `sth`, `lth`, `age_under_4m`, `age_under_6m`,
`age_over_4m`, and `age_over_6m`. The boundaries are 120, 150 and 180 days;
`under` is strict and `over` includes the boundary. Individual disjoint age
ranges remain available under names such as `age_4m_to_5m`.
All filters support the weights `none`, `wakefulness` (Cointime's, the distribution behind the
`*_awake_*` series) and `mobility` (Coinflow's, behind `*_mobile_*`) through the same history
reconstruction. For example:

`/api/urpd/age_under_4m/900000?weight=wakefulness&agg=raw`

Cointime and Coinflow expose the same per-block URPD metric layout for each
aggregate: cost-basis percentiles per coin/per dollar, capitalized price and
supply density. Each plugin projects all requested cohorts in one pass; the
age filters do not require separate copies of the UTXO set history or saved URPDs.
