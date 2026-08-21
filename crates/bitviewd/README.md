# Bitview

Bitview is a composable, self-hostable Bitcoin data platform built on the
[Bitcoin Research Kit](https://bitcoinresearchkit.org). Learn more at
[bitview.dev](https://bitview.dev).

This package provides the `bitviewd` process shell and executable: command-line
arguments, configuration files, logging, signal handling, and the official
plugin composition. The composition-agnostic daemon runner lives in the
`bitview` crate.

[bitview.space](https://bitview.space) is the official free hosted instance.
For AI clients, the official stateless, read-only MCP endpoint is
[mcp.bitview.space](https://mcp.bitview.space/). It requires no authentication.

## Requirements

- Linux or macOS
- Bitcoin Core with `server=1` in `bitcoin.conf`
- Access to `blk*.dat` files
- About 300 GiB of disk space for the current default composition, plus Bitcoin
  Core storage and growth headroom (see [Disk usage](#disk-usage))
- 16 GB of RAM recommended for a full sync.

## Disk usage

Bitview storage uses [sparse files](https://en.wikipedia.org/wiki/Sparse_file).
Tools like `ls -l` or Finder report the logical file size (>1 TB), not actual
disk usage. Use `du -sh ~/.bitview` to see allocated space.

## Install

```bash
rustup update && RUSTFLAGS="-C target-cpu=native" cargo install --locked bitviewd --version $(cargo search bitviewd | head -1 | awk -F'"' '{print $2}')
```

This updates Rust, then builds Bitview with optimizations tuned to your CPU. The
version lookup selects the newest published release, including prereleases;
without it, `cargo install` selects the latest stable release.

Portable build (without native CPU optimizations):

```bash
cargo install --locked bitviewd
```

## Update

Re-run the install command. Cargo replaces the existing executable. Indexed
data is reused when its on-disk format is unchanged; otherwise it is reset and
resynced automatically on the next run.

## Run

```bash
bitviewd
```

Bitview indexes the blockchain, computes datasets, starts the server on
`localhost:3110`, and waits for new blocks.

## First sync

The initial sync processes the entire blockchain and can take several hours.
While more than 10,000 blocks behind, indexing completes before the server
starts to reduce memory use. The web interface at `localhost:3110` becomes
available after the sync finishes.

## Options

```bash
bitviewd -h       # Show all options
bitviewd -V       # Show version
```

Command-line options override `~/.bitview/config.toml` for that run only. Edit the
file directly to persist settings:

```toml
bitviewdir = "/path/to/data"
bitcoindir = "/path/to/.bitcoin"
```

All fields are optional. See `bitviewd -h` for the full list.

## Environment variables

```bash
LOG=debug bitviewd    # Enable debug logging while retaining noise filters
RUST_LOG=... bitviewd # Control log filtering directly
```

## Files

```text
~/.bitview/
├── config.toml  Daemon configuration
├── logs/        Runtime logs
└── plugins/     One directory per active plugin ID
```

`~/.bitview` is the default data directory and can be changed with
`--bitviewdir`.

The active composition owns the entire `plugins/` directory. At startup,
Bitview removes every entry whose name is not claimed by an active plugin ID.
Use a different `bitviewdir` when omitted plugin data must be preserved.

Plugin compatibility is defined separately by
[`bitview_plugin`](https://crates.io/crates/bitview_plugin). The platform and
plugin APIs remain experimental while the built-in modules are extracted into
independent plugins.

## Custom plugins

The [custom plugin example](https://github.com/bitcoinresearchkit/brk/tree/main/examples/custom_plugin) is a complete, runnable
template with persistent storage, typed dependencies, reorg-safe computation,
composition, read-only queries, and automatic series API exposure.

Custom compositions can reuse the daemon shell without compiling the official
composition:

```toml
bitviewd = { version = "0.11.2", default-features = false, features = ["series"] }
```

Plugin features flow through `bitview` and `bitview_server` to `bitview_query`,
so only the selected typed API surface and its plugin crates are compiled. The
indexer is the mandatory runner baseline.

Use `features = ["full-api"]` to enable the complete chain, series, and URPD API
without selecting `bitview_default`.

## License

MIT
