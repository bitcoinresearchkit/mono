[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / DateSeriesEndpoint

# Interface: DateSeriesEndpoint\<T\>

Defined in: [modules/bitview-client/index.js:1968](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1968)

## Type Parameters

### T

`T`

## Properties

### fetch

> **fetch**: (`arg?`, `options?`) => `Promise`\<[`DateSeriesData`](../type-aliases/DateSeriesData.md)\<`T`\>\>

Defined in: [modules/bitview-client/index.js:1974](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1974)

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

Defined in: [modules/bitview-client/index.js:1975](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1975)

Fetch all data as CSV

#### Parameters

##### options?

[`ClientFetchOptions`](ClientFetchOptions.md)\<`string`\>

#### Returns

`Promise`\<`string`\>

***

### first

> **first**: (`n`) => [`DateRangeBuilder`](DateRangeBuilder.md)\<`T`\>

Defined in: [modules/bitview-client/index.js:1971](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1971)

#### Parameters

##### n

`number`

#### Returns

[`DateRangeBuilder`](DateRangeBuilder.md)\<`T`\>

***

### get

> **get**: (`index`) => [`DateRangeBuilder`](DateRangeBuilder.md)\<`T`\>

Defined in: [modules/bitview-client/index.js:1969](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1969)

#### Parameters

##### index

`number` \| `Date`

#### Returns

[`DateRangeBuilder`](DateRangeBuilder.md)\<`T`\>

***

### last

> **last**: (`n`) => [`DateRangeBuilder`](DateRangeBuilder.md)\<`T`\>

Defined in: [modules/bitview-client/index.js:1972](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1972)

#### Parameters

##### n

`number`

#### Returns

[`DateRangeBuilder`](DateRangeBuilder.md)\<`T`\>

***

### len

> **len**: () => `Promise`\<`number`\>

Defined in: [modules/bitview-client/index.js:1976](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1976)

Get total number of data points

#### Returns

`Promise`\<`number`\>

***

### path

> **path**: `string`

Defined in: [modules/bitview-client/index.js:1979](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1979)

***

### skip

> **skip**: (`n`) => [`DateSkippedBuilder`](DateSkippedBuilder.md)\<`T`\>

Defined in: [modules/bitview-client/index.js:1973](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1973)

Skip first n items, chain with take()

#### Parameters

##### n

`number`

#### Returns

[`DateSkippedBuilder`](DateSkippedBuilder.md)\<`T`\>

***

### slice

> **slice**: (`start?`, `end?`) => [`DateRangeBuilder`](DateRangeBuilder.md)\<`T`\>

Defined in: [modules/bitview-client/index.js:1970](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1970)

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

Defined in: [modules/bitview-client/index.js:1978](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1978)

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

Defined in: [modules/bitview-client/index.js:1977](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1977)

#### Returns

`Promise`\<`number`\>
