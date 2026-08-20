# Bitview Plugin

The small compatibility contract shared by Bitview's built-in and external
plugins.

It provides stable plugin identity, root storage schema, and publication gates
for query-visible mutable data. A plugin declares one `PluginStorage`, which is
the source of truth for its `PluginId`, root schema version, `plugins/<id>`
directory, database opening, and database finalization. The directory may be
empty for an in-memory plugin. Component versions remain local and additive
when they describe a narrower stored or computed dependency.

Plugin import constructors receive a copyable `ImportContext`, which provides
the composition data root to `PluginStorage`. Computing plugins declare their
typed dependencies and output through `ComputePlugin`; its copyable
`UpdateContext` provides shared update control such as cancellation. Plugin
dependencies stay explicit and typed instead of being hidden in either
context.

The contexts are lightweight borrowed handles: the runner creates one of each
and passes them by value through the composition. The
runnable default composition lives in
[`bitview_default`](https://crates.io/crates/bitview_default), while
[`bitview`](https://crates.io/crates/bitview) runs any compatible composition.
Generic composition and update lifecycle traits live in
[`bitview_runtime`](https://crates.io/crates/bitview_runtime).

The plugin API remains experimental while the built-in Bitview modules are
extracted into independent crates.

## License

MIT
