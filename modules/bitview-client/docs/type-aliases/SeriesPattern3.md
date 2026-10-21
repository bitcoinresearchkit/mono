[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / SeriesPattern3

# Type Alias: SeriesPattern3\<T\>

> **SeriesPattern3**\<`T`\> = `object`

Defined in: [modules/bitview-client/index.js:2414](https://github.com/bitcoinresearchkit/mono/blob/35af38a95a9d8eab77d0bc6900ec332bc53accaf/modules/bitview-client/index.js#L2414)

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
