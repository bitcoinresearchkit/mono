# brk_store

Key-value storage layer built on fjall for Bitcoin indexing.

## What It Enables

Persist and query Bitcoin index data (address→outputs, txid→height, etc.) with access patterns optimized for different workloads: random lookups, sequential scans, and recent-data queries.

## Key Features

- **Workload-optimized configs**: `Kind::Random` (bloom filters, pinned blocks), `Kind::Recent` (point-read optimized), and `Kind::Vec` (append-heavy)
- **Write batching**: Accumulate puts/deletes in memory, then move them into an owned ingestion batch
- **Tiered caching**: Optional bounded in-memory batches before hitting disk
- **Version management**: Automatic schema-version validation when opening a store

## Core API

```rust,ignore
let mut store: Store<Txid, Height> = Store::import(
    &db, &path, "txid_to_height",
    Version::new(1), Mode::Any, Kind::Random
)?;

store.insert(txid, height);
if let Some(ingest) = store.take_pending_ingest() {
    ingest()?;
}

let height = store.get(&txid)?;
```

## Access Patterns

| Kind | Use Case | Optimization |
|------|----------|--------------|
| `Random` | UTXO lookups, txid queries | Aggressive bloom filters |
| `Recent` | Mempool, recent blocks | Point-read hints |
| `Vec` | Append-heavy series | Dense blocks, no filters or pinned blocks |

## Built On

- `brk_error` for error handling
- `brk_types` for `Height`, `Version`
