# Crate audit

Started: 2026-09-18

## Objective

Audit every workspace crate, one by one, and make justified improvements toward
extremely clean, KISS, DRY, predictable, consistent, idiomatic Rust. Rust is fast
by default: concentrate performance engineering on measured hot loops and the
work they trigger. Keep ordinary code straightforward.

This document is the source of truth for audit order, progress, findings,
decisions, and validation. Update it as each crate is reviewed, including when
work stops partway through a crate.

## Review standards

- Prefer the smallest clear implementation, ordinary Rust idioms, and explicit
  ownership and invariants. An abstraction must simplify its callers and the
  overall design.
- Give each struct one cohesive responsibility. Keep stored data separate from
  runtime coordination, and remove unused APIs, state, and layers after checking
  their callers and invariants. Group related types and helpers under their
  owning module directory, keeping the crate root small.
- Keep one source of truth. Consolidate duplicated behavior and fix shared
  contracts instead of adding local exceptions. Similar-looking code only
  belongs together when it represents the same responsibility.
- Make APIs, naming, errors, bounds, and edge cases predictable and consistent
  across crates. Review public consumers before changing or removing an API.
- Expose only what external consumers need. Audit root re-exports, modules,
  types, fields, methods, and signatures; keep implementation helpers private or
  crate-visible. Follow re-exports and feature-gated callers before narrowing.
- Review unused methods, types, dependencies, and features individually:
  remove them, reuse them to eliminate duplication, or simplify the surrounding
  design. Check real callers, optional features, examples, and downstream use.
- Prefer functions, types, and composition over macros. Retain macros when
  substantial repetition reduction or consistency benefits justify them.
- Use module-level `use` imports and short names; retain qualified paths needed
  for exported macro hygiene. Follow the BRK convention of one public struct
  per file when writing or touching public struct definitions, including
  `pub(crate)` structs.
- Audit correctness alongside clarity: unsafe invariants, lifetimes, locking,
  concurrency, overflow, failure handling, persistence, recovery, and resource
  cleanup where applicable. Documentation must match actual guarantees.
- Optimize measured hot paths. First examine algorithms, data layout, repeated
  work, allocations, copying, I/O, and lock contention inside those paths. Keep
  specialized code, caches, unsafe optimizations, and tuning only when their
  measured benefit justifies the complexity and memory cost. Prefer the simpler
  implementation when performance is equivalent or better.

Each cycle runs these passes in order: **1. Cleanliness, 2. KISS, 3. DRY,
4. Idiomatic Rust, 5. Speed, 6. Consistency, 7. Naming/architecture/organization,
8. Exposability**. Complete scan → analysis → relevant tests/benchmarks → justified
fixes → verification for one pass before starting the next. Record an unchanged
outcome when no improvement is justified.

## Scope and order

The initial inventory contains **71 crates under `crates/` and one example
package**, verified with `cargo metadata --offline --no-deps --format-version 1`
on 2026-09-18. Include each crate's source, manifest, features, build scripts,
tests, benchmarks, examples, and documentation where present. The local forks
are part of the audit; consider their upstream compatibility and maintenance
cost when changing them.

Start with **`rawdb`**. It has no internal crate dependencies and provides the
storage foundation for `vecdb`. Then review dependencies before their consumers,
including optional and build dependencies. Development dependencies inform
integration checks rather than determining this order. Independent utilities
are grouped by purpose. Recheck the inventory and order if the workspace changes.

Status: **Pending**, **Active**, **Blocked**, or **Done**. A blocked crate needs a
concrete blocker and next action in its progress entry. Reopen a completed crate
when later work invalidates its reviewed contracts.

### 1. Storage foundations and shared utilities

| Order | Crate | Status |
|---|---|---|
| 1 | [`rawdb`](../crates/rawdb) | Cleanup passes done; design experiments reviewed, implementation pending |
| 2 | [`brk_exit`](../crates/brk_exit) | Pending |
| 3 | [`pco`](../crates/pco) (`brk_pco`) | Pending |
| 4 | [`vecdb_derive`](../crates/vecdb_derive) | Pending |
| 5 | [`vecdb`](../crates/vecdb) | Pending |
| 6 | [`byteview`](../crates/byteview) (`brk_byteview`) | Pending |
| 7 | [`lsm-tree`](../crates/lsm-tree) (`brk_lsm_tree`) | Pending |
| 8 | [`fjall`](../crates/fjall) (`brk_fjall`) | Pending |
| 9 | [`rangeindex`](../crates/rangeindex) | Pending |
| 10 | [`quickmatch`](../crates/quickmatch) | Pending |
| 11 | [`importmap`](../crates/importmap) | Pending |
| 12 | [`brk_alloc`](../crates/brk_alloc) | Pending |
| 13 | [`brk_logger`](../crates/brk_logger) | Pending |

`brk_exit`, `pco`, and `vecdb_derive` precede `vecdb` because it depends on them.
The other storage branch runs from `byteview` through `lsm-tree` to `fjall`.

### 2. BRK domain types and Bitcoin services

| Order | Crate | Status |
|---|---|---|
| 14 | [`brk_error`](../crates/brk_error) | Pending |
| 15 | [`brk_types`](../crates/brk_types) | Pending |
| 16 | [`brk_store`](../crates/brk_store) | Pending |
| 17 | [`brk_rpc`](../crates/brk_rpc) | Pending |
| 18 | [`brk_reader`](../crates/brk_reader) | Pending |
| 19 | [`brk_iterator`](../crates/brk_iterator) | Pending |
| 20 | [`brk_oracle`](../crates/brk_oracle) | Pending |
| 21 | [`brk_mempool`](../crates/brk_mempool) | Pending |
| 22 | [`brk_fetcher`](../crates/brk_fetcher) | Pending |
| 23 | [`brk`](../crates/brk) | Pending |

### 3. Bitview types, computation, and plugin infrastructure

| Order | Crate | Status |
|---|---|---|
| 24 | [`bitview_types`](../crates/bitview_types) | Pending |
| 25 | [`bitview_catalog`](../crates/bitview_catalog) | Pending |
| 26 | [`bitview_traversable_derive`](../crates/bitview_traversable_derive) | Pending |
| 27 | [`bitview_traversable`](../crates/bitview_traversable) | Pending |
| 28 | [`bitview_cohort`](../crates/bitview_cohort) | Pending |
| 29 | [`bitview_collections`](../crates/bitview_collections) | Pending |
| 30 | [`bitview_transforms`](../crates/bitview_transforms) | Pending |
| 31 | [`bitview_compute`](../crates/bitview_compute) | Pending |
| 32 | [`bitview_vecs`](../crates/bitview_vecs) | Pending |
| 33 | [`bitview_plugin`](../crates/bitview_plugin) | Pending |
| 34 | [`bitview_runtime_derive`](../crates/bitview_runtime_derive) | Pending |
| 35 | [`bitview_runtime`](../crates/bitview_runtime) | Pending |

### 4. Indexing and analytics plugins

| Order | Crate | Status |
|---|---|---|
| 36 | [`bitview_plugin_indexer`](../crates/bitview_plugin_indexer) | Pending |
| 37 | [`bitview_plugin_mappings`](../crates/bitview_plugin_mappings) | Pending |
| 38 | [`bitview_plugin_blocks`](../crates/bitview_plugin_blocks) | Pending |
| 39 | [`bitview_plugin_price`](../crates/bitview_plugin_price) | Pending |
| 40 | [`bitview_plugin_inputs`](../crates/bitview_plugin_inputs) | Pending |
| 41 | [`bitview_plugin_outputs`](../crates/bitview_plugin_outputs) | Pending |
| 42 | [`bitview_plugin_transactions`](../crates/bitview_plugin_transactions) | Pending |
| 43 | [`bitview_plugin_mining`](../crates/bitview_plugin_mining) | Pending |
| 44 | [`bitview_plugin_pools`](../crates/bitview_plugin_pools) | Pending |
| 45 | [`bitview_plugin_distribution`](../crates/bitview_plugin_distribution) | Pending |
| 46 | [`bitview_plugin_coinflow`](../crates/bitview_plugin_coinflow) | Pending |
| 47 | [`bitview_plugin_cointime`](../crates/bitview_plugin_cointime) | Pending |
| 48 | [`bitview_plugin_bedrock`](../crates/bitview_plugin_bedrock) | Pending |
| 49 | [`bitview_plugin_supply`](../crates/bitview_plugin_supply) | Pending |
| 50 | [`bitview_plugin_market`](../crates/bitview_plugin_market) | Pending |
| 51 | [`bitview_plugin_capital_sentiment`](../crates/bitview_plugin_capital_sentiment) | Pending |
| 52 | [`bitview_plugin_indicators`](../crates/bitview_plugin_indicators) | Pending |
| 53 | [`bitview_plugin_investing`](../crates/bitview_plugin_investing) | Pending |
| 54 | [`bitview_plugin_constants`](../crates/bitview_plugin_constants) | Pending |
| 55 | [`bitview_plugin_op_return`](../crates/bitview_plugin_op_return) | Pending |
| 56 | [`bitview_plugin_rarity_meter`](../crates/bitview_plugin_rarity_meter) | Pending |

