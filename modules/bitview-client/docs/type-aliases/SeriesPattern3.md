[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / SeriesPattern3

# Type Alias: SeriesPattern3\<T\>

> **SeriesPattern3**\<`T`\> = `object`

Defined in: [modules/bitview-client/index.js:2546](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L2546)

## Type Parameters

### T

`T`

## Type Declaration

### by

> **by**: `object`

#### by.minute10

> `readonly` **minute10**: [`DateSeriesEndpoint`](../interfaces/DateSeriesEndpoint.md)\<`T`\>

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
