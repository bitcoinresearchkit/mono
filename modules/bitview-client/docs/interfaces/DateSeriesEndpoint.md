[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / DateSeriesEndpoint

# Interface: DateSeriesEndpoint\<T\>

Defined in: [modules/bitview-client/index.js:1882](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1882)

## Type Parameters

### T

`T`

## Properties

### fetch

> **fetch**: (`arg?`, `options?`) => `Promise`\<[`DateSeriesData`](../type-aliases/DateSeriesData.md)\<`T`\>\>

Defined in: [modules/bitview-client/index.js:1888](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1888)

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

Defined in: [modules/bitview-client/index.js:1889](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1889)

Fetch all data as CSV

#### Parameters

##### options?

[`ClientFetchOptions`](ClientFetchOptions.md)\<`string`\>

#### Returns

`Promise`\<`string`\>

***

### first

> **first**: (`n`) => [`DateRangeBuilder`](DateRangeBuilder.md)\<`T`\>

Defined in: [modules/bitview-client/index.js:1885](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1885)

#### Parameters

##### n

`number`

#### Returns

[`DateRangeBuilder`](DateRangeBuilder.md)\<`T`\>

***

### get

> **get**: (`index`) => [`DateRangeBuilder`](DateRangeBuilder.md)\<`T`\>

Defined in: [modules/bitview-client/index.js:1883](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1883)

#### Parameters

##### index

`number` \| `Date`

#### Returns

[`DateRangeBuilder`](DateRangeBuilder.md)\<`T`\>

***

### last

> **last**: (`n`) => [`DateRangeBuilder`](DateRangeBuilder.md)\<`T`\>

Defined in: [modules/bitview-client/index.js:1886](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1886)

#### Parameters

##### n

`number`

#### Returns

[`DateRangeBuilder`](DateRangeBuilder.md)\<`T`\>

***

### len

> **len**: () => `Promise`\<`number`\>

Defined in: [modules/bitview-client/index.js:1890](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1890)

Get total number of data points

#### Returns

`Promise`\<`number`\>

***

### path

> **path**: `string`

Defined in: [modules/bitview-client/index.js:1893](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1893)

***

### skip

> **skip**: (`n`) => [`DateSkippedBuilder`](DateSkippedBuilder.md)\<`T`\>

Defined in: [modules/bitview-client/index.js:1887](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1887)

Skip first n items, chain with take()

#### Parameters

##### n

`number`

#### Returns

[`DateSkippedBuilder`](DateSkippedBuilder.md)\<`T`\>

***

### slice

> **slice**: (`start?`, `end?`) => [`DateRangeBuilder`](DateRangeBuilder.md)\<`T`\>

Defined in: [modules/bitview-client/index.js:1884](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1884)

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

Defined in: [modules/bitview-client/index.js:1892](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1892)

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

Defined in: [modules/bitview-client/index.js:1891](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1891)

#### Returns

`Promise`\<`number`\>
