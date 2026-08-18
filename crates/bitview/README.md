# Bitview

Bitview is a composable, self-hostable Bitcoin data platform built on the
[Bitcoin Research Kit](https://bitcoinresearchkit.org). Learn more at
[bitview.dev](https://bitview.dev).

This crate provides the official composition: Bitcoin Core reading, indexing,
dataset computation, mempool tracking, queries, and the HTTP server. The `brk`
binary runs it through `bitview::run()`.

Plugin compatibility is defined separately by
[`bitview_plugin`](https://crates.io/crates/bitview_plugin). The platform and
plugin APIs remain experimental while the built-in modules are extracted into
independent plugins.

Bitcoin data can also be explored through the official hosted instance at
[bitview.space](https://bitview.space).

## License

MIT