### 5. Composition, queries, interfaces, and applications

| Order | Crate | Status |
|---|---|---|
| 57 | [`bitview_default`](../crates/bitview_default) | Pending |
| 58 | [`bitview_query`](../crates/bitview_query) | Pending |
| 59 | [`bitview_website`](../crates/bitview_website) | Pending |
| 60 | [`bitview_server`](../crates/bitview_server) | Pending |
| 61 | [`bitview_bindgen`](../crates/bitview_bindgen) | Pending |
| 62 | [`bitview_client`](../crates/bitview_client) | Pending |
| 63 | [`bitview`](../crates/bitview) | Pending |
| 64 | [`bitviewd`](../crates/bitviewd) | Pending |
| 65 | [`bitview_cli`](../crates/bitview_cli) | Pending |
| 66 | [`bitview_mcp`](../crates/bitview_mcp) | Pending |
| 67 | [`blk`](../crates/blk) | Pending |
| 68 | [`mmpl`](../crates/mmpl) | Pending |

### 6. Benchmark tooling and external composition example

| Order | Crate | Status |
|---|---|---|
| 69 | [`bitviewd_bench`](../crates/bitviewd_bench) | Pending |
| 70 | [`bitviewd_latency`](../crates/bitviewd_latency) | Pending |
| 71 | [`bitviewd_bench_visualizer`](../crates/bitviewd_bench_visualizer) | Pending |
| 72 | [`custom_plugin`](../examples/custom_plugin) (`bitview-custom-plugin-example`) | Pending |

Use the benchmark tools earlier when needed to validate changes; their own full
code audits come here.

## Per-crate workflow

1. **Establish scope.** Mark the crate Active. Read its API, implementation,
   consumers, features, tests, and documentation. Record the revision and any
   relevant pre-existing working-tree changes. Identify invariants and likely
   hot paths before editing.
2. **Record findings and improve the code.** Review every module against the
   standards above. Make small, coherent changes with a clear reason. Follow a
   shared-contract fix into affected callers; record any broader follow-up
   against the crate that owns it. A reviewed area may need no changes.
3. **Validate the actual behavior.** Run formatting, targeted checks, Clippy,
   and relevant tests for the affected packages and supported feature sets.
   Add regression coverage for meaningful behavior changes or uncovered
   invariants. Exercise real consumers when changing a shared contract. Record
   exact commands, results, and any environment or platform gaps.
4. **Measure performance when relevant.** Capture comparable before/after
   release measurements for changed hot paths, with revision, hardware,
   workload, feature set, and timing/memory results. Validate a representative
   integration workload as well as a microbenchmark when needed. Existing
   benchmark numbers are historical until reproduced.
5. **Close the review.** Update the crate's entry with reviewed coverage,
   changes, retained design decisions, validation, and remaining work. Mark
   Done only after the full crate has been reviewed and required fixes and
   checks are complete. Any deliberately deferred improvement needs a reason
   and an owning crate or issue; unresolved correctness or safety findings
   keep the crate open.

After the crate reviews, run a final workspace integration pass covering
supported features, public APIs, generated-client contracts, and representative
indexing/query workloads. Record failures and reopen the responsible entries
before declaring the full audit complete.

## Progress log

### 2026-09-18 — Audit setup

- Inventoried all 72 workspace packages and established the review order from
  their internal normal/build dependencies, including optional dependencies.
- No crate audit has started. First review: `rawdb`.
- Initial `rawdb` review scope: region allocation/growth/removal, mmap reader
  lifetimes and remapping, lock ordering, metadata validation, flush/reopen
  guarantees, hole reuse/compaction, background work, and error paths. Check
  the storage contract at its `vecdb` consumer boundary.

For each crate, append a dated entry containing:

- **Coverage:** modules and contracts reviewed; exact stopping point if partial.
- **Findings and decisions:** problems found, changes made, and why retained
  complexity is justified; link relevant files and commits when available.
- **Validation:** commands and outcomes, integration checks, benchmark evidence
  where relevant, and gaps.
- **Remaining work:** unresolved findings, explicit deferrals with owners, and
  the next concrete action.

### 2026-09-18 — `rawdb` first pass

- **Baseline:** `f38b460d7162515ea5c35db0e74dc24e2987db68`; `rawdb` and
  `vecdb` had no working-tree changes at the start of this review. Other
  unrelated workspace changes were already present.
- **Coverage:** reviewed every source module, the manifest, README, and all 65
  original tests; traced mmap and buffered readers, writes, and region flushes
  through `vecdb`, plus background compaction in `bitview_plugin`. Checked the
  indexer and distribution loops: both already release their stored readers
  before publishing buffered writes.
- **Findings and changes:**
  - Failed removal previously removed the allocation before rejecting extra
    references. Validate ownership before changing either registry.
  - Reject invalid IDs, corrupted IDs, and oversized capacities without panics
    or consuming free space. Publish mapping capacity only after remapping works.
  - Protect borrowed bytes with region read/write locks and use raw mappings
    without creating a shared slice over independently writable regions.
    Bound every reader to its region; align buffered `vecdb` readers with the
    same access contract and release readers before publishing writes.
  - Unify reservation and write growth. Retry after remapping without holding
    allocation or flush-barrier locks; remove tentative allocation bookkeeping.
  - Use one database-wide data-before-metadata flush path. Exclude mutations,
    retain dirty state on errors/panics, and include file-size-only changes.
  - Replace uncounted background `Arc` handles with owned storage and a shared
    shutdown guard. Join remaining tasks after errors or panics.
  - Reclaim unused trailing file space after reopen. Correct the README's
    unsupported atomic-metadata and crash-consistency claims.
  - Replace the hole-punch wrapper type with a function and use libc's macOS
    ABI type. Remove unused error and reservation bookkeeping, inherit the
    workspace's `smallvec` dependency, and correct the package description.
- **Retained design:** region locks enforce the lifetime of borrowed bytes;
  the database mutation barrier orders data and metadata synchronization.
  Raw mappings allow disjoint regions to be accessed without forming aliases
  over the entire map. The reader's lifetime extension remains localized, with
  owning handles and guard drop order documented. Separate background storage
  ownership and shutdown ownership avoid reference cycles and uncounted `Arc`s.
  Existing allocation indexes and residency sampling retain their focused
  storage responsibilities; this review does not retune their heuristics.
- **Compatibility at this stage:** the disk format is unchanged. Raw mapping
  and mutable file access became internal. The second pass below further
  simplifies the reader API and removes its compatibility alias. All reads keep
  the region stable,
  including buffered `vecdb` I/O; release readers before modifying that region
  and mmap readers before growing the file. `Region::flush` now flushes the
  database because metadata is shared. These API/behavior changes require
  downstream callers outside this workspace to follow the documented contract.

**Validation**

The baseline passed 65 integration tests and one doctest. The first five new
failure-path regressions failed on that baseline and passed after the fixes.
Added 19 tests covering concurrency, failed removal, invalid metadata/IDs,
capacity overflow, batch unwinding and ordering, flush/reopen behavior,
compaction, and background task errors, panics, and owner shutdown.

Commands that build/link use
`SDKROOT=/Library/Developer/CommandLineTools/SDKs/MacOSX26.5.sdk`; the default
macOS SDK has a local linker incompatibility. No global toolchain setting changed.

| Command | Result |
|---|---|
| `cargo test --offline -p rawdb -p vecdb --all-features` | 540 tests/doctests passed; 6 existing ignored tests/doctests |
| `cargo test --offline -p rawdb` | 84 integration tests and 1 doctest passed after the platform wrapper cleanup |
| `cargo clippy --offline -p rawdb --all-targets -- -D warnings` | Passed |
| `cargo fmt -p rawdb -- --check` | Passed |
| `rustfmt --check --edition 2024 crates/vecdb/src/variants/raw/sources/io.rs crates/vecdb/src/variants/compressed/sources/io.rs crates/vecdb/tests/overflow.rs` | Passed |
| `cargo check --offline --workspace --all-targets` | Passed |
| `cargo test --offline -p bitview_plugin compaction_completes_on_next_update_and_database_drop` | Passed |
| `cargo test --offline -p bitview_plugin_indexer parent_resolution_preserves_store_updates_bounds_errors_and_pending_reads` | Passed |

**Performance**

Added a repeatable [storage benchmark](../crates/rawdb/benches/storage.rs).
[Recorded samples and environment](../benches/rawdb/2026-09-18-audit.json) compare
the baseline with the audited hot paths on an Apple M3 Pro with 36 GiB RAM,
Rust 1.98.1, release optimization, fat LTO, one codegen unit, and native CPU
targeting. Four interleaved rounds produced 20 samples per version with no
concurrent builds. Both versions used the same workload and dependency versions.

