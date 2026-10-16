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
| Transactions | `resolve_transaction`, `resolve_confirmed_tx`, `transaction_status`, `outspend_json`, `outspends_json` |
| Addresses | `addr`, `resolve_addr_txs`, `resolve_addr_utxos`, `resolve_addr_mempool_txs`, `addr_hash_prefix_matches` |
| Mining | `difficulty_adjustments`, `hashrate`, `mining_pools`, `reward_stats` |
| Mempool | `mempool_info_json`, `recommended_fees`, `precise_fees`, `mempool_blocks` |

## Async Usage

```rust,ignore
let async_query = AsyncQuery::build(&plugins, mempool);

// Run a read with deadline-bounded publication protection.
let result = async_query.read(move |q| q.resolve_block_hash(height)).await;
```

`with_deadline(Instant)` creates a cheap request-local query view sharing the
same data. Admission waits use its remaining budget, and expired blocking jobs
return `ReadTimeout` before invoking their closure. The server supplies one HTTP
deadline; standalone async reads have a four-second total budget.

Every read pins the published state once (`SafeLengths`: the lengths every plugin
has computed to, the tip, a revision) and reads below it: the pin bounds its
thread's reads, lazy calendar and count views included, while it lives. Pinning never waits and
an update never waits for readers: it appends past the pin and publishes by
swapping the pin's source; only a rollback waits for the pins taken before it.
Data rewritten in place is read consistently with the pin: the spend index masks
spends past it, an update publishes its changed addresses from memory with its
lengths, under a short lock address readers take before pinning, and writes them
to the address state afterward (a catch-up's or rollback's write holds that lock,
and address reads answer 503 meanwhile), and UTXO set snapshots are replaced by
rename. The mempool publishes only at the published tip and is loaded before the
pin, so it is at the pin's tip, behind it, or (mid-reorg) on the branch being
replaced: a lookup the chain answers first reads it as is (it only adds what is
still pending), while address views, which add it to the chain's numbers, use
the confirmed state alone until it is at the pin's tip.

`read` and `read_with_admission` execute the query closure once on a blocking
worker. Resolve snapshots inside the closure. `run` remains appropriate for
actions and already-prepared immutable work. Running work keeps its pin and
admission until it finishes, even after HTTP cancellation. No publication
notifications or async runtime are required by the synchronous data layer.

## Resolve, then build

Reads served after an async handoff split in two. A `resolve_*` method pins the publication,
finds the exact source and returns a token carrying its representation identity, so HTTP
revalidation can answer before any body is built. The matching `*_resolved` method (or
`format` for series) then builds the body.

Transaction tokens (`resolve_transaction`, `resolve_confirmed_tx`, `resolve_cpfp`) hold no
pin: a mempool body travels with the token, and a confirmed one is named by its position and
block hash. Consuming it (`transaction_json_resolved`,
`transaction_raw_resolved`, `transaction_hex_resolved`, `merkle_proof_resolved`,
`merkleblock_proof_resolved`, `cpfp_json_resolved`) takes one new `ReadPin`, checks that the
block is still at that height (a token from a replaced block answers `UnknownTxid`, or
`StateUpdating` while a lowered publication hides it), and does every dependent read under that
pin.

`resolve` for series instead keeps its pin: the `ResolvedQuery` holds the selected series and
the publication's bounds until `format` (or `format_bulk`, `format_raw`) consumes it, so
formatting reads exactly the rows `resolve` counted. Every column read is bounded by that
publication, nested lazy inputs included, while ordinary storage and compute reads stay
unbounded.

## Errors

Queries return `bitview_query::Result`. `Error` is the API's vocabulary: invalid
requests, missing data, temporary unavailability and internal failures. Errors
from lower layers arrive as `Error::Lower` and are always internal; the conditions
those layers name on purpose (an updating mempool, invalid address input, an
unindexable date) keep their meaning through `From<brk_error::Error>`.

## Built On

- `bitview_plugin::PluginSet` for generic plugin discovery
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
