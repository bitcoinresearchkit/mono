[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / BitviewClientOptions

# Interface: BitviewClientOptions

Defined in: [modules/bitview-client/index.js:1661](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1661)

## Properties

### baseUrl

> **baseUrl**: `string`

Defined in: [modules/bitview-client/index.js:1662](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1662)

***

### browserCache?

> `optional` **browserCache?**: `string` \| `boolean`

Defined in: [modules/bitview-client/index.js:1664](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1664)

Enable browser Cache API with default name (true), custom name (string), or disable (false). No effect in Node.js. Default: true

***

### memCache?

> `optional` **memCache?**: `number` \| `boolean`

Defined in: [modules/bitview-client/index.js:1665](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1665)

LRU capacity of the in-memory parsed-response cache, which lets 304 responses skip the JSON parse: true/undefined → 1000, false/0 → disabled

***

### timeout?

> `optional` **timeout?**: `number`

Defined in: [modules/bitview-client/index.js:1663](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1663)

Request timeout in milliseconds
