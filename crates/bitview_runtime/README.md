# bitview_runtime

The generic synchronous lifecycle for statically composed Bitview plugins.

`PluginSet` can be derived from composition fields. Use
`#[plugin_set(flatten)]` for a nested composition and `#[plugin_set(skip)]` for
non-plugin runtime state. The update lifecycle closes every plugin gate,
computes the complete composition, commits its pipeline-wide publication
cursor, and only then reopens reads. Bootstrap uses each plugin's storage
identity to create active roots, reject duplicate IDs, and remove roots that no
active plugin owns.

The runner constructs one `ImportContext` for bootstrap and one
`UpdateContext` for computation. A composition forwards the import context to
each plugin constructor and passes the update context to each `ComputePlugin`.
Plugin-to-plugin inputs remain ordinary typed dependency structs. This keeps
shared lifecycle resources extensible without turning the contexts into a
service locator.

Most users should use [`bitview`](https://crates.io/crates/bitview), which adds
the official composition, configuration, query service, mempool tracking, and
HTTP server.
