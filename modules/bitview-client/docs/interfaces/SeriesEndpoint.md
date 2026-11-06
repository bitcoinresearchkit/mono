[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / SeriesEndpoint

# Interface: SeriesEndpoint\<T\>

Defined in: [modules/bitview-client/index.js:2050](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L2050)

## Type Parameters

### T

`T`

## Properties

### fetch

> **fetch**: (`arg?`, `options?`) => `Promise`\<[`SeriesData`](../type-aliases/SeriesData.md)\<`T`\>\>

Defined in: [modules/bitview-client/index.js:2056](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L2056)

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

Defined in: [modules/bitview-client/index.js:2057](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L2057)

Fetch all data as CSV

#### Parameters

##### options?

[`ClientFetchOptions`](ClientFetchOptions.md)\<`string`\>

#### Returns

`Promise`\<`string`\>

***

### first

> **first**: (`n`) => [`RangeBuilder`](RangeBuilder.md)\<`T`\>

Defined in: [modules/bitview-client/index.js:2053](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L2053)

#### Parameters

##### n

`number`

#### Returns

[`RangeBuilder`](RangeBuilder.md)\<`T`\>

***

### get

> **get**: (`index`) => [`RangeBuilder`](RangeBuilder.md)\<`T`\>

Defined in: [modules/bitview-client/index.js:2051](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L2051)

#### Parameters

##### index

`number`

#### Returns

[`RangeBuilder`](RangeBuilder.md)\<`T`\>

***

### last

> **last**: (`n`) => [`RangeBuilder`](RangeBuilder.md)\<`T`\>

Defined in: [modules/bitview-client/index.js:2054](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L2054)

#### Parameters

##### n

`number`

#### Returns

[`RangeBuilder`](RangeBuilder.md)\<`T`\>

***

### len

> **len**: () => `Promise`\<`number`\>

Defined in: [modules/bitview-client/index.js:2058](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L2058)

Get total number of data points

#### Returns

`Promise`\<`number`\>

***

### path

> **path**: `string`

Defined in: [modules/bitview-client/index.js:2061](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L2061)

***

### skip

> **skip**: (`n`) => [`SkippedBuilder`](SkippedBuilder.md)\<`T`\>

Defined in: [modules/bitview-client/index.js:2055](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L2055)

Skip first n items, chain with take()

#### Parameters

##### n

`number`

#### Returns

[`SkippedBuilder`](SkippedBuilder.md)\<`T`\>

***

### slice

> **slice**: (`start?`, `end?`) => [`RangeBuilder`](RangeBuilder.md)\<`T`\>

Defined in: [modules/bitview-client/index.js:2052](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L2052)

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

Defined in: [modules/bitview-client/index.js:2060](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L2060)

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

Defined in: [modules/bitview-client/index.js:2059](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L2059)

#### Returns

`Promise`\<`number`\>
