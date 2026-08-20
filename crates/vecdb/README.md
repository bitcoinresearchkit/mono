# vecdb

High-performance mutable persistent vectors built on [`rawdb`](../rawdb/README.md).

## Features

- **Vec-like API**: `push` and `truncate`, plus `update` and sparse deletion through `MutableVec`
- **Multiple storage formats**:
  - **Raw**: `BytesVec`, `ZeroCopyVec` (uncompressed)
  - **Compressed**: `PcoVec`, `LZ4Vec`, `ZstdVec`
- **Computed vectors**: `EagerVec` (stored computations), `LazyVecFrom1/2/3` (on-the-fly computation)
- **Rollback support**: Time-travel via stamped change deltas without full snapshots
- **Sparse deletions**: Delete elements leaving holes, no reindexing required
- **Thread-safe**: Concurrent reads with exclusive writes
- **Blazing fast**: See the local [benchmark suite](examples/bench.rs)
- **Lazy persistence**: Changes buffered in memory, persisted only on explicit `flush()`

## Not Suited For

- **Key-value storage** - Use [`fjall`](https://crates.io/crates/fjall) or [`redb`](https://crates.io/crates/redb)
- **Variable-sized types** - Types like `String`, `Vec<T>`, or dynamic structures
- **ACID transactions** - No transactional guarantees (use explicit rollback instead)

## Install

```bash
cargo add vecdb
```

## Quick Start

```rust,no_run
use vecdb::{
    AnyStoredVec, AnyVec, BytesVec, Database, WritableVec,
    ImportableVec, ReadableVec, Result, Version
};
use std::path::Path;

fn main() -> Result<()> {
    // Open database
    let db = Database::open(Path::new("data"))?;

    // Create vector with index type usize and value type u64
    let mut vec: BytesVec<usize, u64> =
        BytesVec::import(&db, "my_vec", Version::TWO)?;

    // Push values (buffered in memory)
    for i in 0..1_000_000 {
        vec.push(i);
    }

    // Flush writes to rawdb region and syncs to disk
    vec.flush()?;  // Calls write() internally then flushes region
    db.flush()?;   // Syncs database metadata

    // Sequential scan via fold
    let sum = vec.fold_range(0, vec.len(), 0u64, |acc, v| acc.wrapping_add(v));

    // Random access via reader
    let reader = vec.reader();
    for i in [500, 1000, 10] {
        println!("vec[{}] = {}", i, reader.get(i));
    }

    Ok(())
}
```

## Type Constraints

vecdb works with **fixed-size types**:
- Numeric primitives: `u8`, `i32`, `f64`, etc.
- Fixed arrays: `[T; N]`
- Structs with `#[repr(C)]`
- Types implementing `zerocopy::FromBytes + zerocopy::AsBytes` (for `ZeroCopyVec`)
- Types implementing `Bytes` trait (for `BytesVec`, `LZ4Vec`, `ZstdVec`)
- Numeric types implementing `Pco` trait (for `PcoVec`)

Use `#[derive(Bytes)]` or `#[derive(Pco)]` from `vecdb_derive` to enable custom wrapper types.

## Vector Variants

### Raw (Uncompressed)

**`BytesVec<I, T>`** - Append-only custom serialization via `Bytes` trait
```rust,ignore
use vecdb::{BytesVec, Bytes};

#[derive(Bytes)]
struct UserId(u64);

let mut vec: BytesVec<usize, UserId> =
    BytesVec::import(&db, "users", Version::TWO)?;
```

**`ZeroCopyVec<I, T>`** - Append-only zero-copy mmap access (fastest random reads)
```rust,ignore
use vecdb::ZeroCopyVec;

let mut vec: ZeroCopyVec<usize, u32> =
    ZeroCopyVec::import(&db, "raw", Version::TWO)?;
```

Wrap either raw vector in **`MutableVec<V>`** when existing values must be
updated or deleted:

```rust,ignore
use vecdb::{BytesVec, MutableVec};

let mut vec: MutableVec<BytesVec<usize, u64>> =
    MutableVec::import(&db, "mutable", Version::TWO)?;
```

### Compressed

**`PcoVec<I, T>`** - Pcodec compression (best for numeric data, excellent compression ratios)
```rust,ignore
use vecdb::PcoVec;

let mut vec: PcoVec<usize, f64> =
    PcoVec::import(&db, "prices", Version::TWO)?;
```

**`LZ4Vec<I, T>`** - LZ4 compression (fast, general-purpose)
```rust,ignore
use vecdb::LZ4Vec;

let mut vec: LZ4Vec<usize, [u8; 16]> =
    LZ4Vec::import(&db, "hashes", Version::TWO)?;
```

**`ZstdVec<I, T>`** - Zstd compression (high compression ratio, general-purpose)
```rust,ignore
use vecdb::ZstdVec;

let mut vec: ZstdVec<usize, u64> =
    ZstdVec::import(&db, "data", Version::TWO)?;
```

### Computed Vectors

**`EagerVec<V>`** - Wraps any stored vector to enable eager computation methods

Stores computed results on disk, incrementally updating when source data changes. Use for derived metrics, aggregations, transformations, moving averages, etc.

```rust,ignore
use vecdb::EagerVec;

let mut derived: EagerVec<BytesVec<usize, f64>> =
    EagerVec::import(&db, "derived", Version::TWO)?;

// Compute methods store results on disk
// derived.compute_add(&source1, &source2)?;
// derived.compute_sma(&source, 20)?;
```

**`LazyVecFrom1/2/3<...>`** - Lazily computed vectors from 1-3 source vectors

Values computed on-the-fly during iteration, nothing stored on disk. Use for temporary views or simple transformations.

```rust,ignore
use vecdb::LazyVecFrom1;

let lazy = LazyVecFrom1::init(
    "computed",
    Version::TWO,
    Box::new(source.clone()),  // ScannableBoxedVec
    |_i, v| v * 2,
);

// Computed on-the-fly via ReadableVec trait, not stored
lazy.for_each(|value| {
    // ...
});
```

## Core Operations

### Write and Persistence

```rust,ignore
// Push values (buffered in memory)
vec.push(42);
vec.push(100);

// write() moves pushed values to storage (visible for reads)
vec.write()?;

// flush() calls write() + region().flush() for durability
vec.flush()?;
db.flush()?;   // Also flush database metadata
```

### Updates and Deletions

```rust,ignore
use vecdb::{BytesVec, MutableVec};

let mut vec: MutableVec<BytesVec<usize, u64>> =
    MutableVec::import(&db, "mutable", Version::TWO)?;

// Update element at index (works on stored data)
vec.update(5, 999)?;

// Delete element (creates a hole at that index)
let reader = vec.reader();
let _ = vec.take(10, &reader);
drop(reader);

// Holes are tracked and can be checked
if vec.holes().contains(&10) {
    println!("Index 10 is a hole");
}

// Reading a hole returns None
let reader = vec.reader();
assert_eq!(vec.get_with_reader(10, &reader), None);
```

### Rollback with Stamps

Rollback uses stamped change deltas - lightweight compared to full snapshots.

```rust,ignore
use vecdb::Stamp;

// Create initial state
vec.push(100);
vec.push(200);
vec.stamped_write_with_changes(Stamp::new(1))?;

// Make more changes
vec.push(300);
vec.update(0, 999)?;
vec.stamped_write_with_changes(Stamp::new(2))?;

// Rollback to previous stamp (undoes changes from stamp 2)
vec.rollback()?;
assert_eq!(vec.stamp(), Stamp::new(1));

// Rollback before a stamp (undoes everything including stamp 1)
vec.rollback_before(Stamp::new(1))?;
assert_eq!(vec.stamp(), Stamp::new(0));
```

Configure number of stamps to keep:
```rust,ignore
let options = (&db, "vec", Version::TWO)
    .into()
    .with_saved_stamped_changes(10);  // Keep last 10 stamps
let vec = BytesVec::import_with(options)?;
```

## When To Use

**Perfect for:**
- Storing large `Vec`s persistently on disk
- Append-only or append-mostly workloads
- High-speed sequential reads
- High-speed random reads (improved with `ZeroCopyVec`)
- Space-efficient storage for numeric time series (improved with `PcoVec`)
- Sparse deletions without reindexing
- Lightweight rollback without full snapshots
- Derived computations stored on disk (with `EagerVec`)

**Not ideal for:**
- Heavy random write workloads
- Frequent insertions in the middle
- Variable-length data (strings, nested vectors)
- ACID transaction requirements
- Key-value lookups (use a proper key-value store)

## Feature Flags

No features are enabled by default. Enable only what you need:

```bash
cargo add vecdb  # BytesVec only, no compression or optional features
```

Available features:
- `pco` - Pcodec compression support (`PcoVec`)
- `zerocopy` - Zero-copy mmap access (`ZeroCopyVec`)
- `lz4` - LZ4 compression support (`LZ4Vec`)
- `zstd` - Zstd compression support (`ZstdVec`)
- `derive` - Derive macros for `Bytes` and `Pco` traits
- `serde` - Serde serialization support
- `serde_json` - JSON output using serde_json
- `sonic-rs` - Faster JSON using sonic-rs

With Pcodec compression:
```bash
cargo add vecdb --features pco,derive
```

With all compression formats:
```bash
cargo add vecdb --features pco,zerocopy,lz4,zstd,derive
```

## Examples

Comprehensive examples in [`examples/`](examples/):
- [`zerocopy.rs`](examples/zerocopy.rs) - MutableVec over ZeroCopyVec with holes, updates, and rollback
- [`pcodec.rs`](examples/pcodec.rs) - PcoVec with compression

Run examples:
```bash
cargo run --example zerocopy --features zerocopy
cargo run --example pcodec --features pco
```

## Benchmarks

> 10B sequential `u64` values (80 GB), Apple Silicon, `--release`. Compression ratios reflect sequential data — real-world ratios will vary.

| Type | Disk | Write | Read |
|------|------|-------|------|
| `BytesVec` | 80.0 GB | 1.8 GB/s | 6.7 GB/s |
| `ZeroCopyVec` | 80.0 GB | 1.7 GB/s | 6.7 GB/s |
| `PcoVec` | 181 MB | 0.4 GB/s | 7.7 GB/s |
| `LZ4Vec` | 40.1 GB | 0.4 GB/s | 3.0 GB/s |
| `ZstdVec` | 10.4 GB | 0.5 GB/s | 1.0 GB/s |

```bash
cargo run --release --example bench -p vecdb --features pco,lz4,zstd,zerocopy
BENCH_COUNT=100_000_000 cargo run ...  # smaller dataset
```

### PcoVec SIMD (x86_64)

For best `PcoVec` decompression on x86_64, enable BMI and AVX2:

```bash
RUSTFLAGS="-C target-feature=+bmi1,+bmi2,+avx2" cargo build --release
```
