# Bitview Catalog

Shared series catalog model: tree nodes, branches, leaves, JSON schemas, and
source metadata used to construct catalogs and generate clients.

No dependency on traversal, the query runtime, `vecdb`, or `rawdb` in a standalone
build. `bitview_primitives` storage support is opt-in; a combined application build can
enable it through Cargo feature unification.
`bitview_traversable` builds this model; query, server, and generation code consume it. Serialized
catalogs retain their existing API shape; internal construction metadata stays
out of JSON.
