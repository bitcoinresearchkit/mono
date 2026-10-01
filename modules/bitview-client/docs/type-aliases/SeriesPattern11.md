[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / SeriesPattern11

# Type Alias: SeriesPattern11\<T\>

> **SeriesPattern11**\<`T`\> = `object`

Defined in: [Developer/mono/modules/bitview-client/index.js:2458](https://github.com/bitcoinresearchkit/mono/blob/ea62dc574077b34b3299e636a16f81ea96849ead/modules/bitview-client/index.js#L2458)

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
