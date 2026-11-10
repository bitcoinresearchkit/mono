# Bitview

Live and historical Bitcoin data for Claude Code from [Bitview](https://bitview.space), a free, open-source data server read from a full node. It covers on-chain metrics such as MVRV, realized price, SOPR, holder cohorts and supply by age, as well as blocks, transactions, addresses, mempool and fees, mining and difficulty, and the BTC/USD price computed from the chain.

## Install

```
claude plugin marketplace add bitcoinresearchkit/mono
claude plugin install bitview@bitview
```

## What it adds

- **The `bitview` skill.** It tells Claude where Bitview's API reference is (`https://bitview.space/llms.txt`) and how to find a series: search, read the series, then pick an index.
- **The `bitview` MCP server** at `https://mcp.bitview.space/`. It is remote, stateless and read-only, and offers one tool for each API operation.

## What it runs and sends

The plugin runs nothing on your machine.

When Claude uses Bitview, it sends HTTPS GET requests to `bitview.space` or `mcp.bitview.space`. These carry only the query: the series ids, indexes, ranges, block hashes, transaction ids or addresses being looked up.

- No account, API key or credential is needed or read.
- Request logs keep no IP address, user agent or other client detail.

## Source

The skill and the plugin manifest are generated from Bitview's OpenAPI document by `crates/bitview_bindgen` in <https://github.com/bitcoinresearchkit/mono>. The project is MIT licensed.

Support: support@bitview.space
