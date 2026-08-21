# bitview_traversable_derive

Derives `bitview_traversable::Traversable` and `vecdb::ReadOnlyClone` for
structs. Generated traversal builds the public `TreeNode`, walks every
exportable vec, and keeps a separate visible-only iterator.

Field attributes:

- `skip` excludes a field from traversal and clones it unchanged in a
  read-only projection.
- `flatten` merges a nested tree into its parent.
- `hidden` keeps a field exportable while omitting it from the public tree and
  visible iterator.
- `rename = "name"` changes its tree key.
- `wrap = "path"` places it below an additional path.

Struct attributes support `merge`, `transparent`, `hidden`, and `wrap =
"path"`. Single-field tuple structs delegate transparently. Named fields may
be optional, and doc comments are collected as series-description fragments.

```rust,ignore
#[derive(Traversable)]
struct Metrics {
    #[traversable(flatten)]
    public: PublicMetrics,
    #[traversable(hidden)]
    internal: InternalMetrics,
    #[traversable(skip)]
    cache: Cache,
}
```

For structs generic over `M: StorageMode`, the generated read-only projection
uses the same struct with `M = Ro`. Other generic fields propagate their own
`ReadOnlyClone` implementation.