| Workload | Before median (ms) | After median (ms) |
|---|---:|---:|
| Append 64 MiB in 64 KiB blocks | 5.2650 | 5.2595 |
| Ordered batch of 1 million u64 updates | 0.2700 | 0.3005 |
| 100,000 scoped one-byte reads | 0.5835 | 0.8375 |
| Eight scans of 64 MiB | 9.9245 | 9.8530 |
| Database flush after writes | 22.1385 | 21.4170 |

Bulk append and scan timings are comparable in this workload. Bounds/ordering
checks cost about 0.031 ms per million batch updates (11%); scoped locking and
ownership cost about 2.54 ns per call (44%). Retain those costs for correctness;
callers can reuse a reader or batch operations in hot loops. Flush timings are
noisy and do not establish a speedup. No full-indexer performance claim is made.

**Limits and next action**

Validated on macOS with temporary fixtures. Linux/FreeBSD runtime behavior,
physical power-loss recovery, the existing ignored stress tests, and a full
production indexing benchmark were not exercised. Rawdb remains intentionally
non-transactional; the README now states its actual persistence limits.

The correctness pass is complete. The full `vecdb` review remains pending,
including its own buffering, compression, cache, and rollback contracts.

### 2026-09-18 — `rawdb` responsibility and API cleanup

Reopened `rawdb` for a stricter pass on single responsibilities, idiomatic Rust,
unused APIs, and unnecessary state. The comparison baseline is the first pass's
working tree, saved before this cleanup; unrelated workspace changes remain intact.

**Responsibilities**

| Type | Responsibility |
|---|---|
| `Database` / `DatabaseInner` | Coordinate storage operations / own shared storage components |
| `DataFile` | Data mapping, growth, physical allocation, and file synchronization |
| `MetadataFile` | Fixed-size metadata slots and their persistence |
| `Regions` | Register and find live regions by ID and metadata slot |
| `Layout` | Track allocations and reusable/pending holes |
| `RegionMetadata` | Stored fields and their encoding/validation |
| `Region` / `RegionInner` | Region operations / live region state and synchronization |
| `Reader` | Bounded borrowed access with stable ownership and guards |
| `BackgroundTasks` | Worker registration, deferred waits, and joining |
| `DatabaseOwner` / `WeakDatabase` | Join on last foreground-owner drop / recover handles without cycles |
| `DirtyRange` / `DiskUsage` | Preserve dirty bytes during unwinding / represent physical disk usage |

Each struct has its own production source file. The small ownership and unwind
guards remain because they enforce resource lifetimes and failure behavior.

**Changes**

- Removed metadata's redundant `needs_write` state, generic change-tracking
  helper, custom `Clone`, and unused `Display`. Mutation sites write metadata
  when fields change; reclamation state belongs to the live region.
- Decode fixed-size slots into `Result<Option<RegionMetadata>>`. Empty slots
  are ordinary absence; malformed slots return corruption errors. Removed
  impossible size/sentinel errors and three private allocation error variants;
  updated `vecdb`'s error classification accordingly.
- Replaced size-to-vector hole buckets with `BTreeSet<(size, start)>`. This
  removes the `smallvec` dependency and bucket maintenance, and gives a stable
  lowest-offset tie break. Added a regression covering separate equal-sized
  holes and preserved neighboring regions.
- Made registry, layout, weak handles, and raw file access internal. Moved the
  existing tests inside the crate so they can verify invariants without exposing
  diagnostic implementation details as production APIs. Coverage is preserved.
- Removed unused `open_with_min_len`, weak-handle conversion helpers, public
  layout constants, redundant accessors, and forced-inline annotations. Use
  standard integer alignment methods and direct ownership transfer for task joins.
- Removed `unchecked_read`; all callers use checked `read`. Renamed `prefixed`
  to `read_from` and updated every workspace caller. Moved unused-in-rawdb branch
  hints to `vecdb`, retaining its existing exports for consumers.
- Replaced broad root imports/exports with explicit ones and reduced `lib.rs`
  to module declarations and the public API. No on-disk format change.

These intentionally narrow the public API. External callers of removed APIs
must migrate; workspace consumers are updated together.

The pass reduces production Rust source from 2,265 to 2,101 lines, including
the new responsibility boundaries. It removes a dependency and unnecessary
state without replacing them with generic frameworks or additional runtime layers.

**Validation**

The same local SDK override described above applies to build/link commands.

| Command | Result |
|---|---|
| `cargo test --offline -p rawdb -p vecdb --all-features` | 541 tests/doctests passed; 6 existing ignored tests/doctests |
| `cargo test --offline -p vecdb --all-features --lib` | All 38 tests passed after relocating the branch hints |
| `cargo test --offline -p bitview_plugin compaction_completes_on_next_update_and_database_drop` | Passed |
| `cargo check --offline --workspace --all-targets` | Passed |
| `cargo clippy --offline -p rawdb --all-targets -- -D warnings` | Passed |
| `cargo fmt -p rawdb -- --check` | Passed |
| `rustfmt --check --edition 2024 --config skip_children=true` on every modified `vecdb` Rust file | Passed |
| `git diff --check` on affected crates and this document | Passed |

**Performance**

[Recorded samples, source hashes, and environment](../benches/rawdb/2026-09-18-cleanup.json)
compare this cleanup with the first pass's working tree. The benchmark now also
measures reuse of 2,048 equal-sized holes among 4,096 regions, excluding setup
and flush from allocation timing. Identical release harnesses use the same
toolchain and shared dependency versions. After an initially noisy scan result,
both executables were warmed, then run in four interleaved rounds for 20 samples
per version. Initial results and warmups are retained separately in the artifact.

| Workload | First pass median (ms) | Cleanup median (ms) |
|---|---:|---:|
| Append 64 MiB in 64 KiB blocks | 5.5430 | 5.6530 |
| Ordered batch of 1 million u64 updates | 0.2985 | 0.3020 |
| 100,000 scoped one-byte reads | 0.8240 | 0.8115 |
| Eight scans of 64 MiB | 9.8180 | 9.8355 |
| Database flush after writes | 21.1700 | 20.9415 |
| Reuse 2,048 equal-sized holes | 5.6625 | 3.9015 |

The simpler hole index reduces allocation latency by about 31% in this workload.
Other medians are within about 2%; scan timing is essentially unchanged after
warmup. These are local microbenchmarks, not full-indexer performance guarantees.

**Remaining work:** none identified for this pass. The platform, power-loss, and
production-workload validation limits from the first pass still apply. `rawdb`
is Done; next crate is **`brk_exit`**. `vecdb` retains its separate full audit slot.

### 2026-09-18 — `rawdb` deduplication and hot-path pass

Reopened `rawdb` to challenge the previous responsibility boundaries and measure
remaining repeated work. The baseline is the second pass's saved working tree;
its source hash matches that pass's recorded result.

**Responsibilities and simplifications**

- `DirtyRanges` owns one invariant: sorted, nonempty byte ranges with overlap
  and adjacency merged. Region writes and database flushes now use this single
  implementation. Appending ordered ranges touches only the last entry; merging
  many existing ranges shifts the surviving suffix once instead of repeatedly
  removing entries from the middle of a vector.
- `PendingFlush` owns dirty bookkeeping until a flush succeeds. Its drop guard
  restores that bookkeeping on errors or unwinding, replacing manual recovery
  branches. This is an in-memory recovery guarantee, not transactional storage.
- Flush traverses regions in the layout's existing allocation order and combines
  their ordered dirty ranges directly. Removed the extra global sort and second
  range-coalescing implementation. Relocation and metadata-slot reuse do not
  change this ordering guarantee.
- Renamed the batch panic guard from `DirtyRange` to `DirtyWrite`, distinguishing
  its responsibility from the range collection. Each new type has its own file.
- `Regions` uses a standard min-heap of vacant metadata slots, preserving lowest
  slot reuse without scanning the registry on every creation. Reopening rebuilds
  the index; shrinking removes stale entries. This costs one `usize` per vacant
  slot plus vector spare capacity. The standard container fits the registry's
  existing responsibility; an additional allocation framework would not help.
- Metadata capacity checks return immediately when the mapping already fits,
  avoiding a filesystem metadata syscall on each reused slot. Growth still
  retries a failed remap before committing a registry change.
- Existing slots update only their 24 bytes of numeric bounds. The immutable ID
  and padding no longer incur a 4 KiB encode/copy for every append or truncation.
  Full creation and removal still write the whole slot. Bounds encoding is shared
  with full-slot encoding; the disk format is unchanged.

The separate scoped-read and owned-reader APIs remain: their lifetimes differ,
and forcing a temporary owned reader into every scoped read would introduce
extra handle cloning. No wrapper types were added solely to split up methods.

