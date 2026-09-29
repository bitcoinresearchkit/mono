# statedb

A prototype database for historical state, stored as full snapshots and per-block
diffs. Its current state is remaining sats and UTXO counts aggregated by creation
height: `creation_height -> { sats, count }`. An **origin** means a creation
height throughout the API and format. This is not an outpoint database: it cannot
identify individual UTXOs, scripts, or owners. It depends on no other Bitview
crate or vecdb and stores no prices, cohorts, or derived analytics.

Each block has two producer contributions:

- `Spends`: sats and UTXO counts removed from each creation height.
- `Creations`: new spendable sats and UTXOs, plus an optional historical overwrite
  correction kept separate from actual spends.

`History` joins those contributions, checks their block hashes, applies them to a
`State`, and writes full snapshots every 5,000 blocks and at the latest published
block. Height arguments are exclusive prefixes: `reader.state_at(700_001)` returns
state after block 700,000. A seeded history cannot answer before its seed.

Inputs writes `Spends` while resolving input values. Outputs writes `Creations`
while already scanning new outputs, excluding OP_RETURN and genesis outputs and
handling historical overwrites. Neither needs to scan outputs again to produce
these records.

The `bitview_plugin_utxo_history` stage publishes history once both contributions
are committed, writing global supply/count from the same state update. Outputs
owns neither snapshots nor combined supply accounting. The pipeline assembles a
single `Reader` for Age and query code; consumers know nothing about the producer
plugins or the two underlying files. Prices, timestamps, and weighting models
belong to those consumers.

`Reader` borrows an immutable published prefix. It exposes `state_at`,
`read_block` into a reusable `BlockDiff`, and state `replay`. It rejects
unpublished ranges and a published tip whose hash no longer matches either
producer. Both snapshot replay and consumer iteration use the same decoder and
hash checks. The block record owns its reusable buffers; consumers keep their
ordinary loops without seeing encoding or files. There is no merged log or
allocation of a full batch of records.

`BlockDiff::spent()` excludes historical overwrites; `removed()` includes them
for state accounting. Both are slices of one reused buffer, keeping the hot
loops over removals simple without duplicating records.

## Files and recovery

Each producer uses an append-only data file, a fixed-width offset/checksum index,
and an atomically replaced commit manifest. Reopening discards unpublished tails
and reports incomplete or corrupt committed data. A failed write invalidates the
writer until it is reopened. One process may write each producer column.

Snapshots include the prefix, final block hash, producer versions, and a checksum.
A rewind publishes the recovered ancestor before removing descendant snapshots.
Queries load the nearest matching snapshot and replay the suffix. Zero-sat
origins with remaining UTXOs are preserved. Same-block spends apply after creation.

The diff encoding retains producer order and uses unsigned varints.
Snapshots compress dense sats and count arrays independently with LZ4;
heights are implicit. When a periodic snapshot is also latest, both names share
its immutable file; replacing either uses a new file and atomic rename.

## Binary layout and tradeoffs

Fixed-width integers are little-endian; varints are canonical unsigned LEB128.
Each producer has its own `data`, `index`, and `commit` files. Record lengths come
from consecutive index offsets, so variable rows need neither padding nor a
stored row count.

| File or record | Encoding |
| --- | --- |
| Spend record | Block hash (32 bytes), total sats/count (two u64), then `(origin, sats, count)` varints to the end of the record |
| Creation record | Block hash (32 bytes), new sats/count (two u64); optional overwrite correction `(origin: u32, sats: u64, count: u64)` |
| Index entry | Exclusive end offset (u64) and record CRC32 (u32): 12 bytes per block |
| Commit manifest | `ORIGIN02`, producer version, base height, record count, data end (four u64), CRC32: 44 bytes |
| Full snapshot | `ORGSNAP2`, exclusive height, final block hash, two producer versions, compressed sats length; LZ4 sats and count sections; CRC32 |

Snapshot headers occupy 72 bytes. Each decoded section has one u64 per creation
height. Separate sections keep like values together for compression; height is
the array index. Zero sats with nonzero count must be retained. The two producer
hashes detect mismatched chains when joining contributions; CRCs detect damaged
records and snapshots. Format identifiers and the `origins` data directory are
unchanged by the crate rename.

Tests at 700k, 800k, and 900k compared these encodings with sorted delta origins
and sparse snapshots. Sorting saved about 26% of diff bytes but nearly doubled
append time. Sparse snapshots saved about 26% and wrote faster but restored
slower. The retained format favors direct appends and state restoration, without
optional codecs. A 5,000-block interval reduced snapshot bytes by about 73%
against 1,000-block spacing in the measured 21,000-block corpora, with roughly
90–121 ms warm restoration near the interval boundary. These are measured
tradeoffs, not a claim that the format is optimal for every workload.

## Prototype boundary

Sequential writers retain their validated tip in memory and reuse it for appends.
A no-op does not rewrite snapshots. Reorgs, producer-version changes, or failed
updates discard the resident state before recovery. Queries use a separately
opened read-only `View`, validating the published prefix and producer hashes.
There is no segment rotation or sats-only decode path. Files use Unix positional
reads. The prototype default graph and URPD HTTP endpoints use this history;
URPD price distributions are reconstructed and are not persisted.

Integration tests cover replay, reorgs, restart, incomplete writes, corruption,
producer mismatch, zero-value outputs, and historical corrections. They do not
constitute a complete power-loss or production-storage audit.
