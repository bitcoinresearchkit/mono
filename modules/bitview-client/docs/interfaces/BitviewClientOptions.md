[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / BitviewClientOptions

# Interface: BitviewClientOptions

Defined in: [Developer/brk/modules/bitview-client/index.js:1577](https://github.com/bitcoinresearchkit/brk/blob/b971f8d1b413e122481dddc3c9c980b7866ab1d3/modules/bitview-client/index.js#L1577)

## Properties

### baseUrl

> **baseUrl**: `string`

Defined in: [Developer/brk/modules/bitview-client/index.js:1578](https://github.com/bitcoinresearchkit/brk/blob/b971f8d1b413e122481dddc3c9c980b7866ab1d3/modules/bitview-client/index.js#L1578)

Base URL for the API

***

### browserCache?

> `optional` **browserCache?**: `string` \| `boolean`

Defined in: [Developer/brk/modules/bitview-client/index.js:1580](https://github.com/bitcoinresearchkit/brk/blob/b971f8d1b413e122481dddc3c9c980b7866ab1d3/modules/bitview-client/index.js#L1580)

Enable browser Cache API with default name (true), custom name (string), or disable (false). No effect in Node.js. Default: true

***

### memCache?

> `optional` **memCache?**: `number` \| `boolean`

Defined in: [Developer/brk/modules/bitview-client/index.js:1581](https://github.com/bitcoinresearchkit/brk/blob/b971f8d1b413e122481dddc3c9c980b7866ab1d3/modules/bitview-client/index.js#L1581)

In-memory parsed-response cache size (LRU). true/undefined → 1000, false/0 → disabled. Lets 304 responses skip the JSON parse entirely. Default: 1000

***

### timeout?

> `optional` **timeout?**: `number`

Defined in: [Developer/brk/modules/bitview-client/index.js:1579](https://github.com/bitcoinresearchkit/brk/blob/b971f8d1b413e122481dddc3c9c980b7866ab1d3/modules/bitview-client/index.js#L1579)

Request timeout in milliseconds
