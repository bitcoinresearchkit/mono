# Bitview

Bitview is a composable, self-hostable Bitcoin data platform built on the
[Bitcoin Research Kit](https://bitcoinresearchkit.org). Learn more at
[bitview.dev](https://bitview.dev).

This package provides the `bitviewd` executable: the official plugin composition
(`bitview_default`) run by the `bitview` daemon runtime, which owns command-line
arguments, configuration files, logging, signal handling, and serving.

[bitview.space](https://bitview.space) is the official free hosted instance.

## Requirements

- Linux or macOS
- Bitcoin Core on mainnet (testnet, signet and regtest are not supported) with
  `server=1` in `bitcoin.conf`
- Access to `blk*.dat` files, complete from genesis for the initial sync (no
  pruned node)
- About 290 GiB of disk space for the current default composition, plus Bitcoin
  Core storage and growth headroom (see [Disk usage](#disk-usage))
- 16 GB of RAM recommended for a full sync (see [perf.bitview.dev](https://perf.bitview.dev)
  for what each measured run used).

## Disk usage

Bitview storage uses [sparse files](https://en.wikipedia.org/wiki/Sparse_file).
Tools like `ls -l` or Finder report the logical file size (>1 TB), not actual
disk usage. Use `du -sh ~/.bitview` to see allocated space.

## Install

```bash
rustup update && RUSTFLAGS="-C target-cpu=native" cargo install --locked bitviewd
```

This updates Rust, then builds the latest stable Bitview release with
optimizations tuned to your CPU.

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

Bitview indexes the blockchain, computes datasets, starts the server (at
`http://localhost:3110`, this machine only: set `serverbind = "0.0.0.0"` to serve
your network, behind access control, since it has no authentication), and waits for
new blocks.

## Generating clients during development

From the workspace, `cargo dev` regenerates the clients and starts the daemon; see the
Regenerating section of [`bitview_bindgen`](https://crates.io/crates/bitview_bindgen) for
`cargo bindgen` and `cargo prod`.

## First sync

The initial sync processes the entire blockchain: about an hour and a half on a 16 GB Mac
mini with a fast SSD, longer on slower disks and CPUs. Every measured run, with its
memory, disk and CPU, is at [perf.bitview.dev](https://perf.bitview.dev).
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

Plugin compatibility is defined by
[`bitview_plugin`](https://crates.io/crates/bitview_plugin). The platform and
plugin APIs are experimental.

## Custom plugins

The [custom plugin example](https://github.com/bitcoinresearchkit/mono/tree/main/examples/custom_plugin)
is a complete, runnable template. Custom binaries call
[`bitview::run`](https://crates.io/crates/bitview) with their own composition
(extending `bitview_default` or not); see the `bitview` README for features.

## License

MIT
