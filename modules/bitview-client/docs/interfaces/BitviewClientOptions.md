[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / BitviewClientOptions

# Interface: BitviewClientOptions

Defined in: [modules/bitview-client/index.js:1722](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1722)

## Properties

### baseUrl

> **baseUrl**: `string`

Defined in: [modules/bitview-client/index.js:1723](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1723)

***

### browserCache?

> `optional` **browserCache?**: `string` \| `boolean`

Defined in: [modules/bitview-client/index.js:1725](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1725)

Enable browser Cache API with default name (true), custom name (string), or disable (false). No effect in Node.js. Default: true

***

### memCache?

> `optional` **memCache?**: `number` \| `boolean`

Defined in: [modules/bitview-client/index.js:1726](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1726)

LRU capacity of the in-memory parsed-response cache, which lets 304 responses skip the JSON parse: true/undefined → 1000, false/0 → disabled

***

### timeout?

> `optional` **timeout?**: `number`

Defined in: [modules/bitview-client/index.js:1724](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1724)

Request timeout in milliseconds, the download included. Default: 30000
