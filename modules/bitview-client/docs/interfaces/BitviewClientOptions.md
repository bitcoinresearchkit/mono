[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / BitviewClientOptions

# Interface: BitviewClientOptions

Defined in: [modules/bitview-client/index.js:1596](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1596)

## Properties

### baseUrl

> **baseUrl**: `string`

Defined in: [modules/bitview-client/index.js:1597](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1597)

***

### browserCache?

> `optional` **browserCache?**: `string` \| `boolean`

Defined in: [modules/bitview-client/index.js:1599](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1599)

Enable browser Cache API with default name (true), custom name (string), or disable (false). No effect in Node.js. Default: true

***

### memCache?

> `optional` **memCache?**: `number` \| `boolean`

Defined in: [modules/bitview-client/index.js:1600](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1600)

LRU capacity of the in-memory parsed-response cache, which lets 304 responses skip the JSON parse: true/undefined → 1000, false/0 → disabled. Every caller of a URL shares the value it holds, frozen deeply (bytes, a Uint8Array, can't be frozen: don't change them): copy it before changing it, or disable the cache to own every value

***

### timeout?

> `optional` **timeout?**: `number`

Defined in: [modules/bitview-client/index.js:1598](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1598)

Request timeout in milliseconds, the download included. Default: 30000
