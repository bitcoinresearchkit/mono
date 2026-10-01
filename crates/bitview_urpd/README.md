# URPD views

There is one durable source: `statedb` stores remaining sats and UTXO counts by
creation height, using block diffs and full snapshots. Inputs and Outputs supply
their contributions during their existing scans. The UTXO History plugin
publishes their complete prefix and owns snapshots, every 5,000 blocks and at the
latest published block. Neither prices nor URPDs are stored in that history.

`Reader::state_at(end)` reconstructs an arbitrary block from the nearest matching
snapshot and its following diffs. A `Cursor` applies the same diffs to resident
state during sequential processing. These are storage operations; they know
nothing about age filters or weighting models.

`OriginUrpd` is the shared price/age view of that state. It groups remaining sats
by rounded creation price and age. Both API queries and plugin computations use
its `project(weights, filters)` iterator. A filter is a slice of age ranges; raw,
Cointime, and Coinflow supply use the same projection. Multiple weights and
filters are projected together in one histogram pass, rounding weighted sats
only after combining ages within each price bucket. Raw totals remain integers.
The histogram maintains a private four-byte occupancy mask per price bucket, so
projection visits populated ages instead of checking all 23 cells. It updates
that mask alongside creations, removals and age crossings; consumers do not
handle this metadata. Each requested filter is another compact age mask, so
excluded ages are skipped before applying any model weights. Statistics reuse
their cumulative supply totals for density and classify each price once across
all cohorts.

`Replay` retains a consumer's state and histogram between successful updates. It
adds new prices, moves supply across age boundaries, and applies removals and
creations for each block. It rebuilds after a reorg, a source version change, or
a failed update. Each plugin owns its complete `compute()` call and supplies its
read-only dependencies and model weights; replay owns reconstruction details.

An isolated query constructs this view once. A backfill constructs it once and
updates it with diffs. Model weights can change every block, so weighted scalar
analytics still visit the histogram at each block they compute. There are no
persisted URPDs or per-model copies of origin state within a plugin.

The sequential loop is therefore: restore the nearest snapshot once, apply diffs
to reach the first requested block, then repeat **advance state and histogram →
project filters and weights → compute the consumer's data**. Successful updates
retain that state for the next call. Only a discontinuity requires another restore.
