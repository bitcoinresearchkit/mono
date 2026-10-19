# brk_store

Key-value storage layer built on fjall for Bitcoin indexing.

- **Write batching**: The caller decides the batch: puts/deletes accumulate in memory (unbounded) until it takes them as an owned ingestion, whose successful completion persists and publishes them
- **Fixed-width records**: Key and value types define their encoded arrays for ingestion, reads, and background compaction; `Unit` uses zero bytes
- **Version management**: Automatic schema-version validation when opening a store

## Core API

```rust,ignore
let mut store: Store<TxidPrefix, Height> = Store::import(
    &db, &path, "txid_to_height",
    Version::new(1), Kind::Random
)?;

store.insert(txid, height);
if let Some(ingest) = store.take_pending_ingest() {
    ingest.run()?;
}

let height = store.get(&txid)?;
```

## Access Patterns

| Kind | Use Case | Configuration |
|------|----------|---------------|
| `Random` | Random point lookups | Bloom filters and pinned filter blocks |
| `Recent` | Point lookups expected to hit | Point-read hints and Bloom filters |
| `Vec` | Prefix and range scans | Restart interval 8; no filters or block pinning |
