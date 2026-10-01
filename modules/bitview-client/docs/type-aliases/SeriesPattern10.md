[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / SeriesPattern10

# Type Alias: SeriesPattern10\<T\>

> **SeriesPattern10**\<`T`\> = `object`

Defined in: [Developer/mono/modules/bitview-client/index.js:2455](https://github.com/bitcoinresearchkit/mono/blob/ea62dc574077b34b3299e636a16f81ea96849ead/modules/bitview-client/index.js#L2455)

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
