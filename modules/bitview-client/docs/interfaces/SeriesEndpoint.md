[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / SeriesEndpoint

# Interface: SeriesEndpoint\<T\>

Defined in: [modules/bitview-client/index.js:1878](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1878)

## Type Parameters

### T

`T`

## Properties

### fetch

> **fetch**: (`arg?`, `options?`) => `Promise`\<[`SeriesData`](../type-aliases/SeriesData.md)\<`T`\>\>

Defined in: [modules/bitview-client/index.js:1884](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1884)

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

Defined in: [modules/bitview-client/index.js:1885](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1885)

Fetch all data as CSV

#### Parameters

##### options?

[`ClientFetchOptions`](ClientFetchOptions.md)\<`string`\>

#### Returns

`Promise`\<`string`\>

***

### first

> **first**: (`n`) => [`RangeBuilder`](RangeBuilder.md)\<`T`\>

Defined in: [modules/bitview-client/index.js:1881](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1881)

#### Parameters

##### n

`number`

#### Returns

[`RangeBuilder`](RangeBuilder.md)\<`T`\>

***

### get

> **get**: (`index`) => [`RangeBuilder`](RangeBuilder.md)\<`T`\>

Defined in: [modules/bitview-client/index.js:1879](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1879)

#### Parameters

##### index

`number`

#### Returns

[`RangeBuilder`](RangeBuilder.md)\<`T`\>

***

### last

> **last**: (`n`) => [`RangeBuilder`](RangeBuilder.md)\<`T`\>

Defined in: [modules/bitview-client/index.js:1882](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1882)

#### Parameters

##### n

`number`

#### Returns

[`RangeBuilder`](RangeBuilder.md)\<`T`\>

***

### len

> **len**: () => `Promise`\<`number`\>

Defined in: [modules/bitview-client/index.js:1886](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1886)

Get total number of data points

#### Returns

`Promise`\<`number`\>

***

### path

> **path**: `string`

Defined in: [modules/bitview-client/index.js:1889](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1889)

***

### skip

> **skip**: (`n`) => [`SkippedBuilder`](SkippedBuilder.md)\<`T`\>

Defined in: [modules/bitview-client/index.js:1883](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1883)

Skip first n items, chain with take()

#### Parameters

##### n

`number`

#### Returns

[`SkippedBuilder`](SkippedBuilder.md)\<`T`\>

***

### slice

> **slice**: (`start?`, `end?`) => [`RangeBuilder`](RangeBuilder.md)\<`T`\>

Defined in: [modules/bitview-client/index.js:1880](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1880)

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

Defined in: [modules/bitview-client/index.js:1888](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1888)

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

Defined in: [modules/bitview-client/index.js:1887](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1887)

#### Returns

`Promise`\<`number`\>
