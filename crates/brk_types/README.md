# brk_types

Bitcoin domain and storage-index types shared across BRK and Bitview.

## What it provides

Purpose-built types for heights, amounts, hashes, addresses, transactions,
calendar indexes, protocol epochs, and API values that are intrinsically tied
to Bitcoin. Query-protocol types such as `SeriesSelection`, `SeriesData`,
`Pagination`, and `TreeNode` live in `bitview_types`.

## Type categories

| Category | Examples |
|----------|----------|
| Block metadata | `Height`, `BlockHash`, `BlockTimestamp`, `BlkPosition` |
| Transactions | `Txid`, `TxIndex`, `TxIn`, `TxOut`, `VSize`, `Weight` |
| Addresses | `Addr`, `OutputType`, `P2PKHAddrIndex`, `AnyAddrIndex`, `AddrStats` |
| Values | `Sats`, `Bitcoin`, `Dollars`, `Cents`, `OHLCCents` |
| Time indexes | `Day1`, `Day3`, `Week1`, `Month1`, `Month3`, `Month6`, `Year1`, `Year10` |
| Protocol | `Epoch`, `Halving`, `TxVersion`, `RawLockTime` |

The types implement the serialization, JSON Schema, arithmetic, formatting,
and vecdb traits needed by their domains rather than exposing a parallel set of
API wrapper types.

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
- `vecdb` for persistent-vector traits
