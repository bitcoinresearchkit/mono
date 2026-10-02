# vecdb

Typed persistent vectors built on [`rawdb`](../rawdb/README.md) for large,
fixed-width datasets.

VecDB is designed for append-heavy sequences addressed by integer-like index
types. Writes are buffered, readers can scan or access ranges, and stored
vectors can retain stamped changes for explicit rollback. It is not a
key-value database or an ACID transaction layer.

## Choose a vector

| Type | Use |
|---|---|
| `BytesVec<I, T>` | Portable, fixed-width values implementing `Bytes` |
| `ZeroCopyVec<I, T>` | Native-layout mmap reads for zerocopy-compatible values |
| `PcoVec<I, T>` | Numeric data compressed with pco |
| `LZ4Vec<I, T>` | Fast general-purpose compression |
| `ZstdVec<I, T>` | Denser general-purpose compression |
| `MutableVec<V>` | Updates and sparse deletions over a raw stored vector |
| `OverflowVec<I, T>` | Compact common values with a stored overflow path |
| `EagerVec<V>` | Incrementally computed results stored on disk |
| `LazyVec<I, T, SI, ST>` | A cheap, read-only derivation from one source vector |

`BytesVec` is the default starting point. Choose another representation only
when its layout or access behavior provides a concrete benefit. Lazy vectors
have exactly one source; computations that require multiple inputs should have
an explicit stored source of truth.

## Install

```bash
cargo add vecdb
```

No optional feature is enabled by default. Enable the representation or
integration you use, for example:

```bash
cargo add vecdb --features pco,derive
```

## Basic use


The tuple `(database, name, version)` identifies stored data. Import validates
its on-disk schema; `forced_import` resets incompatible data when the caller
explicitly wants rebuild behavior.

## Reads, writes, and rollback

- `push` appends to the in-memory write buffer.
- `write` publishes buffered changes to the backing regions.
- `flush` publishes the vector's changes and synchronizes its database.
- `Database::flush` synchronizes all dirty data and shared metadata.
- `reader` creates a read handle for repeated random access.
- `collect`, `collect_range`, folds, and iterators provide sequential access.
- `truncate_if_needed` removes a suffix without changing earlier indexes.

Readers and cursors keep their backing regions stable. Drop them before writing
or truncating those same regions, and before growing the database mapping.
Writes within another region's existing capacity can proceed while readers
are alive. Buffered file readers use the same region lock as mmap readers.

Wrap `BytesVec` or `ZeroCopyVec` in `MutableVec` when existing positions must be
replaced or deleted. Deletions leave holes, so later indexes do not move.

Import options can set `saved_stamped_changes`;
`stamped_write_with_changes` then preserves a bounded rollback history before
overwriting values. Periodic writes inside compute loops use
`stamped_write_maybe_with_changes(stamp, false)` to advance the baseline without
undo history. The final update write saves changes; `rollback` and
`rollback_before` run when preparing an update, before its compute loop.

A failed or unwound write fences the writer, including failures in a mutable or
overflow sidecar. Further writes return `WriteFailed`; infallible mutation
methods panic. Resetting pending state cannot clear the fence. Discard the
affected handles and validate or rebuild the data before continuing. In-place
writes are not crash-atomic, and reopening does not itself repair a partial write.

## Range retention

Stored vectors accept a cache policy as their third type parameter:

- `NoCache` (the default) has no cache allocation or cache-side locking.
- `Budgeted` shares one immutable, process-wide byte limit across sources.


Read-only and type-erased clones share the source's ranges. Use ordinary point,
range, sorted, and fold reads; there is no shared whole-vector snapshot API.

Cache retention is always evictable. Algorithms that need a working set own
ordinary read results and pass references through their computation context.
Those buffers remain valid after cache eviction without pinning cache entries.
Reads retain only requested ranges, not physical decoder overread. Isolated
values stay inline in a sorted span directory; uniquely owned tails can extend
in place without copying existing history.
When the shared budget is full, reclamation rotates through source owners and
evicts all of each selected source's ranges. Recently used sources get one second
chance, busy sources are skipped, and a read stays uncached if two bounded passes
cannot free enough space.
Source writes invalidate changed suffixes and preserve unchanged prefixes.
Successful persisted writes can extend an already-retained tail, but do not warm
cold caches. `Budgeted::global()?.clear()` evicts retained data without changing
source data. Missing or repeated initialization is an error; a zero-byte
budget disables retention. `NoCache` needs no initialization or global lookup.

Global and per-source charges account for allocation capacity and follow buffer
lifetimes, including buffers still borrowed by a fold after eviction.
Caller-owned results and decoder scratch are outside this
retention budget. Lazy readers use their stored sources' caches without retaining
another copy of derived values or requiring separate cache invalidation.
Cache protection covers an individual source operation, not a transaction across
multiple vectors: related reads still require the application's publication guard.

## Value and index types

Values are fixed width. Numeric primitives and the supported fixed byte arrays
work directly with the relevant representation. Custom portable values
implement `Bytes`; custom pco values implement `Pco`; zero-copy values satisfy
the zerocopy traits used by `ZeroCopyVec`. The optional `derive` feature exports
`#[derive(Bytes)]` and `#[derive(Pco)]`.

Indexes implement `VecIndex`. Using domain-specific newtypes instead of
`usize` keeps unrelated vector axes distinct at compile time.

## Features

| Feature | Enables |
|---|---|
| `derive` | `Bytes` and `Pco` derive macros |
| `pco` | `PcoVec` |
| `zerocopy` | `ZeroCopyVec` |
| `lz4` | `LZ4Vec` |
| `zstd` | `ZstdVec` |
| `serde` | Serialization support for public metadata types |
| `schemars` | JSON Schema support for public metadata types |

## Examples and benchmarks

- [`examples/bench.rs`](examples/bench.rs) compares the available storage
  representations on a chosen workload.

```bash
cargo run --release -p vecdb --example bench --features pco,lz4,zstd,zerocopy
```
