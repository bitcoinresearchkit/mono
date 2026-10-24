[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / BitviewClientOptions

# Interface: BitviewClientOptions

Defined in: [modules/bitview-client/index.js:1672](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1672)

## Properties

### baseUrl

> **baseUrl**: `string`

Defined in: [modules/bitview-client/index.js:1673](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1673)

***

### browserCache?

> `optional` **browserCache?**: `string` \| `boolean`

Defined in: [modules/bitview-client/index.js:1675](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1675)

Enable browser Cache API with default name (true), custom name (string), or disable (false). No effect in Node.js. Default: true

***

### memCache?

> `optional` **memCache?**: `number` \| `boolean`

Defined in: [modules/bitview-client/index.js:1676](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1676)

LRU capacity of the in-memory parsed-response cache, which lets 304 responses skip the JSON parse: true/undefined → 1000, false/0 → disabled

***

### timeout?

> `optional` **timeout?**: `number`

Defined in: [modules/bitview-client/index.js:1674](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1674)

Request timeout in milliseconds
