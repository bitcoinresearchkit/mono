[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / SeriesEndpoint

# Interface: SeriesEndpoint\<T\>

Defined in: [modules/bitview-client/index.js:1867](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1867)

## Type Parameters

### T

`T`

## Properties

### fetch

> **fetch**: (`arg?`, `options?`) => `Promise`\<[`SeriesData`](../type-aliases/SeriesData.md)\<`T`\>\>

Defined in: [modules/bitview-client/index.js:1873](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1873)

Fetch all data

#### Parameters

##### arg?

[`SeriesFetchArg`](../type-aliases/SeriesFetchArg.md)\<`T`\>

##### options?

[`ClientFetchOptions`](ClientFetchOptions.md)\<[`SeriesData`](../type-aliases/SeriesData.md)\<`T`\>\>

#### Returns

`Promise`\<[`SeriesData`](../type-aliases/SeriesData.md)\<`T`\>\>

***

### fetchCsv

> **fetchCsv**: (`options?`) => `Promise`\<`string`\>

Defined in: [modules/bitview-client/index.js:1874](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1874)

Fetch all data as CSV

#### Parameters

##### options?

[`ClientFetchOptions`](ClientFetchOptions.md)\<`string`\>

#### Returns

`Promise`\<`string`\>

***

### first

> **first**: (`n`) => [`RangeBuilder`](RangeBuilder.md)\<`T`\>

Defined in: [modules/bitview-client/index.js:1870](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1870)

#### Parameters

##### n

`number`

#### Returns

[`RangeBuilder`](RangeBuilder.md)\<`T`\>

***

### get

> **get**: (`index`) => [`RangeBuilder`](RangeBuilder.md)\<`T`\>

Defined in: [modules/bitview-client/index.js:1868](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1868)

#### Parameters

##### index

`number`

#### Returns

[`RangeBuilder`](RangeBuilder.md)\<`T`\>

***

### last

> **last**: (`n`) => [`RangeBuilder`](RangeBuilder.md)\<`T`\>

Defined in: [modules/bitview-client/index.js:1871](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1871)

#### Parameters

##### n

`number`

#### Returns

[`RangeBuilder`](RangeBuilder.md)\<`T`\>

***

### len

> **len**: () => `Promise`\<`number`\>

Defined in: [modules/bitview-client/index.js:1875](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1875)

Get total number of data points

#### Returns

`Promise`\<`number`\>

***

### path

> **path**: `string`

Defined in: [modules/bitview-client/index.js:1878](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1878)

***

### skip

> **skip**: (`n`) => [`SkippedBuilder`](SkippedBuilder.md)\<`T`\>

Defined in: [modules/bitview-client/index.js:1872](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1872)

Skip first n items, chain with take()

#### Parameters

##### n

`number`

#### Returns

[`SkippedBuilder`](SkippedBuilder.md)\<`T`\>

***

### slice

> **slice**: (`start?`, `end?`) => [`RangeBuilder`](RangeBuilder.md)\<`T`\>

Defined in: [modules/bitview-client/index.js:1869](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1869)

#### Parameters

##### start?

`number`

##### end?

`number`

#### Returns

[`RangeBuilder`](RangeBuilder.md)\<`T`\>

***

### then

> **then**: \<`TResult1`, `TResult2`\>(`onfulfilled?`, `onrejected?`) => `PromiseLike`\<`TResult1` \| `TResult2`\>

Defined in: [modules/bitview-client/index.js:1877](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1877)

Thenable (await endpoint)

Attaches callbacks for the resolution and/or rejection of the Promise.

#### Type Parameters

##### TResult1

`TResult1` = [`SeriesData`](../type-aliases/SeriesData.md)\<`T`\>

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

Defined in: [modules/bitview-client/index.js:1876](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1876)

#### Returns

`Promise`\<`number`\>
