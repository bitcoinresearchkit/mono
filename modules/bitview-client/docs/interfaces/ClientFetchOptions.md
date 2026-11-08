[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / ClientFetchOptions

# Interface: ClientFetchOptions\<T\>

Defined in: [modules/bitview-client/index.js:1986](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1986)

## Type Parameters

### T

`T`

## Properties

### cache?

> `optional` **cache?**: `boolean`

Defined in: [modules/bitview-client/index.js:1988](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1988)

Use HTTP/browser/client caches. Set false for a no-store network fetch.

***

### memCache?

> `optional` **memCache?**: `boolean`

Defined in: [modules/bitview-client/index.js:1989](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1989)

Use the parsed in-memory response cache, whose values are shared and frozen deeply (bytes can't be frozen: don't change them). Set false for large one-shot reads, or a value of your own to change.

***

### onValue?

> `optional` **onValue?**: (`value`) => `void`

Defined in: [modules/bitview-client/index.js:1990](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1990)

Receive stale/fresh values as they arrive

#### Parameters

##### value

`T`

#### Returns

`void`

***

### signal?

> `optional` **signal?**: `AbortSignal`

Defined in: [modules/bitview-client/index.js:1987](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1987)

Abort this request
