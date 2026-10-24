[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / ClientFetchOptions

# Interface: ClientFetchOptions\<T\>

Defined in: [modules/bitview-client/index.js:1954](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1954)

## Type Parameters

### T

`T`

## Properties

### cache?

> `optional` **cache?**: `boolean`

Defined in: [modules/bitview-client/index.js:1956](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1956)

Use HTTP/browser/client caches. Set false for a no-store network fetch.

***

### memCache?

> `optional` **memCache?**: `boolean`

Defined in: [modules/bitview-client/index.js:1957](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1957)

Use the parsed in-memory response cache. Set false for large one-shot reads.

***

### onValue?

> `optional` **onValue?**: (`value`) => `void`

Defined in: [modules/bitview-client/index.js:1958](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1958)

Receive stale/fresh values as they arrive

#### Parameters

##### value

`T`

#### Returns

`void`

***

### signal?

> `optional` **signal?**: `AbortSignal`

Defined in: [modules/bitview-client/index.js:1955](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1955)

Abort this request
