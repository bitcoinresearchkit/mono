# Bitview and the Bitcoin Research Kit

[![MIT Licensed](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/bitcoinresearchkit/brk/blob/main/LICENSE)
[![Bitview](https://img.shields.io/crates/v/bitviewd.svg?label=bitviewd)](https://crates.io/crates/bitviewd)
[![BRK](https://img.shields.io/crates/v/brk.svg?label=brk)](https://crates.io/crates/brk)
[![Supported by OpenSats](https://img.shields.io/badge/supported%20by-opensats-ff7b00)](https://opensats.org/)
[![Discord](https://img.shields.io/discord/1350431684562124850?label=Discord&logo=discord&color=5865F2)](https://discord.gg/WACpShCB7M)

This monorepo contains two layers:

- **Bitcoin Research Kit (BRK):** reusable Rust primitives for reading Bitcoin
  Core data, RPC, mempool tracking, storage, domain types, and the on-chain
  price oracle.
- **Bitview:** the self-hostable application that composes BRK with a typed
  indexer and analytics plugin graph, query layer, REST server, website, and
  generated clients.

Together they turn a Bitcoin Core node into a local Bitcoin data platform for
chain exploration, mempool data, and on-chain research. The official free
hosted instance is [bitview.space](https://bitview.space).

```text
Bitcoin Core ──> BRK reader, RPC, and mempool ──> Bitview plugins
                                                      │
                                                      v
                                            Query + REST server
                                              │      │      │
                                              v      v      v
                                           Website Clients  MCP
```

See [Architecture](./ARCHITECTURE.md) for the component and data-flow details.

## Repository map

| Path | Purpose |
|---|---|
| [`crates/brk`](../crates/brk) and `crates/brk_*` | BRK umbrella crate and reusable Bitcoin primitives |
| `crates/bitview_plugin_*` | Official indexing and analytics plugins |
| `crates/bitview*` | Plugin contract, runtime, composition, query/server stack, daemon, code generation, and Rust clients |
| `crates/{vecdb,rawdb,byteview,fjall,lsm-tree}` | Storage and data infrastructure |
| [`examples/custom_plugin`](../examples/custom_plugin) | Runnable external-plugin and custom-composition example |
| [`modules/bitview-client`](../modules/bitview-client) | JavaScript/TypeScript client |
| [`packages/bitview_client`](../packages/bitview_client) | Python client |
| `website*` and `experiments` | Browser applications and research visualizations |
| `docs`, `benches`, `docker`, and `scripts` | Project documentation, measurements, packaging, and maintenance tooling |

Each publishable crate or client owns its detailed usage documentation. The
main README is only the map and common entrypoint.

## Use Bitview

The hosted website, API, and MCP endpoint require no account or authentication:

- [Website](https://bitview.space)
- [Interactive API](https://bitview.space/api)
- [OpenAPI](https://bitview.space/openapi.json)
- MCP client endpoint: `https://mcp.bitview.space/`
- [CLI](https://crates.io/crates/bitview_cli)
- [JavaScript](https://www.npmjs.com/package/bitview-client)
- [Python](https://pypi.org/project/bitview-client)
- [Rust](https://crates.io/crates/bitview_client)

```bash
curl https://bitview.space/api/mempool/price
curl https://bitview.space/api/series/count
```

The series catalog is discovered at runtime rather than documented as a
hard-coded count. Use `/api/series/search?q=<concept>` to find identifiers.

## Self-host Bitview

```bash
cargo install --locked bitviewd
bitviewd
```

The default composition requires Linux or macOS, a Bitcoin Core node with RPC
and readable `blk*.dat` files, about 300 GiB for Bitview plus Bitcoin Core
storage and growth headroom, and 16 GB of RAM recommended for a full sync. The
website and API listen on [localhost:3110](http://localhost:3110).

Bitview uses sparse files, so logical size is much larger than allocated disk
space. Use `du -sh ~/.bitview` to measure actual usage.

See the [`bitviewd` guide](../crates/bitviewd) for installation, configuration,
initial sync, storage, and custom compositions.

## Build with BRK

Use the umbrella crate for BRK primitives without the full Bitview application:

```toml
[dependencies]
brk = { version = "0.11", features = ["reader", "rpc", "types"] }
```

See the [`brk` crate guide](../crates/brk) for its feature map. For a new
Bitview metric or composition, start with the
[custom plugin example](../examples/custom_plugin) and the
[`bitview_plugin` contract](../crates/bitview_plugin).

## Develop

The repository pins its Rust toolchain in `rust-toolchain.toml`.

```bash
cargo check --workspace
cargo test --workspace
```

## Documentation

- [Architecture](./ARCHITECTURE.md)
- [Changelog](./CHANGELOG.md)
- [Self-hosting](../crates/bitviewd)
- [BRK crates](../crates/brk)
- [Plugin contract](../crates/bitview_plugin)
- [Custom plugin example](../examples/custom_plugin)
- [Professional hosting](./PROFESSIONAL_HOSTING.md)

## Support and community

BRK is supported by [OpenSats](https://opensats.org/) from December 2024 through
June 2027.

[Discord](https://discord.gg/WACpShCB7M) ·
[X](https://x.com/_nym21_) ·
[Issues](https://github.com/bitcoinresearchkit/brk/issues) ·
[Support](mailto:support@bitcoinresearchkit.org)

<img src="./qr.png" alt="Bitcoin donation QR code" width="120" />

[`bc1pkqvzn3pepug695xtvwmpm0sjd6n9j55v792cwfruzkqzrf2jn47s3hq0ts`](bitcoin:bc1pkqvzn3pepug695xtvwmpm0sjd6n9j55v792cwfruzkqzrf2jn47s3hq0ts)

## License

[MIT](../LICENSE)
