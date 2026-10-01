# brk_iterator

Unified block iteration with automatic source selection.

## What It Enables

Iterate over Bitcoin blocks with a simple API that automatically chooses between RPC (for small ranges) and direct blk file reading (for large scans). Handles reorgs gracefully.

## Key Features

- **Smart source selection**: RPC for ≤10 blocks, Reader for larger ranges
- **Inclusive height ranges**: Empty reversed ranges require no source access
- **Continuity checks**: Iteration returns an error if a reorg breaks the chain
- **Thread-safe**: Clone and share freely

## Core API

```rust,ignore
let blocks = Blocks::new(&rpc_client, &reader);

for block in blocks.range(Height::new(800_000), Height::new(800_100))? { ... }
```

Ranges are inclusive. Up to ten blocks use RPC; larger ranges use the block
reader. Reversed ranges return an empty iterator without contacting either
source. Counts support the full `u32` height domain without overflow.

## Built On

- `brk_error` for error handling
- `brk_reader` for direct blk file access
- `brk_rpc` for RPC queries
- `brk_types` for `Height`, `BlockHash`
