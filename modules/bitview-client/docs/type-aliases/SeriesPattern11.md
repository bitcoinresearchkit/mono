[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / SeriesPattern11

# Type Alias: SeriesPattern11\<T\>

> **SeriesPattern11**\<`T`\> = `object`

Defined in: [modules/bitview-client/index.js:2554](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L2554)

## Type Parameters

### T

`T`

## Type Declaration

### by

> **by**: `object`

#### by.month1

> `readonly` **month1**: [`DateSeriesEndpoint`](../interfaces/DateSeriesEndpoint.md)\<`T`\>

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
