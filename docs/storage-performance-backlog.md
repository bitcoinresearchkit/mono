# Storage performance backlog

This file records plausible Fjall, LSM-tree, and ByteView optimizations that are
not proven wins for Bitview. None should enter the production configuration
without an isolated benchmark against the current conservative baseline.

## Contract and baseline

- Reads are more important than writes, but an optimization must not materially
  regress the mature indexing tail, write amplification, disk usage, or memory.
- Bitview must remain consistent after an orderly Ctrl+C shutdown. Recovery
  after a process crash, kernel crash, or power loss is not required.
- BRK omits per-block table-data and manifest checksums and does not support
  older checksummed storage formats. The existing archive table-of-contents
  checksum remains because it is only read during table recovery, not point
  reads; replacing that archive format is not a proven optimization.
- Keep the established 64 MiB compaction target, 4 KiB blocks, compression and
  restart policies, cache behavior, descriptor limit, and worker count unless a
  candidate below proves a better result.

The previously tested bundle of storage tuning is rejected as a baseline: in
the closest full Bitview comparison it was about 2.1% slower overall and wrote
about 14.2% more bytes. Its 650k-800k tail was about 13.7% slower than the
historical M3 Pro v0.12 trace and wrote about 27% more bytes. Microbenchmarks
that favored individual pieces did not predict that integrated result.

## Promotion gate

Use the production release profile and real default Bitview plugin schedule on
the same machine, Bitcoin block source, target height, cache state, and empty
plugin directory. Record the exact revision, binary hash, configuration, host,
completion state, per-plugin timings, process I/O, physical database size,
memory, and compaction debt.

For layout, compaction, or write-path changes, run independent empty-database
builds through the same target height. Always inspect at least 0-650k,
650k-700k, 700k-750k, and 750k-800k separately. Reverse A/B execution order or
repeat it so thermal and filesystem-cache order cannot decide the result. A
read-path-only change may additionally use identical copied datasets for
cached, OS-hot/cache-cold, and cold point reads plus representative ranges.

A candidate is promoted only when its benefit repeats outside run-to-run noise
and the full Bitview run confirms that its resource costs are acceptable. Test
one independent variable at a time before testing combinations.

The committed `benches/bitviewd/v0.12_m3pro_int_ssd` run is historical context,
not a substitute for a fresh control built from the same source base.

## Candidates

| Candidate | Hypothesis and main risk | Closest required benchmark |
| --- | --- | --- |
| 128, 192, or 384 MiB compaction targets | Fewer tables and less metadata versus larger rewrites, stalls, and transient space | Separate full Bitview builds; tail time, write bytes, peak disk, debt, and read passes |
| 2 KiB blocks by keyspace kind | Less point-read I/O versus larger indexes, more block work, and worse writes | Change one kind at a time; full build plus cached, OS-hot/cache-cold, and cold reads |
| Data restart intervals 4, 8, 10, or 16 by kind | Less key decoding versus larger tables | Change one kind at a time; full build, table size, point reads, and ranges |
| L0 threshold 6, 8, or 10 | Fewer compactions versus higher read amplification and debt | Full Bitview builds, especially 650k-800k; reads during and after build |
| Eager sub-threshold L0 premerge | Reduce future overlap versus extra foreground write amplification | Isolated full build against the same control; do not infer from synthetic ingestion |
| Level ratio 6 or 16 | Alter table fanout and rewrite frequency | Full build; mature tail, write bytes, table counts, and read latency |
| One or two compaction workers | Return CPU to Bitview versus accumulating debt and read amplification | Full build with production plugin concurrency; tail, debt, and post-build reads |
| Cache size 1, 2, or 4 GiB | Trade block-cache hits against memory available to Bitview and the OS cache | Full build and repeated read passes; RSS, page faults, cache hit rate, and wall time |
| Cache hot/ghost allocation | Improve admission for recurring point reads versus wasting capacity on metadata | Identical-dataset cached and OS-hot/cache-cold traces, then a full build |
| Descriptor cache sizing or pinning | Avoid open/close work versus consuming file descriptors and memory | Point/range reads over a mature database; descriptor misses, open calls, and limits |
| Mmap or cache bypass for selected blocks | Avoid copies and cache lookup overhead versus page faults, RSS, and lifetime complexity | Cold, OS-hot/cache-cold, and cached reads plus ranges on a mature copied database |
| More fixed-width entry fields | Save length/tag bytes and decoding versus a new format and branch complexity | Per-keyspace table corpus, then independent full builds; size, reads, writes, and CPU |
| Disable LZ4 for selected levels or kinds | Save compression CPU versus more physical I/O, cache pressure, and disk | Independent full builds; wall time, bytes written/read, physical size, and reads |
| Data-block hash indexes | Faster in-block points versus table size and build cost; current BRK policy is disabled | Enable only for a selected kind/level; point reads, full build, and physical size |
| Index/filter pinning by kind or level | Avoid cache misses versus permanently occupied memory | Mature-database point/range traces plus full build RSS and cache behavior |
| Preserve table-ID sets or reuse version work during compaction choice | Reduce allocator/scanning CPU versus stale bookkeeping and complexity | Compaction-heavy LSM trace first, then production tail with profiles |
| Reduce copies or allocations in ByteView builders and fused keys | Lower hot-path CPU versus larger representations or slower reads | ByteView benches using actual BRK key/value length distributions, then full Bitview |
| Batch-cache policy and size | Reuse decoded values versus memory pressure and delayed reclamation | Production plugin schedule; per-plugin time, RSS, allocation profile, and full wall time |

When a candidate is tested, add its exact patch, commands, raw result path, and
decision here. Keep rejected results so they are not accidentally rediscovered.
