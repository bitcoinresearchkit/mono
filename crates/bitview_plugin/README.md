# Bitview Plugin

The small compatibility contract shared by Bitview's built-in and external
plugins.

It provides stable plugin identity and root storage schema. A plugin declares one `PluginStorage`, which is
the source of truth for its `PluginId`, root schema version, `plugins/<id>`
directory, database opening, and database finalization. The directory may be
empty for an in-memory plugin. Component versions remain local and additive
when they describe a narrower stored or computed dependency.

Plugin import constructors receive a copyable `ImportContext`, which provides
the composition data root to `PluginStorage` and the shutdown coordinator
(`exit()`) for import-time writes that must not be interrupted. Application startup initializes
vecdb's shared cache budget once; import contexts do not carry it. Source writes
own cache invalidation and preserve unchanged prefixes.
Computing plugins declare their
read-only dependencies through `ComputePlugin` and update their own data in
`compute_state`. The provided `compute` wraps it in the storage lifecycle: it
syncs the database returned by `database()` before computing and compacts it
after success (`None` opts out for plugins that manage their own storage). Its copyable
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

## Plugin layout

Every built-in plugin crate (except the indexer) has the same root:

- `lib.rs`, in this order: `macro_rules!` used by its modules, module
  declarations, re-exports, imports, `STORAGE` (and `ID` when the composition
  names the plugin), the `Vecs` struct, its `impl Plugin`, then any other impls;
- `dependencies.rs`: the typed `Dependencies` its computation borrows;
- `import.rs`: `Vecs::import(ImportContext, ..)`;
- `compute.rs`: `impl ComputePlugin for Vecs` (plugins with replay machinery keep
  it in `compute/mod.rs` next to that machinery);
- `has.rs` (optional): the `HasX` capability trait other plugins depend on;
- one module per metric family: `<family>/{mod.rs, vecs.rs or vecs/, import.rs,
  compute.rs}` when the family is large enough to split, otherwise a single
  `<family>.rs`.

Constructors are associated functions: the plugin and its database-backed
families `import` (like vecdb's vectors, resetting data whose version changed),
lazy views `new` (or `from_*` when derived from one source), and other stores
`open`. Computations on a plugin's own types are methods
(`self.family.compute(..)`); free functions remain only for helpers over
imported vector types.

To schedule a built-in plugin, add its field to `DefaultPlugins`
(`bitview_default/src/lib.rs`, with `#[plugin_set(has = ..)]` when others
depend on it), construct it in `bitview_default/src/import.rs`, and call its
`compute` in `bitview_default/src/compute.rs` after every plugin its
`Dependencies` borrow. `examples/custom_plugin` shows the same steps for an
external plugin.

## Pipeline publication

Plugins do not own locks readers wait on. The indexer publishes the lengths every
plugin has computed to, with their tip and a revision, when the composition
commits (`ComputePluginSet::commit`); readers pin that publication (`Indexer::pin`,
which never waits) and read below it, so a plugin appends past it freely and a
failed update leaves the last publication published (lowered to the fork once a
reorg rolled back).

Data rewritten in place must stay coherent with the pin on its own: mask values a
later publication wrote (the outputs spend index), write it with the publication
under a lock its readers take before pinning (address state), or replace it by
rename (UTXO set snapshots). A plugin about to rewrite rows below the published
lengths first lowers the publication to where its rollback lands
(`Indexer::lower_publication_to`), which waits for the pins of later publications.

Bootstrap holds `exit.lock()` around every import (`ImportContext::exit`), so
imports never wait on the node; no readers exist yet. A rollback's height persists
as the indexer's rollback floor until a complete update commits
(`ComputePluginSet::commit(complete)`), so dependents recompute from it after a
soft quit.

## License

MIT
