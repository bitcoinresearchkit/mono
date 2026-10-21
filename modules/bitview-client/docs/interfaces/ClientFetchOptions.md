[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / ClientFetchOptions

# Interface: ClientFetchOptions\<T\>

Defined in: [modules/bitview-client/index.js:1800](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1800)

## Type Parameters

### T

`T`

## Properties

### cache?

> `optional` **cache?**: `boolean`

Defined in: [modules/bitview-client/index.js:1802](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1802)

Use HTTP/browser/client caches. Set false for a no-store network fetch.

***

### memCache?

> `optional` **memCache?**: `boolean`

Defined in: [modules/bitview-client/index.js:1803](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1803)

Use the parsed in-memory response cache. Set false for large one-shot reads.

***

### onValue?

> `optional` **onValue?**: (`value`) => `void`

Defined in: [modules/bitview-client/index.js:1804](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1804)

Receive stale/fresh values as they arrive

#### Parameters

##### value

`T`

#### Returns

`void`

***

### signal?

> `optional` **signal?**: `AbortSignal`

Defined in: [modules/bitview-client/index.js:1801](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1801)

Abort this request
