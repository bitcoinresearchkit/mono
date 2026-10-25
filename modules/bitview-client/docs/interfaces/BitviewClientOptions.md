[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / BitviewClientOptions

# Interface: BitviewClientOptions

Defined in: [modules/bitview-client/index.js:1762](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1762)

## Properties

### baseUrl

> **baseUrl**: `string`

Defined in: [modules/bitview-client/index.js:1763](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1763)

***

### browserCache?

> `optional` **browserCache?**: `string` \| `boolean`

Defined in: [modules/bitview-client/index.js:1765](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1765)

Enable browser Cache API with default name (true), custom name (string), or disable (false). No effect in Node.js. Default: true

***

### memCache?

> `optional` **memCache?**: `number` \| `boolean`

Defined in: [modules/bitview-client/index.js:1766](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1766)

LRU capacity of the in-memory parsed-response cache, which lets 304 responses skip the JSON parse: true/undefined → 1000, false/0 → disabled

***

### timeout?

> `optional` **timeout?**: `number`

Defined in: [modules/bitview-client/index.js:1764](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1764)

Request timeout in milliseconds, the download included. Default: 30000
