# rawdb design experiments — 2026-09-19

The strongest supported direction is a small public API around bulk region writes,
stable reads, capacity reservation, and an explicit database checkpoint. Keep
selective asynchronous writeback internally. The experiments exposed important
consumer contracts and rejected several apparently simpler implementations.

Production crate sources were not changed. All implementations, fault injection,
and additional tests ran in isolated copies. The complete evidence and replay
instructions are in [the experiment artifact](../benches/rawdb/2026-09-19-design/README.md).

**The immediate priority is predictable failure and checkpoint behavior.** Three
targeted probes reproduced problems that the ordinary suites did not expose:

1. `vec.write(); vec.flush()` can leave a dirty region unsynchronized. A header-only
   `vec.flush()` can do the same: the default method only synchronizes when its
   *current* `write()` returns `true`.
2. Injecting a storage-write error after buffering another 1,024 values leaves both
   raw and PCO vectors with zero pending values. Retrying returns `Ok(false)` and
   reopening finds only the original 1,024 values. Retry behavior needs an explicit
   contract: preserve the pending operation, or enter an error state that prevents
   apparent successful continuation.
3. Injecting failure in PCO page-metadata writing leaves the in-memory length at
   2,048, retry returns `Ok(false)`, and reopening reports a corrupted region.
   Publishing length and consuming pending values before all related writes have
   succeeded is unsafe for a retryable API.

These are recorded in [failure-probes.json](../benches/rawdb/2026-09-19-design/failure-probes.json).
Relevant production code is in `vecdb/src/traits/any_stored.rs` and the raw and
compressed `inner/read_write/any_stored_vec.rs` implementations. Fixing them must
preserve successful-write buffer release; keeping every vector's maximum buffer
capacity would increase steady-state memory across the application.

**Measured results changed the proposed simplifications.** Times below are medians
of seven measured repetitions. Comparisons within a row use the same phase and
workload. Complete minima, maxima, and individual observations are retained.

| Experiment | Reference | Candidate | Interpretation |
| --- | ---: | ---: | --- |
| Remove dirty ranges and asynchronous data writeback: 64 MiB parallel bulk checkpoint | 32.39 ms | 42.77 ms | About 32% slower; reject this approach on this host |
| One asynchronous flush of the entire mapping: same bulk workload | 32.39 ms | 33.54 ms | Restores most of the lost performance |
| Same whole-mapping approach: 4 MiB written across a 256 GiB sparse file | 11.89 ms | 105.01 ms | About 8.8× slower; reject as the default |
| Track ranges in 4 KiB allocation units: fragmented individual updates | 137.01 ms | 61.47 ms | About 55% less time; avoids redundant writeback requests |
| Same page-range candidate: parallel bulk checkpoint | 33.21 ms | 33.17 ms | Essentially unchanged |
| Same page-range candidate: raw vector checkpoints | 81.88 ms | 80.66 ms | No persuasive general throughput improvement |
| Same page-range candidate: 256 GiB sparse-file synchronization | 15.22 ms | 15.14 ms | Preserves selective writeback behavior |
| Same page-range candidate: tiny parallel appends | 121.24 ms | 130.45 ms | Higher median; overlapping ranges of observations, worth retaining as a regression check |
| Reserve known capacity before interleaved growth | 52.14 ms | 32.64 ms | About 37% less time, including the reservation work |
| Publish after sync, reclaim afterward | 13.80 ms | 5.24 ms | Shorter publication-critical interval; reclamation still costs work |

The fragmented workload uses individual overwrites. Current mutable vectors already
submit ordered batches, so its improvement is **not** a demonstrated 55% improvement
to normal indexing. The 4 KiB units are rawdb's allocation granularity, not a claim
about the host's hardware page size.

The page-range change is the best narrowly scoped rawdb optimization found here.
It retains the existing algorithm and API, coalesces nearby writes earlier, and
keeps distant extents separate. The tiny-append result prevents calling it a
universal speed improvement. Ordinary bulk and vector workloads remained close to
baseline. Its fragmented-update peak RSS also fell from about 79.3 to 74.5 MiB.

**Precise writeback information has a real purpose.** A vector header and its newly
appended tail can be far apart. Large reservations can also leave substantial gaps
between regions. A single database flag loses this information. The global-flag
prototype made 1,000 clean flushes over 2,048 regions fall from about 5.8 ms to
0.026 ms, but the resulting writeback alternatives regressed meaningful dirty
workloads. This does not justify replacing the existing range tracking.

