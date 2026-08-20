[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / SeriesPattern12

# Type Alias: SeriesPattern12\<T\>

> **SeriesPattern12**\<`T`\> = `object`

Defined in: [Developer/brk/modules/bitview-client/index.js:2519](https://github.com/bitcoinresearchkit/brk/blob/b971f8d1b413e122481dddc3c9c980b7866ab1d3/modules/bitview-client/index.js#L2519)

## Type Parameters

### T

`T`

## Type Declaration

### by

> **by**: `object`

#### by.month3

> `readonly` **month3**: [`DateSeriesEndpoint`](../interfaces/DateSeriesEndpoint.md)\<`T`\>

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
