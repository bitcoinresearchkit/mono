# brk_lsm_tree

BRK's table-only log-structured merge tree. Its Rust library name remains
`lsm_tree`.

The crate accepts strictly sorted batches, writes them directly to immutable
tables, and atomically publishes the latest table layout. A `ReadView` pins one
published layout for reads: later ingestions and compactions stay invisible to
it, and its tables stay on disk while it lives (drop views before reopening the
tree). It intentionally has no memtable, write-ahead journal, transactions,
sequence-number snapshots, or generic compaction-policy API; BRK supplies the
only writer (one ingestion at a time per tree) and uses fixed leveled
compaction.

This is a specialized fork of
[`lsm-tree`](https://github.com/fjall-rs/lsm-tree), maintained for the
[Bitcoin Research Kit](https://bitcoinresearchkit.org).
