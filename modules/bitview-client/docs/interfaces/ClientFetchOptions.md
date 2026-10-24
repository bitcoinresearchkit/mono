[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / ClientFetchOptions

# Interface: ClientFetchOptions\<T\>

Defined in: [modules/bitview-client/index.js:1877](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1877)

## Type Parameters

### T

`T`

## Properties

### cache?

> `optional` **cache?**: `boolean`

Defined in: [modules/bitview-client/index.js:1879](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1879)

Use HTTP/browser/client caches. Set false for a no-store network fetch.

***

### memCache?

> `optional` **memCache?**: `boolean`

Defined in: [modules/bitview-client/index.js:1880](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1880)

Use the parsed in-memory response cache. Set false for large one-shot reads.

***

### onValue?

> `optional` **onValue?**: (`value`) => `void`

Defined in: [modules/bitview-client/index.js:1881](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1881)

Receive stale/fresh values as they arrive

#### Parameters

##### value

`T`

#### Returns

`void`

***

### signal?

> `optional` **signal?**: `AbortSignal`

Defined in: [modules/bitview-client/index.js:1878](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1878)

Abort this request
