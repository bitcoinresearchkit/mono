# Bitview runner

`bitview` is the composition-agnostic daemon runtime for a statically typed
Bitview plugin set. It owns bootstrap, updates, mempool tracking, queries, and
HTTP serving, but it does not parse process arguments, read configuration
files, install signal handlers, initialize logging, or select an official
composition.

Applications pass a resolved [`Config`](https://docs.rs/bitview/latest/bitview/struct.Config.html),
an exit state, and their composition's import function to `bitview::run`.

The runner is designed for one long-lived instance per process. Its query view
and process-wide services intentionally live until process exit; it is not a
start-stop or multi-instance application server.

The `full-api` feature enables every optional API group (`chain`, `series`, and
`urpd`) without selecting a plugin composition.

The official daemon is provided by
[`bitviewd`](https://crates.io/crates/bitviewd). A complete custom composition is
available in the repository's
[`examples/custom_plugin`](https://github.com/bitcoinresearchkit/brk/tree/main/examples/custom_plugin)
package.

## License

MIT