**Validation**

Added five regression tests: range union against an independent byte-coverage
model, merging thousands of fragments while retaining the suffix, dirty-state
restoration during unwinding after relocation, metadata-slot reuse across reopen
and shrink, and unchanged identity bytes after partial metadata writes and reopen.

Build/link commands use the SDK override documented above.

| Command | Result |
|---|---|
| `cargo test --offline -p rawdb` | 90 unit tests and one doctest passed |
| `cargo test --offline -p rawdb -p vecdb --all-features` | 546 tests/doctests passed; six existing ignored cases |
| `cargo test --offline -p bitview_plugin compaction_completes_on_next_update_and_database_drop` | Passed |
| `cargo check --offline --workspace --all-targets` | Passed |
| `cargo clippy --offline -p rawdb --all-targets -- -D warnings` | Passed |
| `cargo fmt -p rawdb -- --check` | Passed |
| `git diff --check` on affected crates, this document, and benchmark artifacts | Passed |

The measured production-source hash matches the final source. Every production
Rust file contains at most one public or crate-visible struct.

**Performance**

[Recorded samples, source hashes, and environment](../benches/rawdb/2026-09-18-hotpaths.json)
use identical release harnesses and dependency versions. Each executable was
warmed with five samples, then measured in four interleaved rounds, alternating
execution order, for 20 samples per version. No audit builds ran during timing.
The benchmark adds individual small appends and fragmented writes followed by a
full overwrite; setup and flush are excluded from those two timings.

| Workload | Second pass median (ms) | This pass median (ms) |
|---|---:|---:|
| Append 64 MiB in 64 KiB blocks | 5.5705 | 5.5305 |
| Ordered batch of 1 million u64 updates | 0.3000 | 0.2750 |
| 100,000 scoped one-byte reads | 0.7980 | 0.8135 |
| Eight scans of 64 MiB | 9.5515 | 9.7625 |
| Database flush after writes | 21.6965 | 22.1295 |
| Reuse 2,048 equal-sized holes and slots | 3.7910 | 1.9800 |
| 100,000 individual eight-byte appends | 18.0720 | 2.4695 |
| 8,192 fragmented writes, then full overwrite | 6.1290 | 0.1690 |

Small appends are about 7.3 times faster and the fragmented-write workload about
36 times faster. Hole/slot reuse latency falls about 48%, supporting the small
vacancy index's cost. Bulk append, scoped reads, scans, and flush are within about
2.3%; these small differences do not establish a meaningful improvement or
regression. Results describe local microbenchmarks, not production indexer speed.

**Remaining work:** none identified for this pass. The platform, power-loss,
ignored-stress-test, and production-workload limits from the first pass still
apply. `rawdb` is Done; next crate is **`brk_exit`**.

### 2026-09-18 — `rawdb` seven sequential passes

Run each pass to completion before beginning the next: scan, analyze, establish
test or benchmark evidence where needed, apply justified changes, then verify.
An unchanged result is valid when further changes add complexity or lack evidence.

| Pass | Focus | Status |
|---|---|---|
| 1 | Cleanliness | Done |
| 2 | KISS | Done |
| 3 | DRY | Done |
| 4 | Idiomatic Rust | Done |
| 5 | Speed | Done |
| 6 | Consistency | Done |
| 7 | Naming, architecture, organization | Done |

**1. Cleanliness**

- Scan/analysis: reviewed source, dependencies, visibility, derives, comments,
  test scaffolding, and workspace consumers. Confirmed that `Error::other` has
  real indexer callers and retained it; all five production dependencies are used.
- Evidence before edits: strict rawdb Clippy passed; workspace searches found no
  callers needing `WeakDatabase: Clone` or public `DiskUsage::from_file`.
- Applied: removed the unused clone derive, made the disk-usage constructor
  crate-private, removed stale baseline-test recovery scaffolding and a test-only
  inline annotation, and corrected README guard guidance and mmap doc placement.
  External users of `DiskUsage::from_file` must use `Database::disk_usage` instead.
- Verification: 90 rawdb unit tests and one doctest passed; diff whitespace check
  passed. No runtime optimization or new behavior required a benchmark.

**2. KISS**

- Scan/analysis: traced write/append/truncate modes, capacity retries, allocator
  indexes, pending-flush ownership, and background join/wakeup coordination.
- Decisions: retain the direct write helper, allocation indexes, and small RAII
  guards. A write-mode framework, generic mapped-file owner, or additional
  free-space wrapper would add layers without removing a distinct responsibility.
  Growth retries release locks before remapping; merging those paths would hide
  an important reader/deadlock constraint.
- Evidence: the preceding 91-test rawdb run covers these unchanged paths,
  including growth, batch unwinding, concurrent access, and background failures.
  No new behavior or plausible speed change justified another test/benchmark run.
- Applied: no production changes. This pass intentionally retains the simpler
  existing design rather than manufacturing a refactor.

**3. DRY**

- Scan/analysis: found duplicated create-without-truncation/exclusive-lock setup
  in both storage files, and three callers repeating live-slot traversal.
- Evidence before edits: added and passed an integration regression proving that
  locking either database file rejects reopening without truncating stored data;
  releasing the lock permits reopening and reading the original region.
- Applied: one `open_locked_file` function owns the shared opening contract;
  `Regions::iter()` owns live-region traversal. Raw slot access is now test-only.
  Data and metadata still own their distinct mapping and durability behavior.
- Verification: all 91 rawdb unit tests plus one doctest passed after refactoring;
  diff whitespace check passed. No hot-loop algorithm changed.

**4. Idiomatic Rust**

- Scan/analysis: reviewed ownership transfer, cloning, iterators, slice bounds,
  formatting, unsafe boundaries, and guard destruction order.
- Evidence before edits: expanded the existing suffix-reader test to include an
  empty suffix and oversized/overflowing offsets. All 91 unit tests passed,
  including pending-flush unwinding, before applying the implementation changes.
- Applied: failed flushes move their saved dirty-range allocation back into the
  region instead of cloning and merging it into an empty vector. The existing
  mutation barrier guarantees that no newer writes need merging. Suffix reads
  use ordinary checked slicing, reader initialization follows its guard/owner
  field order, and database display writes its string directly.
- Verification: 91 unit tests plus one doctest passed. Retained the owned reader's
  localized lifetime extension: replacing it requires extra lock allocations or
  a different public borrowing API, not a mechanical idiomatic improvement.

**5. Speed**

- Scan/analysis: traced per-record writes, range insertion, allocation, file growth,
  and reads. Found one redundant metadata read lock per non-growing write and an
  eagerly evaluated metadata lookup used only to construct reserve errors.
- Evidence before edits: saved the post-pass-4 source, built the release harness,
  and recorded initial benchmark samples. Existing tests already exercise
  relocation, boundary writes, concurrent appends, and flush/reopen behavior.
- Applied: reuse the start offset read under exclusive region access; refresh it
  only after growth can relocate the region. Construct reserve errors lazily.
  No new cache, allocation, unsafe operation, dependency, or relaxed check.
- Verification: 91 unit tests and one doctest passed. Two interleaved benchmark
  measurements retain all 40 samples per version; the second checked an initial
  5.3% shift in the unchanged allocation path. Its combined shift is 2.5%, with
  overlapping sample ranges. Other control medians are within 0.7%.

| Workload | Before median (ms) | After median (ms) |
|---|---:|---:|
| 100,000 individual eight-byte appends | 2.5255 | 2.3615 |
| 8,192 fragmented writes, then full overwrite | 0.1715 | 0.1570 |

The two write workloads improve by about 6.5% and 8.5%. These local microbenchmarks
support retaining the simpler lock usage; they do not establish a production
indexer improvement. [All samples, controls, and source hashes](../benches/rawdb/2026-09-18-seven-pass-speed.json)
are retained, including initial/warmup runs and each measurement's summary.

**6. Consistency**

- Scan/analysis: compared public errors, return-value descriptions, bounds,
  platform fallbacks, and the actual indexer background-operation callers.
  `Error::other` incorrectly labeled arbitrary caller errors as rawdb invariant
  violations; lock errors also hid their underlying reason.
- Evidence before edits: strengthened the background-error integration test to
  require the original caller message. It failed with the misleading invariant
  prefix, confirming the behavior before the fix.
- Applied: caller errors use `Other(String)` and preserve their message; lock
  errors display their underlying error. Documented extending writes, truncation,
  region-flush results, the non-Unix disk-usage fallback, and the mmap hint's
  intentionally unchecked short-range policy. Actual readers still check bounds.
  External matches on the old `InvariantViolation` variant must use `Other`;
  workspace callers use the unchanged `Error::other` constructor.
- Verification: 91 unit tests plus one doctest passed, including the previously
  failing background-error assertion and the file-lock regression. No hot-path
  behavior changed, so no new benchmark was needed.

