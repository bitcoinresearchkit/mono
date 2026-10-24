[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / SeriesEndpoint

# Interface: SeriesEndpoint\<T\>

Defined in: [modules/bitview-client/index.js:1891](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1891)

## Type Parameters

### T

`T`

## Properties

### fetch

> **fetch**: (`arg?`, `options?`) => `Promise`\<[`SeriesData`](../type-aliases/SeriesData.md)\<`T`\>\>

Defined in: [modules/bitview-client/index.js:1897](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1897)

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

Defined in: [modules/bitview-client/index.js:1898](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1898)

Fetch all data as CSV

#### Parameters

##### options?

[`ClientFetchOptions`](ClientFetchOptions.md)\<`string`\>

#### Returns

`Promise`\<`string`\>

***

### first

> **first**: (`n`) => [`RangeBuilder`](RangeBuilder.md)\<`T`\>

Defined in: [modules/bitview-client/index.js:1894](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1894)

#### Parameters

##### n

`number`

#### Returns

[`RangeBuilder`](RangeBuilder.md)\<`T`\>

***

### get

> **get**: (`index`) => [`RangeBuilder`](RangeBuilder.md)\<`T`\>

Defined in: [modules/bitview-client/index.js:1892](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1892)

#### Parameters

##### index

`number`

#### Returns

[`RangeBuilder`](RangeBuilder.md)\<`T`\>

***

### last

> **last**: (`n`) => [`RangeBuilder`](RangeBuilder.md)\<`T`\>

Defined in: [modules/bitview-client/index.js:1895](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1895)

#### Parameters

##### n

`number`

#### Returns

[`RangeBuilder`](RangeBuilder.md)\<`T`\>

***

### len

> **len**: () => `Promise`\<`number`\>

Defined in: [modules/bitview-client/index.js:1899](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1899)

Get total number of data points

#### Returns

`Promise`\<`number`\>

***

### path

> **path**: `string`

Defined in: [modules/bitview-client/index.js:1902](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1902)

***

### skip

> **skip**: (`n`) => [`SkippedBuilder`](SkippedBuilder.md)\<`T`\>

Defined in: [modules/bitview-client/index.js:1896](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1896)

Skip first n items, chain with take()

#### Parameters

##### n

`number`

#### Returns

[`SkippedBuilder`](SkippedBuilder.md)\<`T`\>

***

### slice

> **slice**: (`start?`, `end?`) => [`RangeBuilder`](RangeBuilder.md)\<`T`\>

Defined in: [modules/bitview-client/index.js:1893](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1893)

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

Defined in: [modules/bitview-client/index.js:1901](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1901)

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

Defined in: [modules/bitview-client/index.js:1900](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1900)

#### Returns

`Promise`\<`number`\>
