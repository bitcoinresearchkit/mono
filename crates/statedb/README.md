# statedb

UTXO state aggregated by creation height: `height -> { sats, count }`. This is
not an outpoint database. It stores no prices, cohorts, or derived analytics and
depends on neither vecdb nor another Bitview crate.

Inputs writes grouped `Spends` while resolving input values. Outputs writes
`Creations` while scanning new outputs, excluding unspendable outputs and keeping
historical overwrite corrections separate from actual spends. UTXO Set joins
these two contributions, checks their block hashes, updates the canonical `State`,
and publishes snapshots plus global supply/count. Neither producer rescans outputs.

Height arguments are exclusive prefixes: `reader.state_at(700_001)` returns the
state after block 700,000. Every history starts at genesis.

## Read and update paths

- `Cursor` borrows the consumer's state and reads sequentially in windows bounded
  by 128 blocks and 1 MiB per producer. Each window also holds the previous
  record, and one larger record may exceed the window limit. It reads adjacent index entries and payloads together; readers never copy
  the full producer index.
- `BlockDiff` borrows fixed-width spend rows directly from the read window. Its
  cloneable iterators expose actual `spent()` rows or all `removed()` rows,
  including an overwrite correction. No decoded row vector or extra row copy is
  needed. The view remains valid until the cursor advances again.
- State application trusts rows the producers validated: removals, their recorded
  total and overflow are debug assertions. Framing, I/O and producer-identity errors
  stop a block before it touches the state. Each block must continue the state's
  chain (its spends parent hash equals the state hash, both read in one window), so
  a reader racing a rewind errors instead of applying another chain's rows. Same-block spends follow creation, and
  zero-sat outputs retain their counts.
- Snapshot opening reads identity only. Restores pick snapshots by indexed height
  before opening any, and readers validate tip identity once. Latest restoration reads the tip's
  pages directly, without diff replay: in page-file order, in bounded spans across small runs of
  stale slots (one span buffer through safe zerocopy traits), copying each page into place.
- Warm writers retain their state. Producer versions, prefix and block hashes
  decide whether it can be reused; failed updates discard it. No-op advances and
  commits do not rewrite files.

`History` owns reconstruction and snapshot publication. `Reader` exposes one
immutable published prefix, `state_at`, `cursor`, `diffs`, and identity checks
for reusable analytical state. `View` opens read-only files for queries: retained
snapshots are only appended and their index replaced by rename, and the tip
snapshot (with the one it replaced, for readers of the previous publication) is
replaced by rename too, so an open view never sees a write. The caller keeps
producers from rewinding below what it reads. Each plugin owns its complete
`ComputePlugin::compute_state()` update with read-only dependencies.

## Storage

The three stores are opened from their owning plugin roots: Inputs provides
`plugins/inputs/` for `spends/`, Outputs provides `plugins/outputs/` for
`creations/`, and UTXO Set provides `plugins/utxo_set/` for
`snapshots/`. Readers accept these three roots separately.

Each producer owns `data`, `index`, `commit`, and a writer-lock file. The manifest
publishes its version, record count and data end. Unpublished tails are
excluded from reads and discarded by a writer on reopen. A failed modifying
write invalidates that writer until reopen.

Periodic full states use `snapshots/data` plus `snapshots/pages`, like a paged
vector with variable-size pages; the compact page index stores exclusive heights
and offsets and is replaced by rename. Periodic snapshots are permanent (rewinds
discard descendants) and compress dense sats and count columns separately with
PCO. The default interval is 5,000 blocks. An older layout's latest page inside
`data` is dropped on open.

Every update publishes its final state as the tip, raw for fast reads and writes:
`snapshots/tip` is a page directory, replaced by rename, into the page file
`snapshots/tips.<generation>` of immutable slots of 256 creation heights (4 KiB).
A tip appends only the pages its update changed and points the rest at the
previous tip's slots; once the page file would exceed four times the live pages,
the next tip starts a fresh generation. The tip it replaced stays as
`snapshots/prev` for readers of the previous publication, and page files neither
uses are removed. Tips of an older format are dropped on open and rebuilt by the
next update; restores use the periodic snapshots meanwhile.

Periodic snapshots are published as they finish, followed by the tip.
If a block or callback fails before another write, the last completed checkpoint
remains readable and can be reopened. Failures during file modification invalidate
the snapshot writer.

All fixed-width integers are little-endian. The format has **no checksums**:
record framing, bounds and producer identities are checked on read, while replay
arithmetic and state invariants are debug assertions.

| Record | Layout |
| --- | --- |
| Spend | Block hash (32 bytes), total sats/count (two u64), then 16-byte rows: origin u32, count u32, sats u64 |
| Creation | Block hash (32 bytes), new sats/count (two u64), optional overwrite: origin u32, sats/count u64 |
| Producer index | Exclusive payload end u64: 8 bytes per block |
| Commit | `ORIGIN03`, version, start height (always 0), record count, data end: 40 bytes |
| Snapshot page index | `STAPAGE4`, one latest-retention byte, then height u32 and exclusive payload end u64 per page |
| Periodic snapshot | `STATEP04`, exclusive height, block hash, two producer versions, compressed sats length; payload |
| Tip directory | `STATEV05`, exclusive height, block hash, two producer versions, page file generation; then a slot u32 per 256 heights |
| Tip page file | Slots of 256 raw `{ sats: u64, count: u64 }` amounts (a final page may be shorter) |

Snapshot and tip headers occupy 72 bytes. Periodic payloads contain two dense u64
streams using PCO level 3, Classic mode, and no delta transform. Creation height
is implicit.
Spend counts are bounded by u32 per origin per block; state and aggregate counts
remain u64.

## Persistence contract

The owner **locks exit, writes, then unlocks exit**, using the project's exit guard
across the complete update. statedb installs no shutdown handler and owns no
application exit lock. Crashes, forced termination and power loss are unsupported;
there is no crash-recovery protocol or compatibility reader.

Stores in an unsupported format must be rebuilt.

Fixed-width diffs deliberately favor sequential throughput over disk size. The
measured spend payloads are about 2.2–2.3 times larger than the earlier varints.
Compressed diff pages and packed rows were tested and rejected for slower reads.
Snapshot compression remains useful for permanent history; the latest page favors
latency. These are component measurements, not an end-to-end daemon speedup.
