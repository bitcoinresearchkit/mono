**bitview-client**

***

# bitview-client

JavaScript/TypeScript client for the [Bitcoin Research Kit](https://github.com/bitcoinresearchkit/brk) API.

Zero dependencies.

[npm](https://www.npmjs.com/package/bitview-client) | [API Reference](https://github.com/bitcoinresearchkit/brk/blob/main/modules/bitview-client/docs/globals.md)

AI clients can use the same API through the official stateless, read-only MCP
endpoint at [mcp.bitview.space](https://mcp.bitview.space/). No authentication
is required.

## Installation

```bash
npm install bitview-client
```

Or just copy [`index.js`](globals.md) into your project - it's a single file with no dependencies.

## Quick Start

```javascript
import { BitviewClient } from 'bitview-client';

// Use the free public API or your own instance
const client = new BitviewClient('https://bitview.space');
// or: `const client = new BitviewClient({ baseUrl: 'https://bitview.space', timeout: 10000 });`

// Blockchain data (mempool.space compatible)
const block = await client.getBlockByHeight(800000);
const tx = await client.getTx('abc123...');
const address = await client.getAddress('bc1q...');

// Metrics API - typed, chainable
const prices = await client.metrics.price.usd.split.close
  .by.dateindex
  .last(30); // Last 30 items

// Generic metric fetching
const data = await client.getMetric('price_close', 'dateindex', -30);
```