**7. Naming, architecture, organization**

- Scan/analysis: reviewed module ownership, method names against what they mutate,
  public versus internal types, platform code, and the 2,000-line test root.
  Region operations also contained the OS residency sampler; `update_metadata`
  no longer described the restricted bounds-only write it performs.
- Evidence before edits: expanded and passed the existing residency test across
  several 16 MiB sampling windows, an unaligned subrange, and overflowing offsets.
- Applied: moved page-size discovery and residency sampling into a private
  `residency` module. Region operations retain bounds and lock ownership; the
  sampler receives the stable mapping. A plain function fits this responsibility
  without a new stateful type. Unavailable page-size information now declines
  mmap instead of panicking, matching the documented probing-error policy.
  Renamed the registry mutation to `update_bounds`.
- Organized 65 existing tests into database, writes, reader, persistence,
  compaction, allocation, concurrency, metadata, lifecycle, and residency modules.
  The test root now contains only module declarations and a shared fixture
  constructor (23 lines, including platform gating). Hash comparisons confirm
  every moved test's attributes and function body are unchanged, including after
  formatting.
- Final review of the extracted sampler boundary added an empty/invalid-range
  regression. It failed before the fix. The sampler now accepts empty in-bounds
  ranges without probing and checks that page geometry divides the sampling
  window before using it. This protects the new internal function boundary;
  region callers still only probe large ranges.
- Verification: 92 unit tests and one doctest passed after the boundary fix;
  strict all-target rawdb Clippy passed. Every production file still contains at
  most one public/crate-visible struct. The sampling policy is unchanged.

**Final integration verification**

All seven passes above ran in order, with each completed before the next began.
Build/link commands use the SDK override documented earlier.

| Command | Result |
|---|---|
| `cargo test --offline -p rawdb -p vecdb --all-features` | 547 tests/doctests passed; six existing ignored cases |
| `cargo test --offline -p rawdb --target-dir /private/tmp/rawdb-seven-unit-target` | Final sampler fix: 92 unit tests and one doctest passed |
| `cargo test --offline -p bitview_plugin compaction_completes_on_next_update_and_database_drop` | Passed on final source |
| `cargo check --offline --workspace --all-targets` | Passed on final source |
| `cargo clippy --offline -p rawdb --all-targets -- -D warnings` with the isolated target directory | Passed on final source |
| `cargo fmt -p rawdb -- --check` | Passed |
| `git diff --check` on affected crates, this document, and benchmark artifacts | Passed |

The combined suite was built before the final sampler-boundary guard. That
localized fix and its additional regression were verified in the separate final
rawdb run; the compaction consumer and workspace checks subsequently compiled
the final source. Speed measurements isolate pass 5, before the diagnostic and
organization changes in passes 6–7.

**Remaining work:** none identified for this seven-pass cycle. `rawdb` is Done.
The existing platform, physical power-loss, ignored-stress-test, and production
indexer validation limits remain. Next crate: **`brk_exit`**.

### 2026-09-18 — `rawdb` repeat cycle with exposability

Repeat the ordered scan → analysis → relevant tests/benchmarks → justified fixes
→ verification process, completing each pass before moving to the next. Add an
explicit boundary review: public exports need real external responsibilities;
internal helpers and implementation details stay inside the crate.

| Pass | Focus | Status |
|---|---|---|
| 1 | Cleanliness | Done |
| 2 | KISS | Done |
| 3 | DRY | Done |
| 4 | Idiomatic Rust | Done |
| 5 | Speed | Done |
| 6 | Consistency | Done |
| 7 | Naming, architecture, organization | Done |
| 8 | Exposability | Done |

**1. Cleanliness**

- Scanned production modules, dependencies, annotations, fallible operations, and
  test support. No new unused dependency or dead implementation was found.
- The starting baseline passed strict all-target Clippy and 92 unit tests plus
  one doctest. Removed forced-inline hints from file opening and compaction:
  these cold syscall paths do not warrant optimization annotations.
- Verification after edits: all 93 rawdb tests/doctests passed. No behavior changed.

**2. KISS**

- Reviewed allocation/growth retry loops, shared ownership, background joining,
  dirty-flush rollback, and the layout indexes. Each retained type owns a distinct
  lifetime or invariant. The size-ordered hole index and vacancy heap already have
  measured justification in earlier passes.
- Kept the explicit lock-release/retry paths: shortening them would hide lock
  ordering or wait for readers while holding allocation locks. Kept the owning
  flush guard and foreground-owner guard; plain cleanup calls lose unwind or
  last-owner semantics. No extra wrapper, mode enum, or trait is warranted.
- No code change. The just-completed 93-test run covers these unchanged paths,
  including concurrency, lifecycle, allocation, and flush failures.

**3. DRY**

- Traced metadata encoding, allocation/growth, three write entry points, dirty
  range merging, file opening, and registry iteration. Shared behavior already
  has one owner: `bounds_bytes`, `write_with`, `reserve_inner`, `DirtyRanges`,
  `open_locked_file`, and `Regions::iter`.
- Kept validation at the persisted-input and mutation boundaries: decoding an
  untrusted file and asserting an internal setter's invariant have different
  failure contracts. Kept creation's second ID lookup after acquiring locks;
  it prevents concurrent duplicate creation and is not redundant with the fast
  initial lookup.
- No further abstraction reduces real duplication without hiding these contracts.
  No code change; existing metadata, write, and concurrency tests remain green.

**4. Idiomatic Rust**

- Reviewed iterators, conversions, error propagation, ownership/drop order,
  callback bounds, module-level imports, and every production unsafe block.
  Input failures use `Result`; slicing/assertions protect programmer contracts.
  The fixed-width metadata decoder's `unwrap` follows an exact eight-byte slice.
- Retained the owned reader's documented guards and stable Arc allocations.
  Replacing this with another ownership framework would add machinery without
  simplifying the existing lifetime proof. No exposed unsafe API is required.
- No code change. Strict baseline Clippy and the existing reader/bounds/lifecycle
  tests cover the unchanged implementation; final verification repeats the gate.

**5. Speed**

- Re-scanned ordinary appends, ordered batch writes, borrowed/owned reads,
  fragmented dirty merging, hole/slot reuse, and flush collection. Ordinary writes
  retain the cached stable start; batch loops acquire guards once; metadata
  updates copy only bounds; ordered range collection does not sort again.
- Considered caching more metadata and changing flush range ownership. Neither
  removes required synchronization without introducing additional mutable state
  or complicating rollback. No plausible remaining hot-loop fix justified an
  experiment in this pass.
- No runtime change and no new timing claim. Existing recorded benchmarks remain
  historical evidence for retained optimizations; no rerun was needed to compare
  identical hot-path code. Final integration tests still exercise those paths.

**6. Consistency**

- Checked README contracts against region/database flushing, reader lifetimes,
  short-range residency hints, error messages, background cancellation, and the
  vecdb/indexer integration. Checked matching APIs for boundary behavior.
- The differing flush return types are intentional and documented: database
  flush reports a region count; region flush reports whether any database data
  or metadata was synchronized. Residency remains an advisory heuristic, while
  readers enforce bounds. No behavior or documentation mismatch found.
- No code change. Existing empty/overflow reader tests, invalid residency tests,
  shared-metadata flush tests, and background error tests cover these contracts.

**7. Naming, architecture, organization**

- Scanned the module graph, root aliases, type responsibilities, and test layout.
  Five internal types were imported through private root aliases, unlike the
  remaining helpers which name their defining modules directly.
- Removed those five aliases and made their consumers import the defining
  modules. The root now lists exports without also serving as an internal
  namespace. No new type or module was needed.
- Verification: all-target rawdb compilation and formatting passed; every
  production file still contains at most one public/crate-visible struct.
  This changes import paths only, so no additional behavioral test was added.

**8. Exposability**

- Inventoried root exports, every public inherent method, public fields, returned
  types, and internal declarations. Searched workspace source, tests, examples,
  docs, and vecdb re-exports; compiler checks below validate resolved callers.
- Made the redundant `error` module private, retaining root `Error` and `Result`.
  Narrowed `Database::file_len` to crate visibility and `Database::name` to its
  own module. Removed the unneeded `compact_deferred(delay)` wrapper; its sole
  caller still waits five cancellable seconds before compacting. Custom delays
  remain expressible through the externally used `bg_sleep` and `compact`.
- Removed `DiskUsage`, both `disk_usage` forwarding methods, and formatting code:
  no production consumer uses this type. Reclamation tests now inspect the data
  file's allocation through one test-only helper, preserving their assertions.
- Made internal fields/functions explicitly `pub(crate)` and enabled
  `unreachable_pub` warnings. These declarations were already effectively
  internal; the lint keeps source visibility honest and catches accidental
  unreachable exports. It does not detect unused reachable public APIs.
