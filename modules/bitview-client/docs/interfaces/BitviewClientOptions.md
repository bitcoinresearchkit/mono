[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / BitviewClientOptions

# Interface: BitviewClientOptions

Defined in: [modules/bitview-client/index.js:1637](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1637)

## Properties

### baseUrl

> **baseUrl**: `string`

Defined in: [modules/bitview-client/index.js:1638](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1638)

***

### browserCache?

> `optional` **browserCache?**: `string` \| `boolean`

Defined in: [modules/bitview-client/index.js:1640](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1640)

Enable browser Cache API with default name (true), custom name (string), or disable (false). No effect in Node.js. Default: true

***

### memCache?

> `optional` **memCache?**: `number` \| `boolean`

Defined in: [modules/bitview-client/index.js:1641](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1641)

LRU capacity of the in-memory parsed-response cache, which lets 304 responses skip the JSON parse: true/undefined → 1000, false/0 → disabled

***

### timeout?

> `optional` **timeout?**: `number`

Defined in: [modules/bitview-client/index.js:1639](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1639)

Request timeout in milliseconds
