# brk_types

Bitcoin domain and storage-index types shared across BRK and Bitview.

## What it provides

Purpose-built types for heights, amounts, hashes, addresses, transactions,
calendar indexes, protocol epochs, and API values that are intrinsically tied
to Bitcoin. Query-protocol types such as `SeriesSelection`, `SeriesData`,
and `Pagination` live in `bitview_types`; `TreeNode` lives in `bitview_catalog`.

## Type categories

| Category | Examples |
|----------|----------|
| Block metadata | `Height`, `BlockHash`, `BlockTimestamp`, `BlkPosition` |
| Transactions | `Txid`, `TxIndex`, `TxIn`, `TxOut`, `VSize`, `Weight` |
| Addresses | `Addr`, `OutputType`, `P2PKHAddrIndex`, `AddrStats` |
| Values | `Sats`, `Bitcoin`, `Dollars`, `Cents`, `OHLCCents` |
| Time indexes | `Day1`, `Day3`, `Week1`, `Month1`, `Month3`, `Month6`, `Year1`, `Year10` |
| Protocol | `Epoch`, `Halving`, `TxVersion`, `RawLockTime` |

The types implement the serialization, JSON Schema, arithmetic, formatting,
and optional vecdb traits needed by their domains rather than exposing a parallel
set of API wrapper types.

## Storage support

The default build has no `vecdb` or `rawdb` dependency. Enable `storage` in crates
that persist domain values; this adds byte/compression derives, vector traits,
storage versions, and storage error conversions. Catalog and API-only consumers
leave it disabled.

Index names and aliases, arithmetic (including `CheckedSub`), Serde, and JSON
Schema remain available without storage. Optional vecdb implementations delegate
to those domain definitions. `bitview_types::SeriesData::version` and Rust client
version endpoints expose the wire value as `u32`, not the storage engine's
`Version` type.

Verify both modes separately to avoid workspace feature unification hiding an
accidental storage dependency:

```sh
cargo test -p brk_types --no-default-features
cargo test -p brk_types --features storage
cargo tree -p bitview_catalog
```

## Example

```rust,ignore
use brk_types::{Date, Day1, Height, Sats};

let height = Height::new(840_000);
let reward = Sats::FIFTY_BTC / 16;
let day = Day1::try_from(Date::new(2024, 4, 20))?;
```

## Built on

- `bitcoin` for consensus primitives and address parsing
- `brk_error` for shared errors
- `vecdb` for optional persistent-vector traits
