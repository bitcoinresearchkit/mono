[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / SeriesPattern20

# Type Alias: SeriesPattern20\<T\>

> **SeriesPattern20**\<`T`\> = `object`

Defined in: [Developer/brk/modules/bitview-client/index.js:2543](https://github.com/bitcoinresearchkit/brk/blob/b971f8d1b413e122481dddc3c9c980b7866ab1d3/modules/bitview-client/index.js#L2543)

## Type Parameters

### T

`T`

## Type Declaration

### by

> **by**: `object`

#### by.txin\_index

> `readonly` **txin\_index**: [`SeriesEndpoint`](../interfaces/SeriesEndpoint.md)\<`T`\>

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
