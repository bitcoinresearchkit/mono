# Bitview and the Bitcoin Research Kit

[![Bitview](https://img.shields.io/crates/v/bitviewd.svg?label=bitviewd)](https://crates.io/crates/bitviewd)

Bitview turns a Bitcoin Core node into a local Bitcoin data platform for chain
exploration, mempool data, and on-chain research. The official free hosted
instance is [bitview.space](https://bitview.space).

- **Bitcoin Research Kit (BRK):** reusable Rust primitives for reading Bitcoin
  Core data, RPC, mempool tracking, storage, domain types, and the on-chain
  price oracle.
- **Bitview:** the self-hostable application that composes BRK with a typed
  indexer and analytics plugin graph, query layer, REST server, showcases, and
  generated clients.

**Used by**

<a href="https://x.com/_checkonchain"><img src="https://pbs.twimg.com/profile_images/2090310945637855234/YA09g3v9_400x400.jpg" width="50" alt="Checkonchain" title="Checkonchain" style="border-radius:50%" /></a>
<a href="https://x.com/newhedge_io"><img src="https://pbs.twimg.com/profile_images/1575960313916178448/taaprZ6q_400x400.jpg" width="50" alt="Newhedge" title="Newhedge" style="border-radius:50%" /></a>
<a href="https://x.com/qfilabs"><img src="https://pbs.twimg.com/profile_images/2087768325904322560/z2xeJsWy_400x400.jpg" width="50" alt="QFI Labs" title="QFI Labs" style="border-radius:50%" /></a>
<a href="https://x.com/TheBitcoinLayer"><img src="https://pbs.twimg.com/profile_images/1674609666507112449/yP-mZb5H_400x400.jpg" width="50" alt="The Bitcoin Layer" title="The Bitcoin Layer" style="border-radius:50%" /></a>
<a href="https://x.com/PapangLabs"><img src="https://pbs.twimg.com/profile_images/2090676888054231040/mIX07S9f_400x400.jpg" width="50" alt="Papang Labs" title="Papang Labs" style="border-radius:50%" /></a>
<a href="https://x.com/atreidis_app"><img src="https://pbs.twimg.com/profile_images/2050569617890783232/S8T4kt6w_400x400.png" width="50" alt="Atreidis" title="Atreidis" style="border-radius:50%" /></a>
<a href="https://x.com/SparrowWallet"><img src="https://pbs.twimg.com/profile_images/1299262054977556481/p8Za-LLX_400x400.jpg" width="50" alt="Sparrow Wallet" title="Sparrow Wallet" style="border-radius:50%" /></a>

See [Architecture](./ARCHITECTURE.md) for the component and data-flow details.

## Repository map

| Path | Purpose |
|---|---|
| `crates/brk_*` | Reusable Bitcoin primitives |
| `crates/bitview_plugin_*` | Official indexing and analytics plugins |
| `crates/bitview*` | Plugin contract, runtime, composition, query/server stack, daemon, code generation, and Rust clients |
| `crates/{vecdb,rawdb,byteview,fjall,lsm-tree}` | Storage and data infrastructure |
| [`examples/custom_plugin`](../examples/custom_plugin) | Runnable external-plugin and custom-composition example |
| [`modules/bitview-client`](../modules/bitview-client) | JavaScript/TypeScript client |
| [`packages/bitview_client`](../packages/bitview_client) | Python client |
| `showcases` | Showcase pages, served by the server, and Studio |
| `docs`, `benches`, `docker`, and `scripts` | Project documentation, measurements, packaging, and maintenance tooling |

## Use Bitview

The hosted showcases, API, and MCP endpoint require no account or authentication:

- [Showcases](https://bitview.space)
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

Find series identifiers with `/api/series/search?q=<concept>`.

## Self-host Bitview

```bash
rustup update && RUSTFLAGS="-C target-cpu=native" cargo install --locked bitviewd
bitviewd
```

The default composition needs Linux or macOS, a mainnet Bitcoin Core node with
RPC and complete `blk*.dat` files from genesis, about 290 GiB for Bitview plus
Bitcoin Core storage and growth headroom, and 16 GB of RAM recommended for a
full sync. The showcases and API listen on `0.0.0.0:3110`
([localhost:3110](http://localhost:3110) locally).

See the [`bitviewd` guide](../crates/bitviewd) for requirements, disk usage,
installation, configuration, initial sync, and custom compositions.

## Build with BRK

```toml
[dependencies]
brk_reader = "0.12"
brk_rpc = "0.12"
brk_types = "0.12"
```

For a new Bitview metric or composition, start with the
[custom plugin example](../examples/custom_plugin) and the
[`bitview_plugin` contract](../crates/bitview_plugin).

## Develop

The repository pins its Rust toolchain in `rust-toolchain.toml`.

```bash
cargo check --workspace
cargo api   # records the API snapshots
cargo bindgen   # regenerates the clients
```

## Documentation

- [Changelog](./CHANGELOG.md)
- [Professional hosting](./PROFESSIONAL_HOSTING.md)

## Support and community

BRK is supported by [OpenSats](https://opensats.org/) from December 2024 through
June 2027.

[Discord](https://discord.gg/WACpShCB7M) ·
[X](https://x.com/_nym21_) ·
[Issues](https://github.com/bitcoinresearchkit/brk/issues) ·
[Support](mailto:support@bitcoinresearchkit.org)

### Donations

<a href="https://x.com/_Checkmatey_"><img src="https://pbs.twimg.com/profile_images/1657255419172253698/ncG0Gt8e_400x400.jpg" width="50" alt="_Checkmatey_" title="_Checkmatey_" style="border-radius:50%" /></a>
<a href="https://x.com/JohanMBergman"><img src="https://pbs.twimg.com/profile_images/1958587470120988673/7rlY5csu_400x400.jpg" width="50" alt="Johan Bergman" title="Johan Bergman" style="border-radius:50%" /></a>
<a href="https://x.com/alonshvartsman"><img src="https://pbs.twimg.com/profile_images/2057096517954007041/7nv7ygDB_400x400.jpg" width="50" alt="Alon Shvartsman" title="Alon Shvartsman" style="border-radius:50%" /></a>
<a href="https://x.com/clearmined1"><img src="https://pbs.twimg.com/profile_images/1657777901830541313/6OAaR8XF_400x400.png" width="50" alt="ClearMined" title="ClearMined" style="border-radius:50%" /></a>
<a href="https://x.com/PositiveCrypto"><img src="https://pbs.twimg.com/profile_images/1506441769424142336/msV99x2s_400x400.jpg" width="50" alt="Philip Swift" title="Philip Swift" style="border-radius:50%" /></a>

<img src="./qr.png" alt="Bitcoin donation QR code" width="120" />

[`bc1pkqvzn3pepug695xtvwmpm0sjd6n9j55v792cwfruzkqzrf2jn47s3hq0ts`](bitcoin:bc1pkqvzn3pepug695xtvwmpm0sjd6n9j55v792cwfruzkqzrf2jn47s3hq0ts)

## License

[MIT](../LICENSE)
