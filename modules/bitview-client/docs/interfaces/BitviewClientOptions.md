[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / BitviewClientOptions

# Interface: BitviewClientOptions

Defined in: modules/bitview-client/index.js:1766

## Properties

### baseUrl

> **baseUrl**: `string`

Defined in: modules/bitview-client/index.js:1767

***

### browserCache?

> `optional` **browserCache?**: `string` \| `boolean`

Defined in: modules/bitview-client/index.js:1769

Enable browser Cache API with default name (true), custom name (string), or disable (false). No effect in Node.js. Default: true

***

### memCache?

> `optional` **memCache?**: `number` \| `boolean`

Defined in: modules/bitview-client/index.js:1770

LRU capacity of the in-memory parsed-response cache, which lets 304 responses skip the JSON parse: true/undefined → 1000, false/0 → disabled

***

### timeout?

> `optional` **timeout?**: `number`

Defined in: modules/bitview-client/index.js:1768

Request timeout in milliseconds, the download included. Default: 30000
