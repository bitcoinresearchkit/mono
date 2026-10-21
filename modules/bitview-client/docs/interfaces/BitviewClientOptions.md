[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / BitviewClientOptions

# Interface: BitviewClientOptions

Defined in: [modules/bitview-client/index.js:1570](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1570)

## Properties

### baseUrl

> **baseUrl**: `string`

Defined in: [modules/bitview-client/index.js:1571](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1571)

***

### browserCache?

> `optional` **browserCache?**: `string` \| `boolean`

Defined in: [modules/bitview-client/index.js:1573](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1573)

Enable browser Cache API with default name (true), custom name (string), or disable (false). No effect in Node.js. Default: true

***

### memCache?

> `optional` **memCache?**: `number` \| `boolean`

Defined in: [modules/bitview-client/index.js:1574](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1574)

LRU capacity of the in-memory parsed-response cache, which lets 304 responses skip the JSON parse: true/undefined → 1000, false/0 → disabled

***

### timeout?

> `optional` **timeout?**: `number`

Defined in: [modules/bitview-client/index.js:1572](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1572)

Request timeout in milliseconds
