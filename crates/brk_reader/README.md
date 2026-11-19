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
let reader = Reader::new(node.blocks_dir()?, &client)?;

// Everything from genesis to the current tip
let tip = client.get_last_height()?;
for block in reader.after(None, tip)? {
    let block = block?;
    println!("{}: {}", block.height(), block.hash());
}

// Everything strictly after a known hash (typical sync / catchup pattern)
for block in reader.after(Some(last_known_hash), tip)? {
    let block = block?;
    // ...
}
```

`Reader` is thread-safe and cheap to clone (Arc-backed). Each item is
a `Result<ReadBlock>` so mid-stream failures (chain breaks, parse
errors, missing canonical blocks) reach the consumer as a final
`Err` instead of being silently dropped.

## How It Works

One pipeline for every range: locate each block by its header, read the bodies in chain order.

```text
canonical chain ──► Locate ──► Parsers ──► BlockReceiver
(hashes via RPC)    hops frames,   read and      in canonical order
                    headers only   decode bodies
```

1. **`CanonicalRange`** asks bitcoind up front for the canonical
   block hash at every height in the target window, in sequential
   JSON-RPC batches of 2,000 heights (one round trip each), no per-block
   RPC chatter. A batch is retried as single calls are (lost connections, the node warming up).
2. **Locate** hops the blk files frame to frame (a frame's 8-byte head gives the next one's
   offset), reads each block's 80-byte header, and looks its hash up in the canonical map. Core
   writes blocks out of height order during initial sync and reindex, so a block found before
   its turn waits as a position (file, offset, header), never as bytes; stale blocks match
   nothing. Each block's `prev_blockhash` is checked against the canonical chain, and against the
   anchor for the first.
   Where the hop starts:
   * right after the last block located, for an update that follows the previous call: only the
     frames appended since are read, with no directory scan;
   * otherwise at a file whose first block is no newer than the anchor (a binary search over
     the files' first blocks), 21 files back (the margin for out-of-order writes), or at the
     first file from genesis.
3. **Parsers** (a third of the cores, at most four) read each located body with one positioned
   read (on macOS past the OS page cache) and decode it. Each block is answered on its own
   channel, and the receiver takes the answers in chain order.

See `src/pipeline.rs` for the pipeline, `src/scan.rs` for the frame hop and `src/canonical.rs`
for the filter map.