The operating-system behavior must remain explicit. Apple's documentation describes
`msync` as writing modified mapped pages; Linux documents `MS_ASYNC` as a no-op since
2.6.19. Rust 1.98.1's installed Apple implementation of `File::sync_all` uses
`F_FULLFSYNC`. The timings above are local Darwin measurements, not a portability
argument. [Apple msync documentation](https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/msync.2.html),
[Linux msync documentation](https://www.man7.org/linux/man-pages/man2/msync.2.html),
[Apple synchronization guarantees](https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/fsync.2.html).

**Capacity planning gave the clearest combined speed and memory benefit.** The
interleaved-growth fixture writes 64 MiB across 16 regions. Reserving their eventual
capacity first eliminated 63 MiB of relocation copies and reduced peak RSS from
129.6 to 66.6 MiB. Sparse reservation itself does not touch all the reserved bytes.
The production indexer already makes substantial initial reservations; this result
supports keeping and using that capability where growth is predictable. It is not
evidence of an additional 37% gain for the existing indexer.

**Compressed-write changes primarily benefited memory.** For the high-entropy PCO
fixture, measured before reopening and verification:

| Output assembly | Peak RSS through writing | Checkpoint time |
| --- | ---: | ---: |
| Existing concatenation, per-region-flag parent | 130.2 MiB | 107.54 ms |
| Preallocate the concatenation buffer exactly | 123.7 MiB | 108.99 ms |
| Copy encoded chunks directly into one reserved region write | 113.2 MiB | 108.81 ms |

Direct chunks saved about 17 MiB, or 13%, against their actual parent implementation,
without a demonstrated throughput gain. They did not show the same memory benefit
for highly compressible monotonic data. Exact preallocation is the smaller change;
a chunked region-write operation is justified when encoded-output memory matters.
If adopted, it should validate the total length, reserve once, hold the write locks
once, copy all slices, and publish the resulting length once. A loop of independent
public append operations would lose those properties.

**Deferred metadata is a checkpoint-contract change.** Keeping bounds only in RAM
until synchronization caused 18 failures in existing vecdb tests and one indexer
rollback/reopen failure. A separate experiment fixed the default vector `flush()`;
nine vecdb failures disappeared. The remaining nine cases reopen after writes or
compute without an explicit database synchronization. The indexer failure follows
the same pattern. Existing tests were kept to expose these assumptions.

Deferred bounds did not show a convincing throughput gain in the normal vector
checkpoint workloads. It should follow a deliberate application-wide checkpoint
contract, rather than lead the optimization work. Cold creation and deletion were
still written immediately in this prototype; it did not implement transactional
metadata, a new format, or a WAL.

**Mapping generations solve a specific latency problem.** With a reader held for
100 ms, an unrelated growth operation took about 112 ms with the current mapping
lock and 2.8 ms with an `Arc`-owned mapping generation; all seven generation runs
finished while the reader remained alive. Existing readers retained stable bytes,
and same-region write exclusion remained tested. Creating a million readers rose
from 11.76 to 12.55 ms; repeated reads were essentially unchanged.

This is useful evidence for independent-region concurrency, not a 40× scan-speed
claim. Old mappings remain alive until their readers are dropped. Many simultaneous
long-lived generations and their virtual-memory/page-table costs were not measured.
Capacity planning should precede adding this machinery solely to solve avoidable
growth stalls.

**Recommended implementation order:**

1. Establish the write-error contract and fix vector synchronization. Do not let a
   failed write become an apparently clean retry. Advance published lengths only
   when their data and page metadata agree. Test raw, compressed, raw-tail, header-only,
   and rollback paths together.
2. Put checkpoint scheduling and completion in the runtime. One checkpoint owner
   writes the relevant vectors, synchronizes their databases and other required
   state, then publishes the checkpoint. Reclamation follows publication when the
   application permits it. Shutdown must join jobs and surface errors explicitly.
   A rawdb sync alone does not cover store markers or rollback files.
3. Keep range-aware writeback and take the page-granularity candidate into the next
   implementation review. Preserve sparse-file and tiny-append benchmarks. No
   replacement dirty-tracking architecture is justified by these results.
4. Apply existing capacity reservation consistently before readers are pinned and
   before predictable bulk growth. Measure relocation bytes as well as elapsed time.
5. Pre-size compressed output first; adopt a chunked bulk-write capability where the
   memory reduction warrants it. Release temporary buffers after successful writes.
6. Simplify ownership and exposure around the resulting contract. Background job
   management belongs with runtime scheduling. A region handle should have a clear
   storage lifetime; separating internal region records from owning handles can
   remove the weak-database dependency without creating an ownership cycle.
7. Revisit deferred bounds and mapping generations with those contracts established.
   Their measured benefits and behavior changes are different and should remain
   separate decisions.

The target public operations are: open/resolve regions, reserve capacity, acquire
stable reads, replace a suffix, overwrite an ordered batch, synchronize a database,
and reclaim space. Dirty-range details, physical offsets, metadata locks, file
handles, and background scheduling stay internal or behind a narrow read capability.
Database synchronization should return `Result<()>`; dirty-region counts are not
part of the application's required contract. Keep both mapped and buffered reading
available behind that capability. The exact read-handle redesign was not benchmarked
here and should preserve direct reads into caller-provided buffers.

**Validation and limits.** The unchanged production rawdb/vecdb suites passed 554
tests/doctests. Each prototype received the complete suites, the three normally
ignored concurrency stress tests, deterministic data/metadata sync failures, and a
randomized region model. Ten configurations also received five process-kill/reopen
checks each. There were 1,043 measured harness runs across 17 workloads, plus 149
warmups and 45 iterations of the existing storage benchmark. The page-range
candidate passed 558 regular tests/doctests and the three extended stress tests;
its indexer, runtime, and default-plugin integration checks passed 87 tests, with
one documentation example ignored. The metadata failures above remain recorded failures.

Measurements used eight Rayon workers, native CPU code generation, fat LTO, one
codegen unit, and an unwind release profile shared by all compared implementations.
The isolated regular test profile used optimization level 1 without LTO or debug
info. Each candidate had an isolated Cargo target and verified binary identity;
preliminary shared-target runs were discarded after cache reuse was detected.
No builds or tests were intentionally run alongside the reported timed experiments.

Process-kill/reopen and injected errors do not prove power-loss consistency. Linux
and FreeBSD runtime behavior, cold-disk scans, and a complete historical indexer
rebuild were not benchmarked. The results support the specific choices above; they
do not establish a whole-project speedup or prove a globally optimal design.
