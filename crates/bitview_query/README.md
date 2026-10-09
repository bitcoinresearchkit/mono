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
// `query` is the `&Query` an `AsyncQuery::run` closure receives (see Async Usage).

// Series queries use a cheap resolve phase before formatting.
let selection = SeriesSelection::from((
    Index::Height,
    SeriesName::from("circulating_supply"),
    DataRangeFormat::default(),
));
let resolved = query.resolve(selection, usize::MAX)?;
let data = query.format(resolved)?;

// Block queries
let hash = query.resolve_block_hash(Height::new(840_000))?;
let status = query.block_status(&hash)?;

// Transaction queries
let tx = query.resolve_transaction(&txid)?;

// Address queries
let stats = query.addr(address)?;
```

## Query Types

| Domain | Methods |
|--------|---------|
| Series | `search_series`, `resolve`, `format`, `series_count`, `series_list`, `series_catalog`, `series_info` |
| Blocks | `resolve_block_hash`, `resolve_block_v1`, `resolve_blocks`, `resolve_block_by_timestamp`, `block_status` |
| Transactions | `resolve_transaction`, `transaction_status`, `transaction_hex_resolved`, `outspend_json`, `outspends_json` |
| Addresses | `addr`, `addr_txs_resolved`, `addr_utxos_resolved`, `addr_mempool_txs`, `addr_hash_prefix_matches` |
| Mining | `difficulty_adjustments`, `hashrate`, `mining_pools`, `reward_stats` |
| Mempool | `mempool_info_json`, `recommended_fees`, `mempool_blocks` |

## Async Usage

```rust,ignore
let async_query = AsyncQuery::build(&plugins, mempool);

// Run a read with deadline-bounded publication protection.
let result = async_query.read(move |q| q.resolve_block_hash(height)).await;
```

`with_deadline(Instant)` creates a cheap request-local query view sharing the
same data. Publication and prefix-lock waits use its remaining budget, and
expired blocking jobs return `ReadTimeout` before invoking their closure. The
server supplies one HTTP deadline; standalone async reads have a four-second
total budget. Synchronous reads retain a four-second per-wait limit.

`read` and `read_with_admission` execute the query closure once on a blocking
worker. Mutable reads wait for the shared pipeline publication guard within the
remaining deadline; incompatible chain/mempool snapshots return `StateUpdating`
immediately. Resolve snapshots inside the closure. `run` remains appropriate for
actions and already-prepared immutable work. Running work keeps its guards and
admission until it finishes, even after HTTP cancellation. No publication
notifications or async runtime are required by the synchronous data layer.

## Confirmed transaction reads

Public transaction methods acquire their own publication protection. Internally,
confirmed-position resolution and handoff revalidation are methods on an
`IndexerRead` view that owns the query's shared pipeline guard. Keep the view alive
through all dependent reads; do not reacquire a guard inside that scope. Proven
immutable-prefix reads instead retain the separate rollback pin and can continue
during an append-only update.

`resolve_confirmed_tx_guarded`, `resolve_confirmed_position`, and
`revalidate_confirmed_tx` are no longer public `Query` methods. Use
`resolve_confirmed_tx`, `resolve_tx`, and the resolved transaction/proof/CPFP
operations instead. Resolved tokens do not retain a lock across async handoffs;
the consuming operation reacquires protection and revalidates the block hash.
Immutable block snapshots still use their separate safe-length protection.

Raw block payload and size helpers borrow the snapshot's `SafeLengths` guard,
not a copied `Lengths` value. They are internal to the block-query module; public
callers use `block_raw` or `ResolvedBlocks::anchor_raw` / `anchor_raw_size`.
The selected height is checked against the pinned prefix before reading the
indexed record. HEAD still checks framing and identity without reading the full
payload, and HTTP revalidation still precedes body preparation.

## Series read bounds

`search` returns a `SeriesRead` that owns the selected plugin guards and published
bounds. `ResolvedQuery` retains that same view through formatting. Both expose
`columns()` as bounded readers, never raw vectors; those readers cannot outlive
their owning view. `Query::weight` accepts the view rather than unbounded vectors.

Each column operation applies its limits automatically: length, latest values,
JSON, CSV, and row writers all use the same published prefix. Binding a vector
without a limit for its index fails closed. Nested lazy inputs still receive
bounds through vecdb's internal thread-local scope, installed by every bounded
operation rather than by callers. Ordinary storage/compute reads remain unbounded.

## Errors

Queries return `bitview_query::Result`. `Error` is the API's vocabulary: invalid
requests, missing data, temporary unavailability and internal failures. Errors
from lower layers arrive as `Error::Lower` and are always internal; the conditions
those layers name on purpose (an updating mempool, invalid address input, an
unindexable date) keep their meaning through `From<brk_error::Error>`.

## Built On

- `bitview_runtime::PluginSet` for generic plugin discovery
- `brk_mempool` for mempool queries
- `brk_cpfp` for confirmed CPFP clusters (`chain`)
- `brk_reader` for raw block access

## Features

The API features select endpoints and the plugins those endpoints read: `chain` (blocks, transactions, addresses, mining, mempool, UTXO set; implies
`price`), `series`, `price` (oracle prices) and `urpd` (implies `price`); `full-api` enables all of
them. Each plugin an enabled API feature reads adds its typed `HasX` requirement to
`QueryPluginSet` and exposes its typed accessor.

`full-api` is the default for standalone users; composition and adapter crates
should disable default features and select only what they expose. Generic
`Vecs` discovery (the `series` feature) works with any `Traversable` plugin
without a dependency on that plugin crate. Query construction validates the enabled
capabilities once and then keeps direct typed references, so hot paths perform
no dynamic lookup.
