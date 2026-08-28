[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / SeriesPattern14

# Type Alias: SeriesPattern14\<T\>

> **SeriesPattern14**\<`T`\> = `object`

Defined in: [Developer/brk/modules/bitview-client/index.js:2590](https://github.com/bitcoinresearchkit/brk/blob/a6ece0db382228669ac4d1a9c673190fe7a4897b/modules/bitview-client/index.js#L2590)

## Type Parameters

### T

`T`

## Type Declaration

### by

> **by**: `object`

#### by.year1

> `readonly` **year1**: [`DateSeriesEndpoint`](../interfaces/DateSeriesEndpoint.md)\<`T`\>

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
