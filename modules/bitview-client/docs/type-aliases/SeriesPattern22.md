[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / SeriesPattern22

# Type Alias: SeriesPattern22\<T\>

> **SeriesPattern22**\<`T`\> = `object`

Defined in: [modules/bitview-client/index.js:2635](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L2635)

## Type Parameters

### T

`T`

## Type Declaration

### by

> **by**: `object`

#### by.txin\_index

> `readonly` **txin\_index**: [`SeriesEndpoint`](../interfaces/SeriesEndpoint.md)\<`T` \| `null`\>

### get

> **get**: (`index`) => [`SeriesEndpoint`](../interfaces/SeriesEndpoint.md)\<`T` \| `null`\> \| `undefined`

#### Parameters

##### index

[`Index`](Index.md)

#### Returns

[`SeriesEndpoint`](../interfaces/SeriesEndpoint.md)\<`T` \| `null`\> \| `undefined`

### indexes

> **indexes**: () => readonly [`Index`](Index.md)[]

#### Returns

readonly [`Index`](Index.md)[]

### name

> **name**: `string`
