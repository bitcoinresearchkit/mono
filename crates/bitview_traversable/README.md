# bitview_traversable

Trait for navigating and exporting hierarchical data structures.

## What It Enables

Traverse nested data collections (datasets, grouped metrics) as trees for inspection, and iterate all exportable vectors for bulk data export.

## Key Features

- **Tree navigation**: Convert nested structs into `TreeNode` hierarchies for exploration
- **Export iteration**: Walk all `AnyExportableVec` instances in a data structure
- **Derive macro**: `#[derive(Traversable)]` with `derive` feature
- **Compression backends**: Support for PCO, LZ4, ZeroCopy, Zstd via feature flags
- **Blanket implementations**: Works with `Box<T>`, `Option<T>`, `BTreeMap<K, V>`

## Core API

```rust,ignore
pub trait Traversable {
    fn to_tree_node(&self) -> TreeNode;
    fn iter_any_exportable(&self) -> impl Iterator<Item = &dyn AnyExportableVec>;
}
```

## Supported Vec Types

All vecdb vector types implement `Traversable`:
- `BytesVec`, `EagerVec`, `PcoVec` (with `pco` feature)
- `ZeroCopyVec` (with `zerocopy` feature)
- `LZ4Vec`, `ZstdVec` (with respective features)
- `LazyVec` for single-source derived vectors

## Feature Flags

- `derive` - Enable `#[derive(Traversable)]` macro
- `pco` - PCO compression support
- `zerocopy` - Zero-copy vector support
- `lz4` - LZ4 compression support
- `zstd` - Zstd compression support

## Built On

- `brk_types` for `TreeNode` type
- `bitview_traversable_derive` for the derive macro (optional)
