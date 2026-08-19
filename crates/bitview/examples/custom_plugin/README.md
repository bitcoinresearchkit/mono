# Custom plugin example

This is a complete custom Bitview plugin and composition. It stores a
`near_full_block_streak` series, computes it after the built-in plugins, exposes
it through the normal series API, and keeps all existing Bitview endpoints.

Run it exactly like the standard Bitview executable:

```bash
cargo run --release -p bitview --example custom_plugin
```

Then query the custom series:

```text
http://localhost:3110/api/series/near_full_block_streak/height
```

## What to copy

- `near_full_blocks/` is the plugin. In a real project, move this directory into
  its own crate.
- `composition.rs` adds the plugin to Bitview's default composition and chooses
  when it computes.
- `main.rs` hands that composition to `bitview::run_with`.

Change the plugin ID, stored vectors, dependency struct, and compute method for
your metric. Keep the publication gate and use `indexer.safe_lengths()` as the
maximum recomputation point so normal updates and reorgs follow the same path.

`QueryPluginSet::query_capabilities` delegates built-in endpoint requirements to
`DefaultPlugins`; the full composition is still traversed, so custom series are
automatically included in the catalog and `/api/series`.
