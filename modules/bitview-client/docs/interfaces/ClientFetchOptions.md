[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / ClientFetchOptions

# Interface: ClientFetchOptions\<T\>

Defined in: [modules/bitview-client/index.js:1810](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1810)

## Type Parameters

### T

`T`

## Properties

### cache?

> `optional` **cache?**: `boolean`

Defined in: [modules/bitview-client/index.js:1812](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1812)

Use HTTP/browser/client caches. Set false for a no-store network fetch.

***

### memCache?

> `optional` **memCache?**: `boolean`

Defined in: [modules/bitview-client/index.js:1813](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1813)

Use the parsed in-memory response cache. Set false for large one-shot reads.

***

### onValue?

> `optional` **onValue?**: (`value`) => `void`

Defined in: [modules/bitview-client/index.js:1814](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1814)

Receive stale/fresh values as they arrive

#### Parameters

##### value

`T`

#### Returns

`void`

***

### signal?

> `optional` **signal?**: `AbortSignal`

Defined in: [modules/bitview-client/index.js:1811](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1811)

Abort this request
