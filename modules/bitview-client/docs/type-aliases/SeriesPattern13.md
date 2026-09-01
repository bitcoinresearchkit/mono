[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / SeriesPattern13

# Type Alias: SeriesPattern13\<T\>

> **SeriesPattern13**\<`T`\> = `object`

Defined in: [Developer/mono/modules/bitview-client/index.js:2587](https://github.com/bitcoinresearchkit/brk/blob/5fc2a239df8aa10fda5a85d6a97dfb7da3f00c2f/modules/bitview-client/index.js#L2587)

## Type Parameters

### T

`T`

## Type Declaration

### by

> **by**: `object`

#### by.month6

> `readonly` **month6**: [`DateSeriesEndpoint`](../interfaces/DateSeriesEndpoint.md)\<`T`\>

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
