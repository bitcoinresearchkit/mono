[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / BitviewClientOptions

# Interface: BitviewClientOptions

Defined in: [modules/bitview-client/index.js:1666](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1666)

## Properties

### baseUrl

> **baseUrl**: `string`

Defined in: [modules/bitview-client/index.js:1667](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1667)

***

### browserCache?

> `optional` **browserCache?**: `string` \| `boolean`

Defined in: [modules/bitview-client/index.js:1669](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1669)

Enable browser Cache API with default name (true), custom name (string), or disable (false). No effect in Node.js. Default: true

***

### memCache?

> `optional` **memCache?**: `number` \| `boolean`

Defined in: [modules/bitview-client/index.js:1670](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1670)

LRU capacity of the in-memory parsed-response cache, which lets 304 responses skip the JSON parse: true/undefined → 1000, false/0 → disabled

***

### timeout?

> `optional` **timeout?**: `number`

Defined in: [modules/bitview-client/index.js:1668](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1668)

Request timeout in milliseconds, the download included. Default: 30000
