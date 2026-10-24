[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / ClientFetchOptions

# Interface: ClientFetchOptions\<T\>

Defined in: [modules/bitview-client/index.js:1901](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1901)

## Type Parameters

### T

`T`

## Properties

### cache?

> `optional` **cache?**: `boolean`

Defined in: [modules/bitview-client/index.js:1903](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1903)

Use HTTP/browser/client caches. Set false for a no-store network fetch.

***

### memCache?

> `optional` **memCache?**: `boolean`

Defined in: [modules/bitview-client/index.js:1904](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1904)

Use the parsed in-memory response cache. Set false for large one-shot reads.

***

### onValue?

> `optional` **onValue?**: (`value`) => `void`

Defined in: [modules/bitview-client/index.js:1905](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1905)

Receive stale/fresh values as they arrive

#### Parameters

##### value

`T`

#### Returns

`void`

***

### signal?

> `optional` **signal?**: `AbortSignal`

Defined in: [modules/bitview-client/index.js:1902](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1902)

Abort this request
