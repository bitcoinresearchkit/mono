# brk_fjall

BRK's table-only specialization of [Fjall](https://github.com/fjall-rs/fjall).
Its Rust library name remains `fjall`.

BRK sorts each indexing batch in memory and ingests it directly into immutable
LSM tables. This crate therefore contains only the pieces that workload needs:

- named keyspaces;
- direct SSTable ingestion;
- latest-version point, range, and prefix reads;
- per-keyspace read views that pin a table layout for later reads;
- recovery and database locking;
- background leveled compaction.

There is deliberately no journal, public memtable write path, cross-keyspace
snapshot, or cross-keyspace batch API. Each keyspace takes one ingestion at a
time. Keyspaces, read views, and returned iterators hold the database lock, so
no reopen can delete the tables they pin. SSTable and manifest userspace buffers
are flushed before an ingestion finishes; storage-device barriers are not
required.

The fork supports only its current `FORMAT_VERSION`. Older databases must be
rebuilt; there is no format migration path.
