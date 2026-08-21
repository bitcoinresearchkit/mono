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
| `ColumnarVec<V, C>` | One row index split into independently readable typed columns |
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

```rust,no_run
use std::path::Path;

use vecdb::{
    AnyStoredVec, AnyVec, BytesVec, Database, ImportableVec, ReadableVec,
    Result, Version, WritableVec,
};

fn main() -> Result<()> {
    let db = Database::open(Path::new("data"))?;
    let mut values: BytesVec<usize, u64> =
        BytesVec::import(&db, "values", Version::ONE)?;

    values.push(21);
    values.push(34);
    values.flush()?;
    db.flush()?;

    assert_eq!(values.collect_range(0, 2), vec![21, 34]);
    Ok(())
}
```

The tuple `(database, name, version)` identifies stored data. Import validates
its on-disk schema; `forced_import` resets incompatible data when the caller
explicitly wants rebuild behavior.

## Reads, writes, and rollback

- `push` appends to the in-memory write buffer.
- `write` publishes buffered changes to the backing regions.
- `flush` writes and synchronizes the vector's regions.
- `Database::flush` synchronizes database metadata.
- `reader` creates a read handle for repeated random access.
- `collect`, `collect_range`, folds, and iterators provide sequential access.
- `truncate_if_needed` removes a suffix without changing earlier indexes.

Wrap `BytesVec` or `ZeroCopyVec` in `MutableVec` when existing positions must be
replaced or deleted. Deletions leave holes, so later indexes do not move.

Import options can set `saved_stamped_changes`; stamped writes then preserve a
bounded rollback history. `rollback` and `rollback_before` restore prior
states. Rollback is explicit recovery machinery, not a multi-vector
transaction.

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
| `serde_json` | JSON output through `serde_json` |
| `sonic-rs` | JSON output through `sonic-rs` |

## Examples and benchmarks

- [`examples/zerocopy.rs`](examples/zerocopy.rs) demonstrates mutable
  zero-copy storage, holes, updates, and rollback.
- [`examples/pcodec.rs`](examples/pcodec.rs) demonstrates pco-compressed
  storage.
- [`examples/bench.rs`](examples/bench.rs) compares the available storage
  representations on a chosen workload.

```bash
cargo run -p vecdb --example zerocopy --features zerocopy
cargo run -p vecdb --example pcodec --features pco
cargo run --release -p vecdb --example bench --features pco,lz4,zstd,zerocopy
```
