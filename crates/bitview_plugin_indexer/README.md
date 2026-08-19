# Bitview Indexer Plugin

Parses and indexes the entire Bitcoin blockchain so you can look up any block, transaction, input, output, or address by index in O(1).

## How It's Organized

Every entity gets a sequential index in blockchain order:

- Block 0, 1, 2, ... → **height**
- Transaction 0, 1, 2, ... → **txindex**
- Input 0, 1, 2, ... → **txinindex**
- Output 0, 1, 2, ... → **txoutindex**
- Address 0, 1, 2, ... → **addressindex** (per address type)

Data is stored in append-only vectors keyed by these indexes. Each block also stores the first index of each entity type it contains (e.g. `first_txindex`, `first_txoutindex`), so you can find all transactions, inputs, outputs, and addresses in any block in O(1).

## What's Indexed

### Per Block (keyed by height)

- Block hash, timestamp, difficulty, size, weight

### Per Transaction (keyed by txindex)

- Txid, version, locktime, base size, total size, RBF flag, block height

### Per Input (keyed by txinindex)

- Spent outpoint, containing txindex, and the spent output's type and address index

### Per Output (keyed by txoutindex)

- Value in satoshis, script type, address index within that type, containing txindex

Script types: P2PK (compressed/uncompressed), P2PKH, P2SH, P2WPKH, P2WSH, P2TR, P2A, P2MS, OP_RETURN, Empty, Unknown

### Per Address (keyed by addressindex, one set per type)

- Raw address bytes (20-65 bytes depending on type: pubkey, pubkey hash, script hash, witness program, etc.)

Address types each get their own index space: P2PK65, P2PK33, P2PKH, P2SH, P2WPKH, P2WSH, P2TR, P2A

### Per Non-Address Script (OP_RETURN, P2MS, Empty, Unknown)

- Containing txindex

## Key-Value Stores

On top of the vectors, key-value stores enable lookups that aren't sequential:

| Store | Purpose |
|-------|---------|
| txid prefix → txindex | Look up a transaction by its txid |
| block hash prefix → height | Look up a block by its hash |
| address hash → addressindex | Look up an address (per type) |
| addressindex + txindex | All transactions involving an address |
| addressindex + outpoint | Unspent outputs for an address (live UTXO set) |
| height → coinbase tag | Miner-embedded message per block |

## How It Works

1. **Block metadata** — store block hash, difficulty, timestamp, size, weight
2. **Compute TXIDs** — parallel SHA256d across all transactions
3. **Process outputs** — classify script types, extract addresses, detect new unique addresses
4. **Process inputs** — resolve spent outpoints, look up address info
5. **Finalize** — update address stores, UTXO set mutations, push all vectors
6. **Snapshot** — periodic flush to disk for crash recovery

Reorg handling is built-in: on chain reorganization, the indexer rolls back to the last valid state.

## Performance

| Version | Machine | Time | Disk | Peak Disk | Memory | Peak Memory |
|---------|---------|------|------|-----------|--------|-------------|
| v0.2.0-pre | MBP M3 Pro (36GB, internal SSD) | 2h40 | 239 GB | 302 GB | 5.9 GB | 13 GB |
| v0.1.0-alpha.0 | Mac Mini M4 (16GB, external SSD) | 4.9h | 233 GB | 303 GB | 5.4 GB | 11 GB |

Full benchmark data: [bitcoinresearchkit/benches](https://github.com/bitcoinresearchkit/benches/tree/main/brk_indexer)

## Recommended: mimalloc v3

Use [mimalloc v3](https://crates.io/crates/mimalloc) as the global allocator to reduce memory usage.

## Built On

- `vecdb` for append-only vectors — integer-compressed (`PcoVec`) or raw bytes (`BytesVec`)
- `brk_reader` for block parsing
- `brk_store` for key-value storage (fjall LSM)
- `brk_types` for domain types
