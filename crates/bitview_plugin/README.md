# Bitview Plugin

The small compatibility contract shared by Bitview's built-in and external
plugins.

It provides stable plugin identity and publication gates for query-visible
mutable data. Computing plugins additionally declare their typed dependencies
and output through `ComputePlugin`. The runnable default composition lives in
[`bitview`](https://crates.io/crates/bitview).

The plugin API remains experimental while the built-in Bitview modules are
extracted into independent crates.

## License

MIT
