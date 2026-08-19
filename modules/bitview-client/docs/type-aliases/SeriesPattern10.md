[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / SeriesPattern10

# Type Alias: SeriesPattern10\<T\>

> **SeriesPattern10**\<`T`\> = `object`

Defined in: [Developer/brk/modules/bitview-client/index.js:2513](https://github.com/bitcoinresearchkit/brk/blob/70a3e90b82b397fa1d5ca46c4ed613066ed3d195/modules/bitview-client/index.js#L2513)

## Type Parameters

### T

`T`

## Type Declaration

### by

> **by**: `object`

#### by.week1

> `readonly` **week1**: [`DateSeriesEndpoint`](../interfaces/DateSeriesEndpoint.md)\<`T`\>

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
