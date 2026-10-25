[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / ClientFetchOptions

# Interface: ClientFetchOptions\<T\>

Defined in: [modules/bitview-client/index.js:2026](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L2026)

## Type Parameters

### T

`T`

## Properties

### cache?

> `optional` **cache?**: `boolean`

Defined in: [modules/bitview-client/index.js:2028](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L2028)

Use HTTP/browser/client caches. Set false for a no-store network fetch.

***

### memCache?

> `optional` **memCache?**: `boolean`

Defined in: [modules/bitview-client/index.js:2029](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L2029)

Use the parsed in-memory response cache. Set false for large one-shot reads.

***

### onValue?

> `optional` **onValue?**: (`value`) => `void`

Defined in: [modules/bitview-client/index.js:2030](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L2030)

Receive stale/fresh values as they arrive

#### Parameters

##### value

`T`

#### Returns

`void`

***

### signal?

> `optional` **signal?**: `AbortSignal`

Defined in: [modules/bitview-client/index.js:2027](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L2027)

Abort this request
