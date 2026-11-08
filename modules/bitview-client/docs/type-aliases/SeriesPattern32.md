[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / SeriesPattern32

# Type Alias: SeriesPattern32\<T\>

> **SeriesPattern32**\<`T`\> = `object`

Defined in: [modules/bitview-client/index.js:2645](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L2645)

## Type Parameters

### T

`T`

## Type Declaration

### by

> **by**: `object`

#### by.p2sh\_address\_index

> `readonly` **p2sh\_address\_index**: [`SeriesEndpoint`](../interfaces/SeriesEndpoint.md)\<`T`\>

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
