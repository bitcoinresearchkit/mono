# Architecture

## Overview

```
blk*.dat ──▶ Reader ──┐
                      ├──▶ Indexer ──▶ Computer ──┐
         RPC Client ──┤                           ├──▶ Query ──▶ Server
                      └──▶ Mempool ───────────────┘

MCP clients ──▶ MCP Adapter ──▶ Server
```

## Components

### Reader (`brk_reader`)

Parses Bitcoin Core's `blk*.dat` files directly, bypassing RPC for historical data. Supports parallel parsing and handles XOR-encoded blocks (Bitcoin Core 28+).

### RPC Client (`brk_rpc`)

Connects to Bitcoin Core for real-time data: new blocks, mempool transactions, and fee estimates. Thread-safe with automatic retries.

### Indexer (`brk_indexer`)

Builds lookup tables from parsed blocks:
- Transaction index (txid → block position)
- Address index (address → transactions, UTXOs)
- UTXO set tracking
- Output type classification (P2PKH, P2WPKH, P2TR, etc.)

### Computer (`brk_computer`)

Derives analytics from indexed data:
- Market metrics: realized cap, MVRV, SOPR, NVT
- Supply metrics: circulating, liquid, illiquid
- UTXO cohorts: by age, size, type
- Address cohorts: by balance, activity
- Pricing models: thermocap, realized price bands

Metrics are computed across multiple time resolutions (daily, weekly, monthly, by block height).

### Mempool (`brk_mempool`)

Monitors unconfirmed transactions:
- Fee rate distribution and estimation
- Projected block templates
- Address mempool activity

### Query (`bitview_query`)

Unified interface to all data sources:
- Block and transaction lookups
- Address balances and history
- Computed metrics with range queries
- Mempool state

### Server (`bitview_server`)

REST API exposing Query functionality:
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
2. Indexer processes blocks sequentially, building indexes
3. Computer derives metrics from indexed data
4. Server starts accepting requests

**Ongoing operation:**
1. RPC client polls for new blocks
2. Reader fetches block data
3. Indexer updates indexes
4. Computer recalculates affected metrics
5. Mempool monitors transaction pool

## Storage

Data is stored in `~/.bitview/` (configurable):

```
~/.bitview/
├── indexer/     # Transaction and address indexes (fjall)
├── computer/    # Computed metrics (vecdb)
└── config.toml  # Configuration
```

Disk usage scales with blockchain size. Full index with metrics: ~400 GB.

## Dependencies

Built on:
- [`rust-bitcoin`](https://github.com/rust-bitcoin/rust-bitcoin) - Bitcoin primitives
- [`fjall`](https://github.com/fjall-rs/fjall) - LSM-tree storage
- [`vecdb`](https://github.com/anydb-rs/anydb) - Vector storage
- [`axum`](https://github.com/tokio-rs/axum) - HTTP server
- [`aide`](https://github.com/tamasfe/aide) - OpenAPI generation
