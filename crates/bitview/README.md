# Bitview runtime

`bitview` is the daemon runtime for a statically typed Bitview plugin set. It
reads the configuration file and arguments, initializes logging and the shutdown
handler, and owns bootstrap, updates, mempool tracking, queries, and HTTP
serving. It never selects a composition: the official one lives in
`bitview_default` and the official binary is
[`bitviewd`](https://crates.io/crates/bitviewd).

- `bitview::run(import)` is the whole daemon for a composition's import
  function: what `bitviewd` and custom binaries call from `main`.
- `bitview::Config` is the file/argument configuration (`~/.bitview/config.toml`);
  `Config::import()` resolves it into a `RunConfig`.

The runner is designed for one long-lived instance per process. Its query view
and process-wide services intentionally live until process exit; it is not a
start-stop or multi-instance application server.

## Custom compositions

The [custom plugin example](https://github.com/bitcoinresearchkit/brk/tree/main/examples/custom_plugin)
is a complete, runnable template with persistent storage, typed dependencies,
reorg-safe computation, composition, read-only queries, and automatic series API
exposure. A custom binary depends on `bitview` plus its own composition (which
may extend `bitview_default`, as the example does, or start from scratch):

```toml
bitview = { version = "0.12", features = ["series"] }
```

Route-family features (`chain`, `series`, `urpd`, `price`) flow through
`bitview_server` to `bitview_query`, so only the selected typed API surface and
its plugin crates are compiled. The indexer is the mandatory runner baseline.
`full-api` enables the complete chain, series, and URPD API.

## License

MIT
