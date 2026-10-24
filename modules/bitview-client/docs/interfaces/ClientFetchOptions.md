[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / ClientFetchOptions

# Interface: ClientFetchOptions\<T\>

Defined in: [modules/bitview-client/index.js:1930](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1930)

## Type Parameters

### T

`T`

## Properties

### cache?

> `optional` **cache?**: `boolean`

Defined in: [modules/bitview-client/index.js:1932](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1932)

Use HTTP/browser/client caches. Set false for a no-store network fetch.

***

### memCache?

> `optional` **memCache?**: `boolean`

Defined in: [modules/bitview-client/index.js:1933](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1933)

Use the parsed in-memory response cache. Set false for large one-shot reads.

***

### onValue?

> `optional` **onValue?**: (`value`) => `void`

Defined in: [modules/bitview-client/index.js:1934](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1934)

Receive stale/fresh values as they arrive

#### Parameters

##### value

`T`

#### Returns

`void`

***

### signal?

> `optional` **signal?**: `AbortSignal`

Defined in: [modules/bitview-client/index.js:1931](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1931)

Abort this request
