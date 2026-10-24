[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / ClientFetchOptions

# Interface: ClientFetchOptions\<T\>

Defined in: [modules/bitview-client/index.js:1925](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1925)

## Type Parameters

### T

`T`

## Properties

### cache?

> `optional` **cache?**: `boolean`

Defined in: [modules/bitview-client/index.js:1927](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1927)

Use HTTP/browser/client caches. Set false for a no-store network fetch.

***

### memCache?

> `optional` **memCache?**: `boolean`

Defined in: [modules/bitview-client/index.js:1928](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1928)

Use the parsed in-memory response cache. Set false for large one-shot reads.

***

### onValue?

> `optional` **onValue?**: (`value`) => `void`

Defined in: [modules/bitview-client/index.js:1929](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1929)

Receive stale/fresh values as they arrive

#### Parameters

##### value

`T`

#### Returns

`void`

***

### signal?

> `optional` **signal?**: `AbortSignal`

Defined in: [modules/bitview-client/index.js:1926](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1926)

Abort this request
