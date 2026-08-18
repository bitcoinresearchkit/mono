# brk_fjall

BRK's table-only specialization of [Fjall](https://github.com/fjall-rs/fjall).
Its Rust library name remains `fjall`.

BRK sorts each indexing batch in memory and ingests it directly into immutable
LSM tables. This crate therefore contains only the pieces that workload needs:

- named keyspaces;
- direct SSTable ingestion;
- latest-version point, range, and prefix reads;
- recovery and database locking;
- background leveled compaction.

There is deliberately no journal, public memtable write path, snapshot API, or
cross-keyspace batch API. SSTable and manifest publication are synced before an
ingestion finishes.
