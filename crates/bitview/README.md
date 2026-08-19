# Bitview

Bitview is a composable, self-hostable Bitcoin data platform built on the
[Bitcoin Research Kit](https://bitcoinresearchkit.org). Learn more at
[bitview.dev](https://bitview.dev).

This package provides both the official composition library and the `bitview`
executable: Bitcoin Core reading, indexing, dataset computation, mempool
tracking, queries, the website, and the HTTP server.

[bitview.space](https://bitview.space) is the official free hosted instance.
For AI clients, the official stateless, read-only MCP endpoint is
[mcp.bitview.space](https://mcp.bitview.space/). It requires no authentication.

## Requirements

- Linux or macOS
- Bitcoin Core with `server=1` in `bitcoin.conf`
- Access to `blk*.dat` files
- [~400 GB disk space](https://bitview.space/api/server/disk) (see [Disk usage](#disk-usage))
- [12+ GB RAM](https://github.com/bitcoinresearchkit/benches#benchmarks)

## Disk usage

BRK storage uses [sparse files](https://en.wikipedia.org/wiki/Sparse_file).
Tools like `ls -l` or Finder report the logical file size (>1 TB), not actual
disk usage (~350 GB). Use `du -sh` to see real usage.

## Install

```bash
rustup update && RUSTFLAGS="-C target-cpu=native" cargo install --locked bitview --version $(cargo search bitview | head -1 | awk -F'"' '{print $2}')
```

This updates Rust, then builds Bitview with optimizations tuned to your CPU. The
version lookup selects the newest published release, including prereleases;
without it, `cargo install` selects the latest stable release.

Portable build (without native CPU optimizations):

```bash
cargo install --locked bitview
```

## Update

Re-run the install command. Cargo replaces the existing executable. Indexed
data is reused when its on-disk format is unchanged; otherwise it is reset and
resynced automatically on the next run.

## Run

```bash
bitview
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
bitview -h       # Show all options
bitview -V       # Show version
```

Command-line options override `~/.bitview/config.toml` for that run only. Edit the
file directly to persist settings:

```toml
bitviewdir = "/path/to/data"
bitcoindir = "/path/to/.bitcoin"
```

All fields are optional. See `bitview -h` for the full list.

## Environment variables

```bash
LOG=debug bitview    # Enable debug logging while retaining noise filters
RUST_LOG=... bitview # Control log filtering directly
```

## Files

```text
~/.bitview/
├── config.toml  Runner configuration
├── logs/        Runtime logs
└── plugins/     One directory per active plugin ID
```

`~/.bitview` is the default data directory and can be changed with
`--bitviewdir`.

Plugin compatibility is defined separately by
[`bitview_plugin`](https://crates.io/crates/bitview_plugin). The platform and
plugin APIs remain experimental while the built-in modules are extracted into
independent plugins.

## Custom plugins

The [custom plugin example](examples/custom_plugin/) is a complete, runnable
template with persistent storage, typed dependencies, reorg-safe computation,
composition, read-only queries, and automatic series API exposure.

## License

MIT
