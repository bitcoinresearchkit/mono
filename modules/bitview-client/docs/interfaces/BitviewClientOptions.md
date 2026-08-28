[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / BitviewClientOptions

# Interface: BitviewClientOptions

Defined in: [Developer/brk/modules/bitview-client/index.js:1628](https://github.com/bitcoinresearchkit/brk/blob/a6ece0db382228669ac4d1a9c673190fe7a4897b/modules/bitview-client/index.js#L1628)

## Properties

### baseUrl

> **baseUrl**: `string`

Defined in: [Developer/brk/modules/bitview-client/index.js:1629](https://github.com/bitcoinresearchkit/brk/blob/a6ece0db382228669ac4d1a9c673190fe7a4897b/modules/bitview-client/index.js#L1629)

Base URL for the API

***

### browserCache?

> `optional` **browserCache?**: `string` \| `boolean`

Defined in: [Developer/brk/modules/bitview-client/index.js:1631](https://github.com/bitcoinresearchkit/brk/blob/a6ece0db382228669ac4d1a9c673190fe7a4897b/modules/bitview-client/index.js#L1631)

Enable browser Cache API with default name (true), custom name (string), or disable (false). No effect in Node.js. Default: true

***

### memCache?

> `optional` **memCache?**: `number` \| `boolean`

Defined in: [Developer/brk/modules/bitview-client/index.js:1632](https://github.com/bitcoinresearchkit/brk/blob/a6ece0db382228669ac4d1a9c673190fe7a4897b/modules/bitview-client/index.js#L1632)

In-memory parsed-response cache size (LRU). true/undefined → 1000, false/0 → disabled. Lets 304 responses skip the JSON parse entirely. Default: 1000

***

### timeout?

> `optional` **timeout?**: `number`

Defined in: [Developer/brk/modules/bitview-client/index.js:1630](https://github.com/bitcoinresearchkit/brk/blob/a6ece0db382228669ac4d1a9c673190fe7a4897b/modules/bitview-client/index.js#L1630)

Request timeout in milliseconds
