# bitview_plugin_derive

Derives `bitview_plugin::PluginSet` for statically composed Bitview plugins
(re-exported by `bitview_plugin` under its `derive` feature).

Field attributes:

- `#[plugin_set(flatten)]` visits a nested composition's plugins.
- `#[plugin_set(skip)]` excludes non-plugin state.
- `#[plugin_set(has = path::HasX<M>)]` implements a plugin's capability trait,
  whose accessor is named after the field.

## License

MIT
