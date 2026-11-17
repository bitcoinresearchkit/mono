# URPD views

The durable source is `statedb`: remaining sats and UTXO counts by creation
height, as block diffs and full snapshots. Inputs and Outputs supply their
contributions during their existing scans. The UTXO Set plugin publishes their
complete prefix and owns the snapshots, every 500 blocks and at the latest
published block.

`UrpdHistory` stores the distribution itself after every block: an age range's
sats at one rounded creation price is a cell, kept as each block's changes with
every cell checkpointed each 250 blocks (statedb's `CellHistory`). The UTXO Set
plugin produces it right after the history it derives from. API queries read one
block's cells (a checkpoint and at most 249 blocks of changes) and project them;
they never rebuild a UTXO set.

`Reader::state_at(end)` reconstructs an arbitrary block from the nearest matching
snapshot and its following diffs. A `Cursor` applies the same diffs to resident
state during sequential processing. These are storage operations; they know
nothing about age filters or weighting models.

`OriginUrpd` is the plugins' price/age view of that state. It groups remaining
sats by rounded creation price and age, and its `project(weights, filters)`
iterator is the one `UrpdCells` also follows. Both apply a block the same way
(`block_changes`): age crossings first, then creations, then spends. A filter is
a slice of age ranges; raw, Cointime, and Coinflow supply use the same projection. Multiple weights and
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
a failed update. Each plugin owns `compute_state()` and supplies read-only
dependencies and model weights; replay reconstructs state.

Model weights can change every block, so weighted scalar analytics still visit
the histogram at each block they compute.

Long backfills (`Replay::map`) split at snapshot boundaries and run each segment
on the thread pool from its own restored state. Results return in block order;
the last segment's state is retained.
