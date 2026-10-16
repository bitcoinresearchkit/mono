# brk_reader

Streams Bitcoin blocks from Bitcoin Core's raw `blk*.dat` files in
canonical chain order, skipping orphans.

## Requirements

A running mainnet Bitcoin Core node with RPC access (block files are matched by mainnet's magic
bytes, so testnet, signet and regtest are not supported). The reader needs:

- The `blocks/` directory (to read `blk*.dat` files), holding every block it is asked for: reading
  from genesis needs an unpruned node
- RPC connection (to resolve the canonical chain up front)

## Quick Start

```rust,ignore
// Bitcoin Core's default data directory, cookie file and RPC port
let node = ConnectArgs::default();
let client = node.client()?;
let reader = Reader::new(node.blocks_dir(), &client);

// Everything from genesis to the current tip
for block in reader.after(None)? {
    let block = block?;
    println!("{}: {}", block.height(), block.hash());
}

// Everything strictly after a known hash (typical sync / catchup pattern)
for block in reader.after(Some(last_known_hash))? {
    let block = block?;
    // ...
}

// A specific inclusive height range
for block in reader.range(Height::new(800_000), Height::new(850_000))? {
    let block = block?;
    // ...
}
```

`Reader` is thread-safe and cheap to clone (Arc-backed). Each item is
a `Result<ReadBlock>` so mid-stream failures (chain breaks, parse
errors, missing canonical blocks) reach the consumer as a final
`Err` instead of being silently dropped.

## What You Get

Each `ReadBlock` gives you access to:

| Field                  | Description                              |
| ---------------------- | ---------------------------------------- |
| `block.height()`       | Block height                             |
| `block.hash()`         | Block hash                               |
| `block.header`         | Block header (timestamp, nonce, ...)     |
| `block.txdata`         | All transactions                         |
| `block.coinbase_tag()` | Miner's coinbase tag                     |
| `block.metadata()`     | Position in the blk file                 |
| `block.tx_metadata()`  | Per-transaction blk file positions       |

## How It Works

Three strategies, picked per call:

* **appended** — an update right after the previous call reads only the
  frames appended after the last block it streamed, in the same blk file:
  no directory scan, probe, or walk from the file's start. Any
  discontinuity (another anchor, a rollover, a gap) falls back to tail.

* **forward** — one reader thread walks blk files in order from a
  bisection lower bound and ships canonical hits to a pool of parser
  threads (a third of the cores, at most four), which decode bodies
  and emit them in order. Blocks found past a gap in the sequence are
  parsed ahead up to a byte budget; beyond it only their positions are
  kept and re-read once the gap closes.
* **tail** — single-threaded reverse header scan of the newest blk
  files, then in-order body reads, used when the requested range sits
  within ~8 files of the chain tip. Avoids the forward pipeline's bisection + 21-file backoff
  (~2.7 GB of reads) for tip-clustered catchups.

```text
canonical chain ──► Reader thread ──► Parser pool ──► Receiver<Result<ReadBlock>>
(pre-fetched         walks blk files,    N workers       in canonical order
 hashes via RPC)     peeks headers,      decode bodies
                     ships hits
```

1. **`CanonicalRange`** asks bitcoind up front for the canonical
   block hash at every height in the target window, in sequential
   JSON-RPC batches of 2,000 heights (one round trip each), no per-block
   RPC chatter. A failed batch fails the read: batches are not retried.
2. **Reader thread** walks blk files, scans each for block magic, and
   for every block found hashes its 80-byte header and looks the hash
   up in the canonical map. Orphans short-circuit before the block
   bytes are cloned.
3. **Parser pool** (scoped threads, forward pipeline only) fully
   decodes canonical bodies in parallel and serialises output through
   an in-order reorder buffer. The reader thread has already checked
   each block's `prev_blockhash` against the canonical chain — and
   against the user-supplied anchor for the very first block.

Orphans can never be mistaken for canonical blocks, and a missing
canonical block produces a final `Err` to the consumer instead of a
silent drop. See `src/pipeline/` for the orchestration and
`src/canonical.rs` for the filter map.
