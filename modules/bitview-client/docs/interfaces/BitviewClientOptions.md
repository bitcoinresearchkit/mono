[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / BitviewClientOptions

# Interface: BitviewClientOptions

Defined in: [modules/bitview-client/index.js:1690](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1690)

## Properties

### baseUrl

> **baseUrl**: `string`

Defined in: [modules/bitview-client/index.js:1691](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1691)

***

### browserCache?

> `optional` **browserCache?**: `string` \| `boolean`

Defined in: [modules/bitview-client/index.js:1693](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1693)

Enable browser Cache API with default name (true), custom name (string), or disable (false). No effect in Node.js. Default: true

***

### memCache?

> `optional` **memCache?**: `number` \| `boolean`

Defined in: [modules/bitview-client/index.js:1694](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1694)

LRU capacity of the in-memory parsed-response cache, which lets 304 responses skip the JSON parse: true/undefined → 1000, false/0 → disabled

***

### timeout?

> `optional` **timeout?**: `number`

Defined in: [modules/bitview-client/index.js:1692](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1692)

Request timeout in milliseconds, the download included. Default: 30000
