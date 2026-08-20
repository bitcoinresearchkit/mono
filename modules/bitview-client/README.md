# bitview-client

Zero-dependency JavaScript/TypeScript client for the [Bitview](https://bitview.space) Bitcoin analytics API.

[npm](https://www.npmjs.com/package/bitview-client) | [API Reference](https://github.com/bitcoinresearchkit/brk/blob/main/modules/bitview-client/docs/globals.md) | [Source](https://github.com/bitcoinresearchkit/brk/tree/main/modules/bitview-client)

AI clients can use the same API through the official stateless, read-only MCP
endpoint at [mcp.bitview.space](https://mcp.bitview.space/). No authentication
is required.

## Installation

```bash
npm install bitview-client
```

You can also copy the `index.js` file into a project. The published client is a
single ES module with no runtime dependencies.

## Quick start

```javascript
import { BitviewClient } from 'bitview-client';

// Use the public API or point the client at a self-hosted Bitview server.
const client = new BitviewClient('https://bitview.space');

// Mempool.space-compatible blockchain endpoints.
const blockHash = await client.getBlockByHeight(800000);
const tx = await client.getTx(
  'a1075db55d416d3ca199f55b6084e2115b9345e16c5cf302fc80e9d5fbf5d48d',
);

// Typed, chainable series access.
const prices = await client.series.price.split.close.usd.by.day1
  .last(30)
  .fetch();

// Programmatic access when the series name is only known at runtime.
const samePrices = await client
  .seriesEndpoint('price_close', 'day1')
  .last(30)
  .fetch();
```

Series endpoints support `first(n)`, `last(n)`, `slice(start, end)`, `get(i)`,
`skip(n).take(m)`, `fetchCsv()`, `len()`, and `version()`. They are also
thenable, so `await endpoint.last(30)` is equivalent to calling `.fetch()`.

Pass an options object to configure request behavior:

```javascript
const client = new BitviewClient({
  baseUrl: 'https://bitview.space',
  timeout: 10_000,
  browserCache: true,
  memCache: 100,
});
```
