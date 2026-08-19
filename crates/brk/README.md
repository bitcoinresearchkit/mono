# brk

Umbrella crate for the Bitcoin Research Kit.

[crates.io](https://crates.io/crates/brk) | [docs.rs](https://docs.rs/brk)

## Usage

Single dependency for BRK's reusable Bitcoin primitives. Enable only what you
need through feature flags.

```toml
[dependencies]
brk = { version = "0.11", features = ["reader", "types"] }
```

```rust,ignore
use brk::reader::Reader;
use brk::types::Height;
```

Feature flags match crate names without the `brk_` prefix. Use `full` to enable all:

```toml
[dependencies]
brk = { version = "0.11", features = ["full"] }
```

## Crates

**Bitcoin data**

| Crate | Description |
|-------|-------------|
| [brk_reader](https://docs.rs/brk_reader) | Read blocks from `blk*.dat` with parallel parsing and XOR decoding |
| [brk_mempool](https://docs.rs/brk_mempool) | Monitor mempool, estimate fees, project upcoming blocks |
| [brk_oracle](https://docs.rs/brk_oracle) | Pure on-chain BTC/USD price oracle |
| [brk_rpc](https://docs.rs/brk_rpc) | Bitcoin Core RPC client |

**Data & Storage**

| Crate | Description |
|-------|-------------|
| [brk_types](https://docs.rs/brk_types) | Domain types: `Height`, `Sats`, `Txid`, addresses, etc. |
| [brk_store](https://docs.rs/brk_store) | Key-value storage (fjall wrapper) |
| [brk_fetcher](https://docs.rs/brk_fetcher) | Fetch price data from exchanges |
| [brk_iterator](https://docs.rs/brk_iterator) | Unified block iteration with automatic source selection |
| [brk_cohort](https://docs.rs/brk_cohort) | UTXO and address cohort filtering |
| [brk_error](https://docs.rs/brk_error) | Shared error types |
| [brk_logger](https://docs.rs/brk_logger) | Logging infrastructure |

Indexing, computation, querying, clients, and serving belong to Bitview. The
complete self-hosted platform and executable are provided by
[`bitview`](https://docs.rs/bitview) (`cargo install --locked bitview`).
