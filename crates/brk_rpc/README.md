# brk_rpc

Thread-safe Bitcoin Core RPC client with automatic retries.

## What It Enables

Query a Bitcoin Core node for blocks, transactions, mempool data, and chain state. Handles connection failures gracefully by retrying.

## Key Features

- **Auto-retry**: Up to 1M retries, one second apart, on transient failures
- **Thread-safe**: Clone freely, share across threads
- **Focused RPC coverage**: Blocks, headers, transactions, mempool, and UTXO queries
- **Mempool transactions**: Resolves prevouts for mempool tx fee calculation
- **Reorg detection**: `get_closest_valid_height` finds main chain after reorg
- **Sync waiting**: `wait_for_synced_node` blocks until node catches up

## Core API

```rust,ignore
// Explicit endpoint and credentials
let client = Client::new("http://localhost:8332", Auth::CookieFile(cookie_path))?;
// Or resolve them like bitcoin-cli does (user/password when a password is set, else the cookie)
let client = ConnectArgs::default().client()?;

let height = client.get_last_height()?;
let tip = client.get_best_block_hash()?;
let header = client.get_block_header_info(&tip)?;

// Mempool
let state = client.fetch_mempool_state()?;
```

## Key Methods

- `get_last_height`, `get_best_block_hash`, `get_block_info`, `get_block_header_info`, `get_block_hashes_range`
- `get_raw_transactions`, `fetch_mempool_state`, `fetch_new_pool_data`
- `get_closest_valid_height`, `get_network`, `wait_for_synced_node`

## Built On

- `brk_error` for error handling
- `brk_logger` for debug logging
- `brk_types` for `Height`, `BlockHash`, `Txid`, `MempoolEntryInfo`
