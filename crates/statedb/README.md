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
  by 128 blocks and 1 MiB per producer. One larger record may exceed the window
  limit. It reads adjacent index entries and payloads together; readers never copy
  the full producer index.
- `BlockDiff` borrows fixed-width spend rows directly from the read window. Its
  cloneable iterators expose actual `spent()` rows or all `removed()` rows,
  including an overwrite correction. No decoded row vector or extra row copy is
  needed. The view remains valid until the cursor advances again.
- State application validates removals and their recorded total in the same pass.
  A failed block restores every touched cell, the prefix, hash and total. Same-block
  spends follow creation, and zero-sat outputs retain their counts.
- Snapshot opening reads identity only. Latest restoration reads the latest full
  page directly, without diff replay. Ordinary latest pages read straight into the
  `Amount` array using safe zerocopy traits; there is no intermediate byte buffer.
- Warm writers retain their state. Producer versions, prefix and block hashes
  decide whether it can be reused; failed updates discard it. No-op advances and
  commits do not rewrite files.

`History` owns reconstruction and snapshot publication. `Reader` exposes one
immutable published prefix, `state_at`, `cursor`, `replay`, and identity checks
for reusable analytical state. `View` opens read-only files for queries. The
caller holds the pipeline publication guard through state capture, excluding
snapshot writes and producer rewinds. Each plugin still owns its complete
`ComputePlugin::compute_state()` update with read-only dependencies.

## Storage

The three stores are opened from their owning plugin roots: Inputs provides
`plugins/inputs/` for `spends/`, Outputs provides `plugins/outputs/` for
`creations/`, and UTXO Set provides `plugins/utxo_set/` for
`snapshots/`. Readers accept these roots separately, so the database does not
require a shared `data/origins/` directory. Existing stores can be moved into
the corresponding owner directories and reopened without replaying the chain.

Each producer owns `data`, `index`, `commit`, and a writer-lock file. The manifest
publishes its version, record count and data end. Unpublished tails are
excluded from reads and discarded by a writer on reopen. A failed modifying
write invalidates that writer until reopen.

Full states use `snapshots/data` plus `snapshots/pages`, like a paged vector with
variable-size pages. The compact page index stores exclusive heights and offsets.
Periodic snapshots are permanent; one nonperiodic latest page replaces its
predecessor. Rewinds discard descendants. A periodic snapshot also serves as
latest when the update ends on that boundary. Ordinary latest pages are raw for
fast reads and writes; periodic pages compress dense sats and count columns
separately with PCO. The default interval is 5,000 blocks.

Periodic pages are published as they finish, followed by the final latest page.
If a block or callback fails before another write, the last completed checkpoint
remains readable and can be reopened. Failures during file modification invalidate
the snapshot writer.

All fixed-width integers are little-endian. The format has **no checksums**.
Bounds, record shapes, producer identities, checked arithmetic and state invariants
remain validated.

| Record | Layout |
| --- | --- |
| Spend | Block hash (32 bytes), total sats/count (two u64), then 16-byte rows: origin u32, count u32, sats u64 |
| Creation | Block hash (32 bytes), new sats/count (two u64), optional overwrite: origin u32, sats/count u64 |
| Producer index | Exclusive payload end u64: 8 bytes per block |
| Commit | `ORIGIN03`, version, start height (always 0), record count, data end: 40 bytes |
| Snapshot page index | `STAPAGE4`, one latest-retention byte, then height u32 and exclusive payload end u64 per page |
| Snapshot page | `STATER04` for raw or `STATEP04` for PCO; exclusive height, block hash, two producer versions, compressed sats length; payload |

Snapshot headers occupy 72 bytes. Raw payloads contain `{ sats: u64, count: u64 }`
for each creation height. Compressed payloads contain two dense u64 streams using
PCO level 3, Classic mode, and no delta transform. Creation height is implicit.
Spend counts are bounded by u32 per origin per block; state and aggregate counts
remain u64.

## Persistence contract and format changes

The owner **locks exit, writes, then unlocks exit**, using the project's exit guard
across the complete update. statedb installs no shutdown handler and owns no
application exit lock. Crashes, forced termination and power loss are unsupported;
there is no crash-recovery protocol or compatibility reader.

This format replaces the earlier varint/CRC journals and LZ4 snapshot directory.
Existing stores using the earlier format must be converted offline or rebuilt
before using them.
Conversion tooling and representative measurements are retained under the
repository's ignored `tmp/statedb-hotpath-20260930` directory. Live data is not
modified by the tests.

Fixed-width diffs deliberately favor sequential throughput over disk size. The
measured spend payloads are about 2.2–2.3 times larger than the earlier varints.
Compressed diff pages and packed rows were tested and rejected for slower reads.
Snapshot compression remains useful for permanent history; the latest page favors
latency. These are component measurements, not an end-to-end daemon speedup.
