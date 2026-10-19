# brk_types

Bitcoin primitives shared across BRK and Bitview.

## What it provides

Types for heights, amounts, hashes, addresses, transactions, block
templates and mempool state, including responses shared with `brk_mempool` such
as `MempoolInfo`. Bitview's
index, value and state types (calendar indexes, protocol epochs, per-type address
indexes, unit-typed scalars) live in `bitview_primitives`; query-protocol types and
REST response DTOs live in `bitview_types`; `TreeNode` lives in `bitview_catalog`; CPFP
linearization lives in `brk_cpfp`.

## Features

The default build has no `vecdb`, `rawdb` or `schemars` dependency, so BRK
libraries such as `brk_mempool` and `brk_rpc` stay lean.

- `schemars` adds the JSON Schema derives (and `Version`'s schema with
  `storage`) that Bitview's API needs; `bitview_primitives` enables it.
- `storage` adds byte/compression derives, vector traits, storage versions, and
  storage error conversions, for crates that persist domain values; catalog and
  API-only consumers leave it disabled.

Index names and aliases (`Height`, `TxIndex`), arithmetic (including
`CheckedSub`), Serde, and JSON Schema (with `schemars`) remain available without
storage. `bitview_types::SeriesData::version` and Rust client version endpoints expose
the wire value as `u32`, not the storage engine's `Version` type.

Verify the modes separately to avoid workspace feature unification hiding an
accidental storage or schemars dependency:

```sh
cargo check -p brk_types --no-default-features
cargo check -p brk_types --features storage
cargo check -p brk_types --features schemars,storage
cargo tree -p bitview_catalog
```

## Example

```rust,ignore
use brk_types::Height;

let height = Height::new(840_000);
let reward = height.subsidy();
let blocks_left = height.left_before_next_halving();
```
