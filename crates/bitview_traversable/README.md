# bitview_traversable

Trait for navigating and exporting hierarchical data structures.

## What It Enables

Traverse nested data collections as public series trees, and iterate their
exportable vectors for persistence and bulk export.

## Key Features

- **Tree navigation**: Convert nested structs into `TreeNode` hierarchies for exploration
- **Export iteration**: Walk all `AnyExportableVec` instances, including a public-only view
- **Derive macro**: `#[derive(Traversable)]` with `derive` feature
- **Read-only projection**: the derive also generates `vecdb::ReadOnlyClone`
  for storage-mode and generic container structs
- **Compression backends**: Support for PCO, LZ4, ZeroCopy, Zstd via feature flags
- **Blanket implementations**: Works with `Box<T>`, `Option<T>`, `BTreeMap<K, V>`

## Core API

```rust,ignore
pub trait Traversable {
    fn to_tree_node(&self) -> TreeNode;
    fn iter_any_exportable(&self) -> impl Iterator<Item = &dyn AnyExportableVec>;
}
```

For a struct generic over `M: StorageMode`, `#[derive(Traversable)]` maps its
read-write form to the same struct with `M = Ro`. For generic container fields,
it propagates `ReadOnlyClone` through those fields. Skipped fields are cloned
unchanged. This gives plugin compositions a read-only query projection without
maintaining a second field list.

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

- `bitview_types` for series-tree schemas
- `brk_types` for index types
- `bitview_traversable_derive` for the derive macro (optional)
