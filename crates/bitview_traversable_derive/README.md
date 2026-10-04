# bitview_traversable_derive

Derives `bitview_traversable::Traversable` and `vecdb::ReadOnlyClone` for
structs. Generated traversal builds the public `TreeNode`, walks every
exportable vec, and keeps a separate visible-only iterator.

Field attributes:

- `skip` excludes a field from traversal. By default its read-only projection
  clones the field, except skipped `Option` fields become `None`.
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
struct Metrics<M: StorageMode = Rw> {
    #[traversable(flatten)]
    public: PublicMetrics,
    #[traversable(hidden)]
    internal: InternalMetrics,
    cache: M::WriteOnly<Cache>,
}
```

For structs generic over `M: StorageMode`, the generated read-only projection
uses the same struct with `M = Ro`. Other generic fields propagate their own
`ReadOnlyClone` implementation.

Use `M::WriteOnly<T>` for computation-only state. It is exactly `T` in `Rw`
and `()` in `Ro`. The derive automatically omits these fields from traversal
and produces `()` when projecting to read-only mode. No field attribute or
`Clone`, `Default`, or `ReadOnlyClone` implementation on `T` is needed.
Writer initialization and field access remain unchanged. Shared state needed
by queries must remain an ordinary field, not `WriteOnly`.

Keep shared `Database` handles: stored vectors hold weak database references,
so the read-only composition must retain an owner even if only writer code
calls the handle's methods. Publication gates and query lookup tables likewise
remain ordinary shared fields.
