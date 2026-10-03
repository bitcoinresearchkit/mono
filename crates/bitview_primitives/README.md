# bitview_primitives

The types Bitview's series are made of, built on the Bitcoin primitives in `brk_types`.

## What it provides

- **Indexes**: calendar and protocol resolutions (`Day1`, `Week1`, `Year10`, `Epoch`, `Halving`, ...), `Date`, per-type
  address and output indexes (`P2PKHAddrIndex`, `TypeIndex`, `TxInIndex`, ...) and the `Index` enum naming them.
- **Values**: storage scalars (`StoredF32`, `StoredU64`, ...), ratios and fixed-point units (`BasisPoints32`,
  `PartsPerMillion64`, `CentsCompact`, `SatsFract`, ...), OHLC prices and percentile ids.
- **State records**: address and supply state (`FundedAddrData`, `EmptyAddrData`, `SupplyState`), cost-basis snapshots.
- **Domain enums**: mining pools, OP_RETURN kinds, capital-sentiment phases.

The `storage` feature adds the `vecdb` impls these types need to live in Bitview's vecs.

Calendar parsing invariants are described in [docs/calendar.md](docs/calendar.md).

## Built on

- `brk_types` for the Bitcoin primitives these types index and aggregate
- `brk_error` for shared errors
- `vecdb` for optional persistent-vector traits
