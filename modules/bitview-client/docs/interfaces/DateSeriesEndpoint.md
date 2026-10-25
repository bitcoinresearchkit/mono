[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / DateSeriesEndpoint

# Interface: DateSeriesEndpoint\<T\>

Defined in: [modules/bitview-client/index.js:2008](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L2008)

## Type Parameters

### T

`T`

## Properties

### fetch

> **fetch**: (`arg?`, `options?`) => `Promise`\<[`DateSeriesData`](../type-aliases/DateSeriesData.md)\<`T`\>\>

Defined in: [modules/bitview-client/index.js:2014](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L2014)

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

Defined in: [modules/bitview-client/index.js:2015](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L2015)

Fetch all data as CSV

#### Parameters

##### options?

[`ClientFetchOptions`](ClientFetchOptions.md)\<`string`\>

#### Returns

`Promise`\<`string`\>

***

### first

> **first**: (`n`) => [`DateRangeBuilder`](DateRangeBuilder.md)\<`T`\>

Defined in: [modules/bitview-client/index.js:2011](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L2011)

#### Parameters

##### n

`number`

#### Returns

[`DateRangeBuilder`](DateRangeBuilder.md)\<`T`\>

***

### get

> **get**: (`index`) => [`DateRangeBuilder`](DateRangeBuilder.md)\<`T`\>

Defined in: [modules/bitview-client/index.js:2009](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L2009)

#### Parameters

##### index

`number` \| `Date`

#### Returns

[`DateRangeBuilder`](DateRangeBuilder.md)\<`T`\>

***

### last

> **last**: (`n`) => [`DateRangeBuilder`](DateRangeBuilder.md)\<`T`\>

Defined in: [modules/bitview-client/index.js:2012](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L2012)

#### Parameters

##### n

`number`

#### Returns

[`DateRangeBuilder`](DateRangeBuilder.md)\<`T`\>

***

### len

> **len**: () => `Promise`\<`number`\>

Defined in: [modules/bitview-client/index.js:2016](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L2016)

Get total number of data points

#### Returns

`Promise`\<`number`\>

***

### path

> **path**: `string`

Defined in: [modules/bitview-client/index.js:2019](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L2019)

***

### skip

> **skip**: (`n`) => [`DateSkippedBuilder`](DateSkippedBuilder.md)\<`T`\>

Defined in: [modules/bitview-client/index.js:2013](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L2013)

Skip first n items, chain with take()

#### Parameters

##### n

`number`

#### Returns

[`DateSkippedBuilder`](DateSkippedBuilder.md)\<`T`\>

***

### slice

> **slice**: (`start?`, `end?`) => [`DateRangeBuilder`](DateRangeBuilder.md)\<`T`\>

Defined in: [modules/bitview-client/index.js:2010](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L2010)

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

Defined in: [modules/bitview-client/index.js:2018](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L2018)

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

Defined in: [modules/bitview-client/index.js:2017](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L2017)

#### Returns

`Promise`\<`number`\>
