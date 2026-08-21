# brk_rpc

Thread-safe Bitcoin Core RPC client with automatic retries.

## What It Enables

Query a Bitcoin Core node for blocks, transactions, mempool data, and chain state. Handles connection failures gracefully with configurable retry logic.

## Key Features

- **Auto-retry**: Up to 1M retries with configurable delay on transient failures
- **Thread-safe**: Clone freely, share across threads
- **Focused RPC coverage**: Blocks, headers, transactions, mempool, and UTXO queries
- **Mempool transactions**: Resolves prevouts for mempool tx fee calculation
- **Reorg detection**: `get_closest_valid_height` finds main chain after reorg
- **Sync waiting**: `wait_for_synced_node` blocks until node catches up

## Core API

```rust,ignore
let client = Client::new("http://localhost:8332", Auth::CookieFile(cookie_path))?;

let height = client.get_last_height()?;
let hash = client.get_block_hash(height)?;
let block = client.get_block(&hash)?;

// Mempool
let txids = client.get_raw_mempool()?;
let entries = client.get_raw_mempool_verbose()?;
```

## Key Methods

- `get_block`, `get_block_hash`, `get_block_header_info`
- `get_transaction`, `get_mempool_transaction`, `get_tx_out`
- `get_raw_mempool`, `get_raw_mempool_verbose`
- `get_blockchain_info`, `get_last_height`
- `is_in_main_chain`, `get_closest_valid_height`

## Built On

- `brk_error` for error handling
- `brk_logger` for debug logging
- `brk_types` for `Height`, `BlockHash`, `Txid`, `MempoolEntryInfo`
