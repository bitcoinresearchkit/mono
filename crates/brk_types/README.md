# brk_types

Bitcoin primitives shared across BRK and Bitview.

## What it provides

Purpose-built types for heights, amounts, hashes, addresses, transactions, block
templates and mempool state: values that are intrinsically tied to Bitcoin,
including responses shared with `brk_mempool` such as `MempoolInfo`. Bitview's
index, value and state types (calendar indexes, protocol epochs, per-type address
indexes, `Stored*` scalars) live in `bitview_primitives`; query-protocol types and
REST response DTOs live in `bitview_types`; `TreeNode` lives in `bitview_catalog`; CPFP
linearization lives in `brk_cpfp`.

## Type categories

| Category | Examples |
|----------|----------|
| Block metadata | `Height`, `BlockHash`, `BlkPosition`, `ReadBlock`, `Timestamp` |
| Transactions | `Txid`, `TxIndex`, `TxIn`, `TxOut`, `VSize`, `Weight`, `RawLockTime` |
| Addresses | `Addr`, `AddrBytes`, `OutputType`, `P2PKHBytes` |
| Values | `Sats`, `Bitcoin`, `Dollars`, `Cents`, `FeeRate` |
| Mempool | `MempoolInfo`, `MempoolBlock`, `BlockTemplate`, `RecommendedFees`, `CpfpInfo` |

The types implement the serialization, arithmetic, formatting, and optional JSON
Schema (`schemars`) and vecdb (`storage`) traits needed by their domains rather
than exposing a parallel set of API wrapper types.

## Features

The default build has no `vecdb`, `rawdb` or `schemars` dependency, so BRK
libraries such as `brk_mempool` and `brk_rpc` stay lean.

- `schemars` adds the JSON Schema derives (and `Version`'s schema with
  `storage`) that Bitview's API needs; `bitview_primitives` enables it.
- `storage`: see below.

## Storage support

Enable `storage` in crates that persist domain values; this adds
byte/compression derives, vector traits, storage versions, and storage error
conversions. Catalog and API-only consumers leave it disabled.

Index names and aliases (`Height`, `TxIndex`), arithmetic (including
`CheckedSub`), Serde, and JSON Schema (with `schemars`) remain available without
storage. Optional vecdb implementations delegate to those domain definitions.
`bitview_types::SeriesData::version` and Rust client version endpoints expose
the wire value as `u32`, not the storage engine's `Version` type.

Verify the modes separately to avoid workspace feature unification hiding an
accidental storage or schemars dependency:

```sh
cargo test -p brk_types --no-default-features
cargo test -p brk_types --features storage
cargo test -p brk_types --features schemars,storage
cargo tree -p bitview_catalog
```

## Example

```rust,ignore
use brk_types::{Height, Sats};

let height = Height::new(840_000);
let reward = Sats::FIFTY_BTC / 16;
let blocks_left = height.left_before_next_halving();
```

## Built on

- `bitcoin` for consensus primitives and address parsing
- `brk_error` for shared errors
- `vecdb` for optional persistent-vector traits
