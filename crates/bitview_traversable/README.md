# bitview_traversable

Traversal contract for Bitview's hierarchical data and public series catalog.

`Traversable` provides three related views of a value:

- `to_tree_node()` builds its query-visible `TreeNode` hierarchy.
- `iter_any_exportable()` visits every exportable vec, including hidden data
  needed for persistence and maintenance.
- `iter_any_visible()` visits only vecs published through the series API.

The optional `derive` feature re-exports `#[derive(Traversable)]`. The derive
also implements `vecdb::ReadOnlyClone`, so storage-mode structs can expose a
read-only query projection from the same field definition.

The crate implements traversal for vecdb's stored, mutable, columnar,
overflow, cached, and lazy vector families, plus common containers such as
`Box`, `Option`, and `BTreeMap`. Backend-specific implementations
are enabled with the matching `pco`, `zerocopy`, `lz4`, or `zstd` feature.

See [`bitview_traversable_derive`](../bitview_traversable_derive) for supported
derive attributes.
