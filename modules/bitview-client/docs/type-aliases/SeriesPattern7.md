[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / SeriesPattern7

# Type Alias: SeriesPattern7\<T\>

> **SeriesPattern7**\<`T`\> = `object`

Defined in: [modules/bitview-client/index.js:2646](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L2646)

## Type Parameters

### T

`T`

## Type Declaration

### by

> **by**: `object`

#### by.hour12

> `readonly` **hour12**: [`DateSeriesEndpoint`](../interfaces/DateSeriesEndpoint.md)\<`T`\>

### get

> **get**: (`index`) => [`SeriesEndpoint`](../interfaces/SeriesEndpoint.md)\<`T`\> \| `undefined`

#### Parameters

##### index

[`Index`](Index.md)

#### Returns

[`SeriesEndpoint`](../interfaces/SeriesEndpoint.md)\<`T`\> \| `undefined`

### indexes

> **indexes**: () => readonly [`Index`](Index.md)[]

#### Returns

readonly [`Index`](Index.md)[]

### name

> **name**: `string`
