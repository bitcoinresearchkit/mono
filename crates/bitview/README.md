# Bitview runner

`bitview` is the process-agnostic runtime for a statically typed Bitview plugin
composition. It owns bootstrap, updates, mempool tracking, queries, and HTTP
serving, but it does not parse process arguments, read configuration files,
install signal handlers, initialize logging, or select an official composition.

Applications pass a resolved [`Config`](https://docs.rs/bitview/latest/bitview/struct.Config.html),
an exit state, and their composition's import function to `bitview::run`.

The official daemon is provided by
[`bitviewd`](https://crates.io/crates/bitviewd). A complete custom composition is
available in the repository's
[`examples/custom_plugin`](https://github.com/bitcoinresearchkit/brk/tree/main/examples/custom_plugin)
package.

## License

MIT