- Kept the concrete read guards needed by vecdb's buffered I/O cursors, and kept
  `RegionMetadata` publicly nameable because `Region::meta` returns it. Its
  fields, constructors, setters, encoding, and decoding remain internal. Clarified
  the metadata guard's read-only role and required access lock for buffered I/O.
- No extra wrapper or opaque return type was introduced to conceal guards whose
  real consumers store them. No unsafe function, internal state field, mapping,
  registry, allocator, or worker-management type is exported.

| Retained surface | External responsibility/evidence |
|---|---|
| `Database::open`, `get_region`, `create_region_if_needed` | vecdb imports, mutable hole storage, plugin setup |
| `Database::set_min_len`, `path` | plugin/indexer preallocation; vecdb rollback paths |
| `Database::remove_region`, `remove_region_if_exists`, `retain_accessed_regions` | vecdb resets and mutable holes; plugin import cleanup |
| `Database::flush`, `compact` | documented durability API; indexer checkpoint publication and plugin import |
| `Database::run_bg`, `bg_sleep`, `compact_deferred_default`, `sync_bg_tasks` | indexer deferred commit; plugin update lifecycle; custom plugin example |
| `Region::create_reader`, `with_read_bytes` | vecdb header/page readers and single-value reads |
| `Region::read_lock`, `open_db_read_only_file`, `prefers_mmap` | vecdb raw/compressed I/O cursors and source selection |
| `Region::reserve_capacity`, `write`, `write_at`, `batch_write_ordered`, `truncate`, `truncate_write` | vecdb capacity/header/batch/rollback operations; documented append API and benchmark |
| `Region::remove`, `flush`, `db`, `meta` | documented removal; vecdb flushing, database ownership, and bounds |
| `Reader::{read, read_all, read_from, len, is_empty}` | vecdb header/body/range reads; `is_empty` is the idiomatic companion to public `len` |
| `RegionMetadata::{start, len, is_empty, reserved, id}` | vecdb buffered I/O bounds, headers/errors, initial-capacity integration tests; idiomatic emptiness query |
| `Error`, `Result`, `Error::other` | fallible public operations, vecdb error classification, indexer callback errors |
| `PAGE_SIZE` | vecdb allocation policy and plugin/indexer preallocation |

The external-consumer probe passed its public API compilation and rejected all
nine intended forbidden accesses (private error/storage modules, removed disk
usage type/method, internal file length/name, removed deferred wrapper, metadata
mutation, and region internals), with the expected Rust privacy/missing-item errors.
Public inherent methods decreased from 45 to 40.

Public root now has four structs (`Database`, `Region`, `Reader`,
`RegionMetadata`), one enum (`Error`), one alias (`Result`), and one constant
(`PAGE_SIZE`), with no public modules. This deliberately removes previously
public APIs; all known workspace consumers are retained. Unknown out-of-repo
consumers of the removed APIs would need migration.

**Final verification for this eight-pass cycle**

All eight passes ran in order. The final source passed every check below using
`SDKROOT=/Library/Developer/CommandLineTools/SDKs/MacOSX26.5.sdk` for builds.

| Check | Result |
|---|---|
| `cargo test --offline -p rawdb -p vecdb --all-features` | 548 unit/integration tests and doctests passed; six existing ignored cases |
| `cargo test --offline -p bitview_plugin compaction_completes_on_next_update_and_database_drop` | Passed |
| `cargo check --offline --workspace --all-targets` | Passed |
| `cargo clippy --offline -p rawdb --all-targets -- -D warnings` | Passed, including `unreachable_pub` |
| `cargo doc --offline -p rawdb --no-deps` | Passed; generated root lists exactly seven exports, with narrowed methods and metadata mutators absent |
| Separate external-consumer compilation | Public API succeeds; nine negative probes fail with the expected privacy/missing-item diagnostics |
| `cargo fmt -p rawdb -- --check` and affected-file `git diff --check` | Passed |

Production source decreased from 2,211 to 2,148 lines in this repeat cycle.
No new hot-path implementation or benchmark claim was introduced. The prior
platform, physical power-loss, ignored-stress-test, and production-indexer
validation limits still apply.

**Remaining work:** none identified for this cycle. `rawdb` is Done; exposability
is now part of the standing review sequence for every crate. Next: **`brk_exit`**.

### 2026-09-18 — `rawdb` ownership and module organization follow-up

Status: Done. The preceding organization review missed that related types and
helpers were still scattered across the crate root. Group them by ownership
under `database/` and `region/`, after checking whether the type splits themselves
are necessary. Keep only genuinely shared helpers at the root.

**Findings and changes**

- `Database`, `DatabaseInner`, and `DatabaseOwner` represent different lifetimes:
  the public handle, shared storage, and foreground shutdown ownership. Workers
  retain storage while the final foreground owner must still wake and join them.
  Removing the guard or relying on a strong-count check would lose the existing
  concurrent-final-drop guarantee. Keep the small guard, but remove its trivial
  forwarding constructor.
- Grouped shared storage, file mappings, allocation, registry, flushing, workers,
  ownership, and weak handles under `database/`. Related files now use local
  `super` imports rather than routing through the crate root.
- Grouped the region handle, state, metadata, reader, batch dirty guard, and
  residency probing under `region/`. `DirtyRanges` remains shared by region writes
  and database flush collection, so it stays at the root.
- The root now declares four production modules instead of twenty-one. Existing
  root exports stay unchanged. `Database::owner` is now private to the database
  module; the owner guard, region-local constructors, and batch guard have
  narrower parent-module visibility.
- Reviewed the per-file move diff: executable logic is unchanged apart from
  replacing the owner constructor's one-line forwarding call with direct tuple
  construction. No extra behavioral tests or benchmark were needed; existing
  concurrency, ownership, read, and integration tests validate the move.

```text
src/
  database/
    mod.rs          # Database handle and orchestration
    inner.rs        # shared storage state
    owner.rs        # last-foreground-owner shutdown guard
    weak.rs         # non-owning database reference held by regions
    ...             # storage, allocation, flush, and worker internals
  region/
    mod.rs          # Region operations
    inner.rs
    metadata.rs
    reader.rs
    dirty_write.rs
    residency.rs
  dirty_ranges.rs   # shared by database flushes and region writes
  error.rs
  lib.rs
  tests/
```

**Verification**

- `cargo test --offline -p rawdb`: 92 unit tests and one doctest passed, including
  concurrent final-owner drops and background failure handling.
- `cargo test --offline -p vecdb --test raw_reader_cursor --test initial_capacity`:
  six external reader/metadata integration tests passed.
- Plugin `compaction_completes_on_next_update_and_database_drop`: passed.
- Workspace all-target compilation, strict rawdb all-target Clippy, formatting,
  and affected-file diff checks: passed.
- Rustdoc: generated successfully; exactly the same seven public root exports.
- One visible struct per production file: verified after the moves.

All builds used the SDK override recorded above. No runtime algorithm or hot-loop
change was introduced. This follow-up corrects the earlier organization review's
claim that no additional grouping was warranted.

### 2026-09-19 — `rawdb` KISS and responsibility review

Status: Done. Re-evaluate every retained type and coordination mechanism,
including possible merges and splits. A concrete candidate is dirty-flush
bookkeeping: the mutation barrier permits retaining dirty state until success,
which could remove the take/restore guard entirely. Validate failure, unwind,
allocation reuse, and consumer behavior before retaining this simplification.

**Scan and decisions**

| Candidate | Decision and reason |
|---|---|
| `PendingFlush` plus take/restore methods | Remove. The exclusive mutation barrier already keeps dirty state stable; retain the original ranges until both syncs succeed, then clear them. Recovery bookkeeping was unnecessary. |
| `Database`, shared storage, owner guard, weak handles | Keep. Foreground shutdown and worker storage have different lifetimes; keeping the guard avoids extra ownership-mode bookkeeping or unsafe reference-count checks. |
| `Regions` + `MetadataFile` | Keep separate. Registry membership/slot reuse and mapped-file encoding/synchronization have different invariants. A merge saves a few forwarding methods but mixes raw-memory access into registry operations. |
| `Regions` + `Layout` | Keep separate. Logical ID/slot lookup and physical free-space allocation serve different operations; combining their locks would couple ordinary metadata writes to allocation bookkeeping. |
| Split `Layout` into another free-space wrapper | Reject. Its indexes jointly enforce one allocation invariant; another wrapper adds delegation without an independent client or lifetime. |
| `RegionInner` + `RegionMetadata` | Keep separate. Runtime locks and dirty/access flags must remain distinct from the public read-only persisted bounds/ID view. |
| `DataFile` + `MetadataFile`, or a generic mapped-file abstraction | Reject. Their remapping, slot encoding, locking, and durability rules differ. Common low-level file opening and copying already have shared functions. |
| Remove `DirtyWrite` | Reject. Batch callbacks/iterators may panic after modifying bytes. Marking an entire region dirty instead would inflate flush work; this small guard protects completed writes. |
| Replace `DirtyRanges` with raw vectors | Reject. Two consumers require its same canonical merging contract; measured ordered insertion avoids repeated sorting. |
| Replace `Reader` guards with another ownership framework | No change. Its guards must retain both mapping lifetime and region/database ownership. Additional Arc-owned locks would add allocations/reference counting while the existing invariants remain explicit. |
| Split region allocation/write operations into helper objects | Reject. Their transient values and guards belong to one operation; local functions keep lock release/retry ordering visible. |

