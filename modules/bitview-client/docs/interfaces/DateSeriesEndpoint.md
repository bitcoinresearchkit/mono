[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / DateSeriesEndpoint

# Interface: DateSeriesEndpoint\<T\>

Defined in: [modules/bitview-client/index.js:1792](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1792)

## Type Parameters

### T

`T`

## Properties

### fetch

> **fetch**: (`arg?`, `options?`) => `Promise`\<[`DateSeriesData`](../type-aliases/DateSeriesData.md)\<`T`\>\>

Defined in: [modules/bitview-client/index.js:1798](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1798)

Fetch all data

#### Parameters

##### arg?

[`DateSeriesFetchArg`](../type-aliases/DateSeriesFetchArg.md)\<`T`\>

##### options?

[`ClientFetchOptions`](ClientFetchOptions.md)\<[`DateSeriesData`](../type-aliases/DateSeriesData.md)\<`T`\>\>

#### Returns

`Promise`\<[`DateSeriesData`](../type-aliases/DateSeriesData.md)\<`T`\>\>

***

### fetchCsv

> **fetchCsv**: (`options?`) => `Promise`\<`string`\>

Defined in: [modules/bitview-client/index.js:1799](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1799)

Fetch all data as CSV

#### Parameters

##### options?

[`ClientFetchOptions`](ClientFetchOptions.md)\<`string`\>

#### Returns

`Promise`\<`string`\>

***

### first

> **first**: (`n`) => [`DateRangeBuilder`](DateRangeBuilder.md)\<`T`\>

Defined in: [modules/bitview-client/index.js:1795](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1795)

#### Parameters

##### n

`number`

#### Returns

[`DateRangeBuilder`](DateRangeBuilder.md)\<`T`\>

***

### get

> **get**: (`index`) => [`DateRangeBuilder`](DateRangeBuilder.md)\<`T`\>

Defined in: [modules/bitview-client/index.js:1793](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1793)

#### Parameters

##### index

`number` \| `Date`

#### Returns

[`DateRangeBuilder`](DateRangeBuilder.md)\<`T`\>

***

### last

> **last**: (`n`) => [`DateRangeBuilder`](DateRangeBuilder.md)\<`T`\>

Defined in: [modules/bitview-client/index.js:1796](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1796)

#### Parameters

##### n

`number`

#### Returns

[`DateRangeBuilder`](DateRangeBuilder.md)\<`T`\>

***

### len

> **len**: () => `Promise`\<`number`\>

Defined in: [modules/bitview-client/index.js:1800](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1800)

Get total number of data points

#### Returns

`Promise`\<`number`\>

***

### path

> **path**: `string`

Defined in: [modules/bitview-client/index.js:1803](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1803)

***

### skip

> **skip**: (`n`) => [`DateSkippedBuilder`](DateSkippedBuilder.md)\<`T`\>

Defined in: [modules/bitview-client/index.js:1797](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1797)

Skip first n items, chain with take()

#### Parameters

##### n

`number`

#### Returns

[`DateSkippedBuilder`](DateSkippedBuilder.md)\<`T`\>

***

### slice

> **slice**: (`start?`, `end?`) => [`DateRangeBuilder`](DateRangeBuilder.md)\<`T`\>

Defined in: [modules/bitview-client/index.js:1794](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1794)

#### Parameters

##### start?

`number` \| `Date`

##### end?

`number` \| `Date`

#### Returns

[`DateRangeBuilder`](DateRangeBuilder.md)\<`T`\>

***

### then

> **then**: \<`TResult1`, `TResult2`\>(`onfulfilled?`, `onrejected?`) => `PromiseLike`\<`TResult1` \| `TResult2`\>

Defined in: [modules/bitview-client/index.js:1802](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1802)

Thenable (await endpoint)

Attaches callbacks for the resolution and/or rejection of the Promise.

#### Type Parameters

##### TResult1

`TResult1` = [`DateSeriesData`](../type-aliases/DateSeriesData.md)\<`T`\>

##### TResult2

`TResult2` = `never`

#### Parameters

##### onfulfilled?

((`value`) => `TResult1` \| `PromiseLike`\<`TResult1`\>) \| `null`

The callback to execute when the Promise is resolved.

##### onrejected?

((`reason`) => `TResult2` \| `PromiseLike`\<`TResult2`\>) \| `null`

The callback to execute when the Promise is rejected.

#### Returns

`PromiseLike`\<`TResult1` \| `TResult2`\>

A Promise for the completion of which ever callback is executed.

***

### version

> **version**: () => `Promise`\<`number`\>

Defined in: [modules/bitview-client/index.js:1801](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1801)

#### Returns

`Promise`\<`number`\>
