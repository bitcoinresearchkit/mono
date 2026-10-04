# bitview_runtime

The generic synchronous lifecycle for statically composed Bitview plugins.

`PluginSet` can be derived from composition fields. Use
`#[plugin_set(flatten)]` for a nested composition, `#[plugin_set(skip)]` for
non-plugin runtime state, and `#[plugin_set(has = path::HasX<M>)]` to implement
a plugin's capability trait, whose accessor is named after the field.
`ComputePluginSet::publication` exposes the single barrier shared with readers. The update lifecycle closes that barrier,
computes the complete composition, commits its pipeline-wide publication
cursor, and only then reopens reads; a bootstrap pass that computed only part of
the composition commits with `complete = false`. Each import runs under the
shutdown lock. Bootstrap uses each plugin's storage
identity to create active roots, reject duplicate IDs, and remove roots that no
active plugin owns.

Plugin-root cleanup is intentionally destructive. The active composition owns
the complete `<data>/plugins` directory, and bootstrap removes every entry not
claimed by an active `PluginId`. Compositions whose stored data must coexist
should use separate data roots.

The runner constructs one `ImportContext` for bootstrap and one
`UpdateContext` for computation. A composition forwards the import context to
each plugin constructor and passes the update context to each `ComputePlugin`.
Plugin-to-plugin inputs remain ordinary typed dependency structs. This keeps
shared lifecycle resources extensible without turning the contexts into a
service locator.

Custom applications normally use [`bitview`](https://crates.io/crates/bitview),
the daemon runtime (configuration, logging, signals, query, mempool, and HTTP
services) around a supplied composition. The official composition is
[`bitview_default`](https://crates.io/crates/bitview_default), run by the
[`bitviewd`](https://crates.io/crates/bitviewd) binary.
