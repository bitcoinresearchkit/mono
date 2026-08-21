# vecdb_derive

Derive macros for using transparent newtypes with [`vecdb`](../vecdb/).

Applications normally enable these macros through VecDB rather than depending
on this crate directly:

```bash
cargo add vecdb --features derive
```

## `Bytes`

`#[derive(Bytes)]` delegates VecDB's portable fixed-width serialization to the
inner field:

```rust
use vecdb::Bytes;

#[derive(Debug, Clone, Copy, PartialEq, Bytes)]
struct UserId(u64);

let id = UserId(42);
assert_eq!(UserId::from_bytes(id.to_bytes().as_ref()).unwrap(), id);
```

The derived type can be stored in `BytesVec` and, with their respective
features, `LZ4Vec` or `ZstdVec`.

## `Pco`

Enable both `derive` and `pco` for numeric newtypes:

```bash
cargo add vecdb --features derive,pco
```

```rust
use vecdb::Pco;

#[derive(Debug, Clone, Copy, PartialEq, Pco)]
struct Price(f64);
```

`#[derive(Pco)]` generates `Bytes`, `Pco`, and the transparent-layout marker
used by `PcoVec`. The inner field must implement both `Bytes` and `Pco`; VecDB
implements those traits for `u8` through `u64`, `i8` through `i64`, `f32`, and
`f64`.

## Shape requirements

Both derives accept only tuple structs with exactly one field. Generic wrappers
are supported, and the generated implementation adds the required trait bounds
to the inner field. The wrapper must remain layout-compatible with its inner
type when used by pco; VecDB enforces the size and alignment assumptions at
compile time.
