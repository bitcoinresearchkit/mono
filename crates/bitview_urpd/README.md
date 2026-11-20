# URPD views

The durable source is `statedb`: remaining sats and UTXO counts by creation
height, as block diffs and full snapshots. Inputs and Outputs supply their
contributions during their existing scans. The UTXO Set plugin publishes their
complete prefix and owns the snapshots, every 500 blocks and at the latest
published block.

`UrpdHistory` stores sats by age range and creation-price interval. Its boundaries
include every raw, linear, and logarithmic bucket edge, so each exact creation
price belongs to one stored cell. The compact price range ends at $42,949,672.94.
A higher creation price returns an error. The UTXO Set plugin records each block's
age crossings, creations, and spends as cell changes. Its `CellHistory` journal
writes the nonzero cells every 250 blocks. A point query restores its block
from the nearer published checkpoint. This replays at most 125 blocks between two
checkpoints and at most 249 blocks at an unfinished tip.

`Reader::state_at(end)` reconstructs an arbitrary block from the nearest matching
snapshot and its following diffs. A `Cursor` applies the same diffs to resident
state during sequential processing. These are storage operations; they know
nothing about age filters or weighting models.

`UrpdCells` is one block's distribution: each age range's sats over the price
places, with a four-byte occupancy mask per place so a projection visits populated
ages instead of checking all 23 cells. Plugins project onto four-significant-digit raw prices. A query
projects onto the requested buckets, retaining the raw price for capital. Each
filter selects age ranges, and each weight scales those ranges before their
supplies are combined. Weighted supply is floored to whole sats at each raw
price. If a bucket edge splits that price, successive differences of the floored
cumulative supply allocate its sats in ascending price order. Each requested filter is
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
