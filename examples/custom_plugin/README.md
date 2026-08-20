# Custom plugin example

This is a complete custom Bitview plugin and composition. It stores a
`near_full_block_streak` series, computes it after the built-in plugins, exposes
it through the normal series API, and keeps all existing Bitview endpoints.
The runner is imported without its default composition; this example depends
on `bitview_default` explicitly because it deliberately extends that
composition.

Run it exactly like the standard Bitview executable:

```bash
cargo run --release -p bitview-custom-plugin-example
```

Then query the custom series:

```text
http://localhost:3110/api/series/near_full_block_streak/height
```

## What to copy

- `src/near_full_blocks/` is the plugin.
- `src/composition.rs` adds the plugin to Bitview's default composition and chooses
  when it computes.
- `src/main.rs` hands that composition to `bitviewd::run`.

Change the plugin storage ID and schema version, stored vectors, dependency
struct, and compute method for your metric. Bump the root schema version when a
change must invalidate the plugin's stored vectors; use narrower component
versions for isolated changes. Keep the publication gate and pass the indexer's
safe height as the maximum recomputation point so normal updates and reorgs
follow the same path.

The author contract is deliberately small:

1. Define one `PluginStorage` and return it, plus the plugin gate, from
   `Plugin`.
2. Accept `ImportContext` in the plugin constructor and open the database
   through that storage descriptor.
3. Define a typed dependency struct and implement `ComputePlugin`, using
   `UpdateContext` only for shared update control such as `Exit`.
4. Put the plugin in a derived `PluginSet`, call it in the composition's typed
   compute schedule, and expose a read-only accessor for consumers that need
   it.

`ImportContext` and `UpdateContext` are copyable borrowed handles. They do not
contain plugin dependencies, so adding a metric cannot silently change its
inputs.

`QueryPluginSet::query_capabilities` delegates built-in endpoint requirements to
`DefaultPlugins`; the full composition is still traversed, so custom series are
automatically included in the catalog and `/api/series`.
