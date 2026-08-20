[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / SeriesPattern5

# Type Alias: SeriesPattern5\<T\>

> **SeriesPattern5**\<`T`\> = `object`

Defined in: [Developer/brk/modules/bitview-client/index.js:2498](https://github.com/bitcoinresearchkit/brk/blob/b971f8d1b413e122481dddc3c9c980b7866ab1d3/modules/bitview-client/index.js#L2498)

## Type Parameters

### T

`T`

## Type Declaration

### by

> **by**: `object`

#### by.hour1

> `readonly` **hour1**: [`DateSeriesEndpoint`](../interfaces/DateSeriesEndpoint.md)\<`T`\>

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
