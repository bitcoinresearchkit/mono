# URPD views

The durable source is `statedb`: remaining sats and UTXO counts by creation
height, as block diffs and full snapshots. Inputs and Outputs supply their
contributions during their existing scans. The UTXO Set plugin publishes their
complete prefix and owns the snapshots, every 500 blocks and at the latest
published block.

`UrpdHistory` stores the distribution itself after every block: an age range's
sats at one rounded creation price is a cell, kept as each block's changes with
every cell checkpointed each 250 blocks (statedb's `CellHistory`). The UTXO Set
plugin produces it right after the history it derives from, applying each block
as `block_changes` orders it: age crossings first, then creations, then spends. API queries read one
block's cells (a checkpoint and at most 249 blocks of changes) and project them;
they never rebuild a UTXO set.

`Reader::state_at(end)` reconstructs an arbitrary block from the nearest matching
snapshot and its following diffs. A `Cursor` applies the same diffs to resident
state during sequential processing. These are storage operations; they know
nothing about age filters or weighting models.

`UrpdCells` is one block's distribution: each age range's sats over the price
places, with a four-byte occupancy mask per place so a projection visits populated
ages instead of checking all 23 cells. Its `project(weights, filters)` iterator
serves the API and the plugins alike. A filter is a slice of age ranges; raw,
Cointime, and Coinflow supply use the same projection. Multiple weights and
filters are projected together in one pass, rounding weighted sats only after
combining ages at each price. Raw totals remain integers. Each requested filter is
another compact age mask, so excluded ages are skipped before applying any model
weights. Statistics reuse their cumulative supply totals for density and classify
each price once across all cohorts.

`UrpdWalk` hands a plugin the cells after each block it computes, applying each
block's recorded changes. It keeps the cells after a complete walk and resumes
from them when the next walk starts where they end on the same chain; otherwise
it restores them from the history. Long backfills split into segments that walk
on the thread pool, each from its own restore; results return in block order.

Model weights can change every block, so weighted scalar analytics still visit
the distribution at each block they compute.
