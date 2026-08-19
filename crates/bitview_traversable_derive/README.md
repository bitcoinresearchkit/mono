# bitview_traversable_derive

Proc-macro for deriving the `Traversable` trait on data structures.

## What It Enables

Automatically generate tree traversal and export iteration for structs, eliminating boilerplate when working with hierarchical data that needs serialization or inspection.

## Key Features

- **Automatic tree building**: Converts struct fields into `TreeNode::Branch` hierarchies
- **Export iteration**: Generates `iter_any_exportable()` to walk all exportable vectors
- **Field attributes**: `#[traversable(skip)]` to exclude fields, `#[traversable(flatten)]` to merge nested structures
- **Option support**: Gracefully handles `Option<T>` fields
- **Generic-aware**: Properly bounds generic parameters with `Traversable + Send + Sync`

## Core API

```rust,ignore
#[derive(Traversable)]
struct MyData {
    pub metrics: MetricsCollection,
    #[traversable(flatten)]
    pub nested: NestedData,
    #[traversable(skip)]
    internal: Cache,
}
```

## Generated Methods

- `to_tree_node(&self) -> TreeNode` - Build navigable tree structure
- `iter_any_exportable(&self) -> impl Iterator<Item = &dyn AnyExportableVec>` - Iterate all exportable vectors
