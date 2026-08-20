# Architecture

## Overview

```
blk*.dat ──▶ Reader ──┐
                      ├──▶ Plugin Set ──┐
         RPC Client ──┤                 ├──▶ Query ──▶ Server
                      └──▶ Mempool ─────┘

MCP clients ──▶ MCP Adapter ──▶ Server
```

## Components

### Reader (`brk_reader`)

Parses Bitcoin Core's `blk*.dat` files directly, bypassing RPC for historical data. Supports parallel parsing and handles XOR-encoded blocks (Bitcoin Core 28+).

### RPC Client (`brk_rpc`)

Connects to Bitcoin Core for real-time data: new blocks, mempool transactions, and fee estimates. Thread-safe with automatic retries.

### Indexer plugin (`bitview_plugin_indexer`)

Builds lookup tables from parsed blocks:
- Transaction index (txid → block position)
- Address index (address → transactions, UTXOs)
- UTXO set tracking
- Output type classification (P2PKH, P2WPKH, P2TR, etc.)

### Mappings plugin (`bitview_plugin_mappings`)

Derives the relationships used to navigate indexed data:
- Block height to time resolutions
- Transaction, input, and output index boundaries
- Address and script identity mappings
- Monotonic and per-resolution timestamps

### Plugin runtime (`bitview_runtime`)

Defines the generic plugin-set, bootstrap, update, and publication lifecycle.
It has no dependency on the official plugins.

### Default composition (`bitview_default`)

Owns the official indexer, mappings, and analytics plugins as one typed plugin
set. The indexer is the root of the dependency graph; downstream plugins derive:
- Market metrics: realized cap, MVRV, SOPR, NVT
- Supply metrics: circulating, liquid, illiquid
- UTXO cohorts: by age, size, type
- Address cohorts: by balance, activity
- Pricing models: thermocap, realized price bands

Metrics are computed across multiple time resolutions (daily, weekly, monthly, by block height).

### Runner (`bitview`)

Owns bootstrap, the update loop, mempool monitoring, queries, and server
startup. It accepts resolved runtime settings, an exit state, and a plugin-set
import function. It has no dependency on the official composition and does not
parse arguments, read configuration files, initialize logging, or install
process signal handlers.

### Daemon (`bitviewd`)

Owns the process boundary: command-line arguments, `config.toml`, logging,
signal handling, and the official executable. Its default feature selects
`bitview_default`; custom executables can disable that feature and pass
their own composition to the same daemon shell.

### Mempool (`brk_mempool`)

Monitors unconfirmed transactions:
- Fee rate distribution and estimation
- Projected block templates
- Address mempool activity

### Query (`bitview_query`)

Composition-independent interface to any compatible plugin set. Plugin-named
features add typed capability requirements, while generic series discovery
automatically includes every active traversable plugin:
- Block and transaction lookups
- Address balances and history
- Computed metrics with range queries
- Mempool state

### Server (`bitview_server`)

Composition-independent REST API exposing the enabled Query functionality.
Its plugin features forward directly to `bitview_query` and unavailable route
families are not compiled or registered:
- OpenAPI documentation (Scalar UI)
- JSON and CSV output formats
- ETag caching
- mempool.space compatible endpoints

### MCP Adapter (`bitview_mcp`)

Provides stateless, read-only MCP tools generated from the server's OpenAPI
operations. It forwards tool calls to the configured REST origin, allowing a
Cloudflare-fronted API to keep serving cached responses. The official endpoint
is [mcp.bitview.space](https://mcp.bitview.space/) and requires no
authentication.

## Data Flow

**Initial sync:**
1. Reader parses all `blk*.dat` files in parallel
2. The plugin set's indexer processes blocks sequentially, building indexes
3. Dependent plugins derive metrics from indexed data
4. Server starts accepting requests

**Ongoing operation:**
1. RPC client polls for new blocks
2. Reader fetches block data
3. The indexer plugin updates indexes
4. Dependent plugins recalculate affected metrics
5. Mempool monitors transaction pool

## Storage

Data is stored in `~/.bitview/` (configurable):

```
~/.bitview/
├── config.toml  # Daemon configuration
├── logs/        # Runtime logs
└── plugins/     # Plugin directories
    ├── indexer/
    ├── mappings/
    ├── blocks/
    ├── price/
    └── .../     # One directory per active plugin ID
```

Disk usage scales with blockchain size. Full index with metrics: ~400 GB.

## Dependencies

Built on:
- [`rust-bitcoin`](https://github.com/rust-bitcoin/rust-bitcoin) - Bitcoin primitives
- [`fjall`](https://github.com/fjall-rs/fjall) - LSM-tree storage
- [`vecdb`](https://github.com/anydb-rs/anydb) - Vector storage
- [`axum`](https://github.com/tokio-rs/axum) - HTTP server
- [`aide`](https://github.com/tamasfe/aide) - OpenAPI generation
