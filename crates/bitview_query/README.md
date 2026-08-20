# bitview_query

Query interface for Bitcoin indexed and computed data.

## What It Enables

Query blocks, transactions, addresses, and on-chain series through a unified
API. Supports pagination, range queries, and multiple output formats.

## Key Features

- **Unified access**: Single entry point to plugin and mempool data
- **Series discovery**: Browse the catalog, inspect supported indexes, and fuzzy search
- **Range queries**: By height, date, or relative offsets (`from=-100`)
- **Bulk queries**: Fetch multiple series in one call
- **Async support**: Tokio-compatible with `AsyncQuery` wrapper
- **Format flexibility**: JSON, CSV, or raw values

## Core API

```rust,ignore
let query = Query::build(&plugins, Some(mempool));

// Current height
let height = query.height();

// Series queries use a cheap resolve phase before formatting.
let selection = SeriesSelection::from((
    Index::Height,
    SeriesName::from("supply"),
    DataRangeFormat::default(),
));
let resolved = query.resolve(selection, usize::MAX)?;
let data = query.format(resolved)?;

// Block queries
let info = query.block_by_height(Height::new(840_000))?;

// Transaction queries
let tx = query.transaction(txid.into())?;

// Address queries
let stats = query.addr(address)?;
```

## Query Types

| Domain | Methods |
|--------|---------|
| Series | `search_series`, `resolve`, `format`, `series_count`, `series_list`, `series_catalog`, `series_info` |
| Blocks | `block`, `block_by_height`, `blocks`, `block_txs`, `block_status`, `block_by_timestamp` |
| Transactions | `transaction`, `transaction_status`, `transaction_hex`, `outspend`, `outspends` |
| Addresses | `addr`, `addr_txids`, `addr_utxos` |
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

Plugin features (`indexer`, `blocks`, `distribution`, `mappings`, `price`, and
the other built-in plugin IDs) are the source of truth. Enabling one adds its
typed `HasX` requirement to `QueryPluginSet` and exposes its typed accessor.

`chain`, `series`, `urpd`, and `full` are convenience aggregators. `full` is the
default for standalone users; composition and adapter crates should disable
default features and select only what they expose. Generic `Vecs` discovery
works with any `Traversable` plugin without a feature or dependency on that
plugin crate. Query construction validates the enabled capabilities once and
then keeps direct typed references, so hot paths perform no dynamic lookup.
