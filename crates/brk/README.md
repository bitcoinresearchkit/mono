# brk

Umbrella crate for the Bitcoin Research Kit.

[crates.io](https://crates.io/crates/brk) | [docs.rs](https://docs.rs/brk)

## Usage

Single dependency to access any BRK component. Enable only what you need via feature flags.

```toml
[dependencies]
brk = { version = "0.1", features = ["query", "types"] }
```

```rust,ignore
use brk::query::Query;
use brk::types::Height;
```

Feature flags match crate names without the `brk_` prefix. Use `full` to enable all:

```toml
[dependencies]
brk = { version = "0.1", features = ["full"] }
```

## Crates

**Core Pipeline**

| Crate | Description |
|-------|-------------|
| [brk_reader](https://docs.rs/brk_reader) | Read blocks from `blk*.dat` with parallel parsing and XOR decoding |
| [brk_indexer](https://docs.rs/brk_indexer) | Index transactions, addresses, and UTXOs |
| [brk_computer](https://docs.rs/brk_computer) | Compute derived metrics (realized cap, MVRV, SOPR, cohorts, etc.) |
| [brk_mempool](https://docs.rs/brk_mempool) | Monitor mempool, estimate fees, project upcoming blocks |
| [brk_oracle](https://docs.rs/brk_oracle) | Pure on-chain BTC/USD price oracle |
| [bitview_query](https://docs.rs/bitview_query) | Query interface for indexed and computed data |
| [bitview_server](https://docs.rs/bitview_server) | REST API with OpenAPI docs |

**Data & Storage**

| Crate | Description |
|-------|-------------|
| [brk_types](https://docs.rs/brk_types) | Domain types: `Height`, `Sats`, `Txid`, addresses, etc. |
| [brk_store](https://docs.rs/brk_store) | Key-value storage (fjall wrapper) |
| [brk_fetcher](https://docs.rs/brk_fetcher) | Fetch price data from exchanges |
| [brk_rpc](https://docs.rs/brk_rpc) | Bitcoin Core RPC client |
| [brk_iterator](https://docs.rs/brk_iterator) | Unified block iteration with automatic source selection |
| [brk_cohort](https://docs.rs/brk_cohort) | UTXO and address cohort filtering |
| [bitview_traversable](https://docs.rs/bitview_traversable) | Navigate hierarchical data structures |

**Clients & Integration**

| Crate | Description |
|-------|-------------|
| [bitview_client](https://docs.rs/bitview_client) | Generated Rust API client |
| [bitview_bindgen](https://docs.rs/bitview_bindgen) | Generate typed clients (Rust, JavaScript, Python) |
| [bitview_mcp](https://crates.io/crates/bitview_mcp) | Stateless, read-only MCP adapter for the Bitview API |

The official MCP endpoint is
[mcp.bitview.space](https://mcp.bitview.space/). It requires no authentication.

The complete self-hosted platform and executable are provided by
[`bitview`](https://docs.rs/bitview) (`cargo install --locked bitview`).

**Internal**

| Crate | Description |
|-------|-------------|
| [brk_error](https://docs.rs/brk_error) | Error types |
| [brk_logger](https://docs.rs/brk_logger) | Logging infrastructure |
| [bitview_bencher](https://docs.rs/bitview_bencher) | Benchmarking utilities |