**Applied simplification and correctness evidence**

- Removed `PendingFlush` and its module, owned per-region range snapshots, Arc
  clones, take/restore methods, and Drop-based recovery. Flush now gathers borrowed
  dirty-region references plus absolute ranges, syncs data and metadata, then
  clears dirty bookkeeping. Dirty allocations are still released after success;
  the change does not retain peak per-region buffers.
- Hold the existing layout lock while borrowing its region handles. Mutations
  already remain excluded by the outer barrier, so no new lock or state exists.
- Added a real `Database::flush` failure-path regression before the change. An
  intentionally invalid dirty span makes mmap flushing return `InvalidInput`;
  two consecutive failures verify that dirty state is retained and removed
  allocations are not prematurely reusable. This exercises mmap-error handling,
  not physical disk-failure simulation, and needs no production test hook.
- The old direct `PendingFlush` unwind test became a public-operation regression
  for relocated regions, a single successful flush, subsequent clean flush, and
  reopening both regions. All 94 rawdb tests/doctests passed before and after the
  production change. Batch panic and concurrent write/flush coverage is retained.
- Extended the existing benchmark with all-dirty and sparse-dirty flushes across
  1,024 regions, plus 32 clean flushes. Compare identical benchmark/dependency
  versions before and after; measurements and final integration results follow.

**Benchmark result**

Kept the simpler implementation after four interleaved comparison rounds
(20 measured samples per version, plus warmups), using identical harness sources
and dependency lockfiles. No audit builds ran during timing. Full measurements:
[`2026-09-19-kiss-flush.json`](../benches/rawdb/2026-09-19-kiss-flush.json).

| Workload | Before median | After median |
|---|---:|---:|
| Flush appended/batch-written data | 22.678 ms | 22.479 ms |
| Flush 1,024 dirty regions | 10.221 ms | 10.080 ms |
| Flush 64 dirty regions among 1,024 | 4.884 ms | 4.9165 ms |
| 32 clean flushes over 1,024 regions | 0.0975 ms | 0.0965 ms |

Flush median shifts are within approximately 1.4%; this run supports comparable
performance, not a new throughput claim. The eight existing control workloads
are retained in the record. The concrete benefit is one fewer struct and 44
fewer production lines, with no moved dirty state or recovery-on-drop path.

**Final verification**

| Check | Result |
|---|---|
| `cargo test --offline -p rawdb -p vecdb --all-features` | 549 tests/doctests passed; six existing ignored cases |
| Plugin `compaction_completes_on_next_update_and_database_drop` | Passed |
| `cargo check --offline --workspace --all-targets` | Passed |
| `cargo clippy --offline -p rawdb --all-targets -- -D warnings` | Passed |
| `cargo fmt -p rawdb -- --check` | Passed |
| `cargo doc --offline -p rawdb --no-deps` | Passed; seven public root exports unchanged |
| Affected-file `git diff --check` and visible-struct layout | Passed |
| Benchmark source hashes | Final production source and workload match the measured versions |

All builds used the SDK override recorded above. The existing cross-platform,
physical crash/power-loss, ignored stress-test, and full-indexer validation limits
remain. The new failure test deliberately exercises a mapping error, not an OS
fsync fault. No public API or on-disk format changed.

**Remaining work:** none identified for this bounded KISS review. `rawdb` is Done.

### 2026-09-19 — `rawdb` control-flow KISS review

Status: Done. Scanned the remaining allocation, write, file, and worker paths for
duplicated decisions and unnecessary intermediate state after the preceding
responsibility review. Validated behavior before production changes and measured
the affected allocation/batch workloads. Retained the allocation and flush
simplifications; rejected the slower chained batch loop.

**Scan and analysis**

- `Layout::is_at_end` independently reasons about the same three indexes as
  `Layout::end`. Compare the region's already-known end with the layout end, then
  reuse that boundary for append allocation. Remove the duplicate decision and
  the two last-entry wrappers used by it.
- `consume_hole` reads a hole and then looks it up again to remove it. The removal
  already returns its size; use that result directly.
- Ordered batches use a closure plus a separate first-item call. Tested replacing
  that with one loop over `once(first).chain(iter)`, retaining the empty fast path
  and panic guard. Rejected after benchmarking: see the comparison below.
- Flush has a separate clean-return branch whose return value is identical to
  the common path. Keep one result and one log that reports data and metadata.
- Keep geometric capacity growth unchanged. Replacing it with a different
  growth policy would change reserved-space behavior rather than just clarify
  the existing algorithm.
- Keep file remap recovery and worker coordination. Those branches represent
  distinct failure and ownership cases, unlike the duplicated decisions above.

**Tests and applied changes**

- Before production edits, all 96 rawdb unit tests and one doctest passed,
  including three new regressions. Allocation coverage exercises growth beside
  a live tail, an unflushed removal, a reusable hole large enough to consume, and
  a reusable hole too small for the requested growth; verify bytes and locations
  after reopening. Batch coverage exercises empty/single/overlapping writes with
  owned values and an iterator panic after its first completed write.
- Removed `is_at_end`, `get_last_region`, and `get_last_hole`; use the one layout
  boundary calculation for both in-place eligibility and append allocation.
  Reusable and pending holes continue contributing to that boundary.
- Hole consumption uses the size returned by removal. Flush uses one return and
  one log for clean or dirty work. Removed old debug-print scaffolding in the
  touched allocation tests.
- Keep the existing batch implementation and a short explanation for its
  separate first-item call. The panic guard, empty fast path, ordering and bounds
  checks are unchanged.

**Rejected batch simplification**

An identical before/after harness, five warmups per version, and four interleaved
rounds of five samples per version measured the chained batch loop at **0.831 ms**
versus **0.307 ms** for one million writes (about 2.7 times slower). The region
growth workload remained comparable at 1.999 ms versus 2.002 ms. This rejects
the chained loop on a measured hot path; it does not establish a general rule
about iterator performance. Raw measurements and candidate source hashes:
[`2026-09-19-kiss-rejected-batch-chain.json`](../benches/rawdb/2026-09-19-kiss-rejected-batch-chain.json).

**Retained-version benchmark**

Repeated the same warmup and interleaved comparison after restoring the original
batch loop. No audit builds ran during either timing comparison. Identical
harness sources and dependency lockfiles were checked; final production/workload
hashes match the measured versions. Full measurements:
[`2026-09-19-kiss-control-flow.json`](../benches/rawdb/2026-09-19-kiss-control-flow.json).

| Workload | Before median | Retained version median |
|---|---:|---:|
| One million ordered writes | 0.3065 ms | 0.3085 ms |
| Reuse 2,048 holes/slots | 1.897 ms | 1.949 ms |
| Grow 1,024 regions into holes, then relocate them | 1.9405 ms | 1.981 ms |
| 32 clean flushes over 1,024 regions | 0.092 ms | 0.091 ms |

Allocation/growth shifts are within 2.8% with overlapping sample ranges; this is
comparable local performance, not a speedup claim. The meaningful change is
**30 fewer production lines**, three fewer helpers, and one boundary calculation
instead of two independent implementations. All other benchmark controls remain
in the record, including noisy filesystem flush measurements.

**Final verification**

| Check | Result |
|---|---|
| `cargo test --offline -p rawdb -p vecdb --all-features` | 552 tests/doctests passed; six existing ignored cases |
| Plugin `compaction_completes_on_next_update_and_database_drop` | Passed |
| `cargo check --offline --workspace --all-targets` | Passed |
| `cargo clippy --offline -p rawdb --all-targets -- -D warnings` | Passed |
| `cargo fmt -p rawdb -- --check` | Passed |
| `cargo doc --offline -p rawdb --no-deps` | Passed; the same seven public root exports |
| Affected-file diff/whitespace checks and visible-struct layout | Passed |
| Benchmark source and workload hashes | Match the retained implementation |

All builds used the SDK override recorded above. The standalone benchmark uses
fat LTO, one codegen unit, native CPU targeting, and default panic unwinding;
both compared versions use identical settings. Existing cross-platform,
power-loss, ignored stress-test, and full-indexer validation limits remain.
No public API, on-disk format, growth policy, or flush ordering changed.

**Remaining work:** none for the changes retained in this pass. `rawdb` is Done.

### 2026-09-19 — `rawdb` registry KISS review

