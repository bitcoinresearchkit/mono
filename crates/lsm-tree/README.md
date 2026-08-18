# brk_lsm_tree

BRK's table-only log-structured merge tree. Its Rust library name remains
`lsm_tree`.

The crate accepts strictly sorted batches, writes them directly to immutable
tables, and atomically publishes the latest table layout. It intentionally has
no memtable, write-ahead journal, transactions, snapshots, or generic
compaction-policy API; BRK supplies the only writer and uses fixed leveled
compaction.

This is a specialized fork of
[`lsm-tree`](https://github.com/fjall-rs/lsm-tree), maintained for the
[Bitcoin Research Kit](https://bitcoinresearchkit.org).
