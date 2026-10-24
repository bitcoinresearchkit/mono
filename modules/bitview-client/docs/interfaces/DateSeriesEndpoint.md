[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / DateSeriesEndpoint

# Interface: DateSeriesEndpoint\<T\>

Defined in: [modules/bitview-client/index.js:1859](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1859)

## Type Parameters

### T

`T`

## Properties

### fetch

> **fetch**: (`arg?`, `options?`) => `Promise`\<[`DateSeriesData`](../type-aliases/DateSeriesData.md)\<`T`\>\>

Defined in: [modules/bitview-client/index.js:1865](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1865)

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

Defined in: [modules/bitview-client/index.js:1866](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1866)

Fetch all data as CSV

#### Parameters

##### options?

[`ClientFetchOptions`](ClientFetchOptions.md)\<`string`\>

#### Returns

`Promise`\<`string`\>

***

### first

> **first**: (`n`) => [`DateRangeBuilder`](DateRangeBuilder.md)\<`T`\>

Defined in: [modules/bitview-client/index.js:1862](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1862)

#### Parameters

##### n

`number`

#### Returns

[`DateRangeBuilder`](DateRangeBuilder.md)\<`T`\>

***

### get

> **get**: (`index`) => [`DateRangeBuilder`](DateRangeBuilder.md)\<`T`\>

Defined in: [modules/bitview-client/index.js:1860](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1860)

#### Parameters

##### index

`number` \| `Date`

#### Returns

[`DateRangeBuilder`](DateRangeBuilder.md)\<`T`\>

***

### last

> **last**: (`n`) => [`DateRangeBuilder`](DateRangeBuilder.md)\<`T`\>

Defined in: [modules/bitview-client/index.js:1863](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1863)

#### Parameters

##### n

`number`

#### Returns

[`DateRangeBuilder`](DateRangeBuilder.md)\<`T`\>

***

### len

> **len**: () => `Promise`\<`number`\>

Defined in: [modules/bitview-client/index.js:1867](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1867)

Get total number of data points

#### Returns

`Promise`\<`number`\>

***

### path

> **path**: `string`

Defined in: [modules/bitview-client/index.js:1870](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1870)

***

### skip

> **skip**: (`n`) => [`DateSkippedBuilder`](DateSkippedBuilder.md)\<`T`\>

Defined in: [modules/bitview-client/index.js:1864](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1864)

Skip first n items, chain with take()

#### Parameters

##### n

`number`

#### Returns

[`DateSkippedBuilder`](DateSkippedBuilder.md)\<`T`\>

***

### slice

> **slice**: (`start?`, `end?`) => [`DateRangeBuilder`](DateRangeBuilder.md)\<`T`\>

Defined in: [modules/bitview-client/index.js:1861](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1861)

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

Defined in: [modules/bitview-client/index.js:1869](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1869)

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

Defined in: [modules/bitview-client/index.js:1868](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1868)

#### Returns

`Promise`\<`number`\>