Status: Done. Replaced ID-to-slot plus slot-to-region lookup with a single
ID-to-region map. Each region already owns its immutable metadata slot;
the vacant-slot heap determines reuse. When no vacancies remain, live region
count is the next slot, so no replacement slot counter is needed.

**Scan and analysis**

- Check every slot-index consumer, reopen validation, removal ownership check,
  metadata shrink, and physical iteration before deleting the redundant vector.
- Preserve lowest-slot reuse, stable live slots, and reserve-before-mutation
  error handling. Derive metadata truncation from the highest live slot.
- Compaction can traverse the layout's existing address-ordered region handles,
  avoiding another registry lock and randomized map order for physical work.
- Retain file growth/recovery, runtime ownership, and the measured batch loop.
  This pass targets registry state rather than unrelated coordination changes.

**Tests and implementation**

- Added a regression before production edits: reopen a database with spare
  metadata slots, remove all live regions in reverse order, exhaust the vacant
  slots, then append more. Verify exact slot numbers and bytes after reopening.
  All 97 unit tests and one doctest passed before the change and after replacing
  the registry representation.
- Replaced `HashMap<String, usize>` plus `Vec<Option<Region>>` with
  `HashMap<String, Region>`. Removed slot-array maintenance, index lookup, and
  test-only collection getters. The vacancy heap remains the one mechanism for
  selecting the lowest reusable slot. No counter or replacement allocation was
  added.
- Slot selection still peeks before metadata reservation and commits the heap
  mutation only after reservation succeeds. Removal still validates handle
  identity and ownership; duplicate IDs and corrupt slots are still rejected.
- Metadata shrink derives the last live slot instead of trimming an array.
  Tests now assert region membership, stable slots, data, allocation layout, and
  persisted file size through the retained interfaces. Assertions about the
  removed array's shape were redundant and were removed.
- Retention and compaction traverse the existing physical layout in address
  order. Compaction no longer takes the registry read lock. Lookup is now the
  single `Regions::get`.
- The change removes one collection and 34 lines from non-test source files
  (including the removed test-only accessors). Public APIs and disk format are
  unchanged.

Added another regression for the ordered maintenance traversal: relocate a
low-slot region beyond later slots, reopen and retain selected regions, shrink
metadata, append a new slot, compact, and reopen again. Verify that physical
location and logical metadata slots remain independent, and every retained byte
survives.

**Benchmark and decision**

Keep the simpler registry. Run the full storage harness and a focused harness
with longer lookup/scan loops, each with five warmups per version followed by
four interleaved rounds of five samples per version. Paired dependency locks and
harness sources match; no audit builds ran during timing. Final source/workload
hashes match the retained implementation.

| Workload | Before median | After median |
|---|---:|---:|
| Append 64 MiB | 5.395 ms | 5.406 ms |
| One million ordered writes | 0.310 ms | 0.3055 ms |
| 100,000 small appends | 2.333 ms | 2.3085 ms |
| Grow/relocate 1,024 regions | 1.9755 ms | 2.0075 ms |
| Reuse 2,048 slots/holes | 2.121 ms | 2.0265 ms |
| Reopen 4,096 regions | 1.324 ms | 1.346 ms |
| Retain 2,048 regions and shrink metadata | 15.2925 ms | 15.9815 ms |
| Compact after retention | 4.622 ms | 4.2245 ms |

The full run's lookup median increased from 4.095 ms to 4.379 ms for 262,144
lookups; the focused run decreased from 15.386 ms to 13.391 ms for 1,048,576
lookups. The focused reopen median was 1.384 ms versus 1.385 ms. These differences
depend on the harness: do not claim a general lookup or reopen speedup. The
longer scan control measured 80.990 ms versus 82.4905 ms. Data-path timings remain
comparable; the concrete benefit is one fewer collection to allocate, maintain,
and keep consistent, plus simpler lookup and slot allocation. Metadata shrink
now scans live entries for the maximum slot, which is appropriate for this cold
maintenance operation instead of retaining an entire slot array.

Final measurements:
[`2026-09-19-kiss-registry.json`](../benches/rawdb/2026-09-19-kiss-registry.json),
[`2026-09-19-kiss-registry-focused.json`](../benches/rawdb/2026-09-19-kiss-registry-focused.json).
The focused record includes its complete standalone harness source. Initial
measurements before ordering retention are retained in the corresponding
`-unordered-retain.json` records; those describe the superseded candidate.

**Final verification**

| Check | Result |
|---|---|
| `cargo test --offline -p rawdb -p vecdb --all-features` | 554 tests/doctests passed; six existing ignored cases |
| Plugin `compaction_completes_on_next_update_and_database_drop` | Passed |
| `cargo check --offline --workspace --all-targets` | Passed |
| `cargo clippy --offline -p rawdb --all-targets -- -D warnings` | Passed |
| `cargo fmt -p rawdb -- --check` | Passed |
| `cargo doc --offline -p rawdb --no-deps` | Passed; same seven public root exports |
| Source/workload hashes, paired dependency locks, whitespace, and visible-struct layout | Verified |

Strict Clippy caught an unused must-use lookup result in the new benchmark.
Made its drop explicit, rebuilt both paired harnesses, repeated the final
measurements above, and reran the remaining checks. Production and test sources
did not change after the full test run.

All builds used the SDK override recorded above. Standalone benchmarks use fat
LTO, one codegen unit, native CPU targeting, and default panic unwinding on both
sides. Existing cross-platform, power-loss, ignored stress-test, and full-indexer
validation limits remain. No public API, on-disk format, or flush ordering changed.

**Remaining work:** none for this registry simplification. `rawdb` is Done.

### 2026-09-19 — `rawdb` design experiments and consumer contracts

**Scope:** audit, prototype, test, and benchmark the proposed leaner storage design.
Production crate sources were unchanged. Implementations and additional tests ran
in isolated snapshots. The earlier Done entries describe their bounded cleanup
passes; this broader design review has identified additional work.

The detailed findings and implementation order are in
[`RAWDB_DESIGN_AUDIT.md`](RAWDB_DESIGN_AUDIT.md). Reproducible sources, individual
measurements, and full test logs are in
[`2026-09-19-design`](../benches/rawdb/2026-09-19-design/README.md).
That evidence directory follows the existing ignored benchmark-run convention.

**Main findings:**

- Keep selective asynchronous writeback. Removing it regressed bulk checkpoints
  by about 32%; flushing the entire mapping regressed the 256 GiB sparse-file
  fixture from about 12 to 105 ms.
- A small page-granularity dirty-range prototype reduced fragmented individual
  updates from 137 to 61 ms while ordinary bulk/vector workloads stayed close to
  baseline. Tiny parallel appends had a higher median, so the result is not a
  universal throughput win.
- Capacity planning eliminated 63 MiB of relocation copies in the growth fixture
  and reduced peak RSS from 129.6 to 66.6 MiB. Existing indexer reservations already
  apply this principle in important places.
- Chunked compressed output reduced high-entropy write-phase peak RSS by about
  17 MiB against its parent implementation, without a clear throughput gain.
  Exact preallocation provided a smaller memory benefit with less API change.
- Deferred bounds failed 18 existing vecdb cases and one indexer rollback/reopen
  case. Nine vecdb failures expose the conditional vector-flush problem; nine
  rely on reopening unsynchronized writes. This requires an explicit checkpoint
  contract across crates.
- Fault injection exposed discarded pending values after write errors and an
  unrecoverable apparent clean retry after compressed page-metadata failure.
  Define predictable failure behavior before further API reduction.
- Mapping generations allowed unrelated growth while a reader stayed alive;
  their lifetime and retained-mapping costs remain a separate design decision.

**Validation:**

| Check | Result |
| --- | --- |
| Unchanged production rawdb/vecdb suites | 554 tests/doctests passed |
| Complete page-range prototype suites | 558 tests/doctests passed |
| Normally ignored concurrency stress tests | All three passed for each of ten configurations |
| Page-range and per-region-flag application integration | 87 tests passed for each; one documentation example ignored |
| Data/metadata sync errors, panic/iterator paths, randomized model | Passed in the relevant complete suites |
| Process kill after acknowledged sync, then reopen | 50 checks passed across ten configurations |
| Timed harness | 1,043 measured runs, 149 warmups, 17 workloads |
| Existing storage benchmark | 45 iterations across three configurations |
| Snapshot reconstruction | All rawdb/vecdb files reproduced byte for byte in twelve snapshots |

Three documentation examples remain ignored in the rawdb/vecdb suites after the
three stress tests are run separately. The two metadata variants retain their
recorded failures. Preliminary shared-target cache reuse was detected and those
results were discarded; reported comparisons use isolated verified binaries.

**Remaining work:** review and implement the checkpoint/failure contract and the
selected narrow improvements. No production redesign was applied in this audit.
Power-loss behavior, Linux/FreeBSD runtime validation, and complete historical
indexer throughput remain unmeasured.
