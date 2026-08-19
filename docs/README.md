# Bitcoin Research Kit

[![MIT Licensed](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/bitcoinresearchkit/brk/blob/main/docs/LICENSE.md)
[![Crates.io](https://img.shields.io/crates/v/brk.svg)](https://crates.io/crates/brk)
[![docs.rs](https://img.shields.io/docsrs/brk)](https://docs.rs/brk)
[![Supported by OpenSats](https://img.shields.io/badge/supported%20by-opensats-ff7b00)](https://opensats.org/)
[![Discord](https://img.shields.io/discord/1350431684562124850?label=Discord&logo=discord&color=5865F2)](https://discord.gg/WACpShCB7M)
[![X](https://img.shields.io/badge/@_nym21_-000000?logo=x)](https://x.com/_nym21_)

> "Shout out to Bitcoin Research Kit. [...] Couldn't recommend them highly enough."
> — James Check (CheckOnChain), [What Bitcoin Did #1000](https://www.whatbitcoindid.com/episodes/wbd1000-checkmate)

Open-source Bitcoin data toolkit that can parse blocks, index the chain, compute metrics, serve data and render it, all from a Bitcoin Core node. It combines what [Glassnode](https://glassnode.com) and [mempool.space](https://mempool.space) do separately into a single self-hostable package, with a built-in price oracle inspired by [UTXO Oracle](https://utxo.live/oracle/).

[Bitview](https://bitview.space) is the official free hosted instance of BRK.
Stateless, read-only MCP access is available at
[mcp.bitview.space](https://mcp.bitview.space/), with no authentication required.

## Data

**Zero external dependencies.** BRK needs only a Bitcoin Core node. 8,000+ metrics across 15 time resolutions, all computed locally from your own copy of the blockchain. Historical prices are built in, live price from your mempool. Your node, your data.

**Blockchain:** Blocks, transactions, addresses, UTXOs.

**Metrics:** Supply distributions, holder cohorts, network activity, fee markets, mining, and market indicators.

**Indexes:** Date, height, halving epoch, address type, UTXO age.

**Mempool:** Fee estimation, projected blocks, unconfirmed transactions.

## Usage

### Website

Browse metrics and charts at [bitview.space](https://bitview.space), no signup required.

### API

```bash
curl https://bitview.space/api/mempool/price
```

Query metrics and blockchain data in JSON or CSV. No rate limit.

[Documentation](https://bitview.space/api) · [JavaScript](https://www.npmjs.com/package/bitview-client) · [Python](https://pypi.org/project/bitview-client) · [Rust](https://crates.io/crates/bitview_client) · [llms.txt](https://bitview.space/llms.txt) · [LLM-friendly schema](https://bitview.space/api.json)

### MCP

Connect any Streamable HTTP MCP client to:

```text
https://mcp.bitview.space/
```

The server is stateless, read-only, and requires no authentication. Its tools
are generated from the same OpenAPI operations as the typed clients.

### Self-host

```bash
cargo install --locked bitview && bitview
```

Run your own website and API. All you need is Bitcoin Core.

> **Note:** BRK uses [sparse files](https://en.wikipedia.org/wiki/Sparse_file). Tools like `ls -l` or Finder report the logical file size (>1 TB), not actual disk usage (~350 GB). Use `du -sh` to see real usage.

[Guide](https://github.com/bitcoinresearchkit/brk/blob/main/crates/bitview/README.md) · [Professional hosting](./PROFESSIONAL_HOSTING.md)

### Library

```bash
cargo add brk
```

Build custom Bitcoin tooling from BRK primitives such as the block reader,
domain types, RPC client, oracle, and storage components. Use Bitview for the
composed indexer, compute plugins, query layer, and server.

[Reference](https://docs.rs/brk) · [Architecture](./ARCHITECTURE.md)

## Supporters

- [OpenSats](https://opensats.org/) (December 2024 - June 2027)

[Become a supporter](mailto:support@bitcoinresearchkit.org)

## Donations

<a href="https://x.com/_Checkmatey_"><img src="https://pbs.twimg.com/profile_images/1657255419172253698/ncG0Gt8e_400x400.jpg" width="40" alt="_Checkmatey_" title="_Checkmatey_" style="border-radius:50%" /></a>
<a href="https://x.com/JohanMBergman"><img src="https://pbs.twimg.com/profile_images/1958587470120988673/7rlY5csu_400x400.jpg" width="40" alt="Johan Bergman" title="Johan Bergman" style="border-radius:50%" /></a>
<a href="https://x.com/alonshvartsman"><img src="https://pbs.twimg.com/profile_images/2005689891028406272/8Qgmnurs_400x400.jpg" width="40" alt="Alon Shvartsman" title="Alon Shvartsman" style="border-radius:50%" /></a>
<a href="https://x.com/clearmined1"><img src="https://pbs.twimg.com/profile_images/1657777901830541313/6OAaR8XF_400x400.png" width="40" alt="ClearMined" title="ClearMined" style="border-radius:50%" /></a>

<img src="./qr.png" alt="Bitcoin donate QR code" width="120" />

[`bc1q09 8zsm89 m7kgyz e338vf ejhpdt 92ua9p 3peuve`](bitcoin:bc1q098zsm89m7kgyze338vfejhpdt92ua9p3peuve)

## Links

- [Changelog](./CHANGELOG.md)
- [Contributing](https://github.com/bitcoinresearchkit/brk/issues)

## Community

- [Discord](https://discord.gg/WACpShCB7M)
- [X](https://x.com/_nym21_)
- [Nostr](https://primal.net/p/nprofile1qqsfw5dacngjlahye34krvgz7u0yghhjgk7gxzl5ptm9v6n2y3sn03sqxu2e6)

## License

[MIT](./LICENSE.md)
