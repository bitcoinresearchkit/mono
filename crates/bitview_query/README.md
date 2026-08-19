# bitview_query

Query interface for Bitcoin indexed and computed data.

## What It Enables

Query blocks, transactions, addresses, and 1000+ on-chain metrics through a unified API. Supports pagination, range queries, and multiple output formats.

## Key Features

- **Unified access**: Single entry point to plugin and mempool data
- **Metric discovery**: List metrics, filter by index type, fuzzy search
- **Range queries**: By height, date, or relative offsets (`from=-100`)
- **Multi-metric bulk queries**: Fetch multiple metrics in one call
- **Async support**: Tokio-compatible with `AsyncQuery` wrapper
- **Format flexibility**: JSON, CSV, or raw values

## Core API

```rust,ignore
let query = Query::build(&plugins, Some(mempool));

// Current height
let height = query.height();

// Metric queries (two-phase: resolve then format)
let resolved = query.resolve(MetricSelection {
    metrics: vec!["supply".into()],
    index: Index::Height,
    range: DataRangeFormat::default(),
}, usize::MAX)?;
let data = query.format(resolved)?;

// Block queries
let info = query.block_by_height(Height::new(840_000))?;

// Transaction queries
let tx = query.transaction(txid.into())?;

// Address queries
let stats = query.address(address)?;
```

## Query Types

| Domain | Methods |
|--------|---------|
| Metrics | `metrics`, `resolve`, `format`, `metric_to_indexes` |
| Blocks | `block`, `block_by_height`, `blocks`, `block_txs`, `block_status`, `block_by_timestamp` |
| Transactions | `transaction`, `transaction_status`, `transaction_hex`, `outspend`, `outspends` |
| Addresses | `address`, `address_txids`, `address_utxos` |
| Mining | `difficulty_adjustments`, `hashrate`, `mining_pools`, `reward_stats` |
| Mempool | `mempool_info`, `recommended_fees`, `mempool_blocks` |

## Async Usage

```rust,ignore
let async_query = AsyncQuery::build(&plugins, mempool);

// Run blocking queries in thread pool
let result = async_query.run(|q| q.block_by_height(height)).await;

// Access inner Query
let height = async_query.inner().height();
```

## Recommended: mimalloc v3

Use [mimalloc v3](https://crates.io/crates/mimalloc) as the global allocator. Query operations involve many short-lived allocations; mimalloc handles this with less fragmentation and lower peak memory than the system allocator.

## Built On

- `bitview_runtime::PluginSet` for generic plugin discovery
- `brk_mempool` for mempool queries
- `brk_reader` for raw block access

## Features

The default `full` feature preserves the complete query API. Smaller consumers
can select `chain`, `series`, or `urpd`. Construction validates the plugins
required by the enabled features once and then keeps direct typed references;
queries do not perform dynamic plugin lookup on the hot path.
