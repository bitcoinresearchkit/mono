[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / ClientFetchOptions

# Interface: ClientFetchOptions\<T\>

Defined in: [modules/bitview-client/index.js:1912](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1912)

## Type Parameters

### T

`T`

## Properties

### cache?

> `optional` **cache?**: `boolean`

Defined in: [modules/bitview-client/index.js:1914](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1914)

Use HTTP/browser/client caches. Set false for a no-store network fetch.

***

### memCache?

> `optional` **memCache?**: `boolean`

Defined in: [modules/bitview-client/index.js:1915](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1915)

Use the parsed in-memory response cache. Set false for large one-shot reads.

***

### onValue?

> `optional` **onValue?**: (`value`) => `void`

Defined in: [modules/bitview-client/index.js:1916](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1916)

Receive stale/fresh values as they arrive

#### Parameters

##### value

`T`

#### Returns

`void`

***

### signal?

> `optional` **signal?**: `AbortSignal`

Defined in: [modules/bitview-client/index.js:1913](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1913)

Abort this request
