[**brk-client**](../README.md)

***

[brk-client](../globals.md) / SeriesPattern3

# Type Alias: SeriesPattern3\<T\>

> **SeriesPattern3**\<`T`\> = `object`

Defined in: [Developer/brk/modules/brk-client/index.js:2492](https://github.com/bitcoinresearchkit/brk/blob/70a3e90b82b397fa1d5ca46c4ed613066ed3d195/modules/brk-client/index.js#L2492)

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
