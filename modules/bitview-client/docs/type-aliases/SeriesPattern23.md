[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / SeriesPattern23

# Type Alias: SeriesPattern23\<T\>

> **SeriesPattern23**\<`T`\> = `object`

Defined in: [modules/bitview-client/index.js:2434](https://github.com/bitcoinresearchkit/mono/blob/35af38a95a9d8eab77d0bc6900ec332bc53accaf/modules/bitview-client/index.js#L2434)

## Type Parameters

### T

`T`

## Type Declaration

### by

> **by**: `object`

#### by.txout\_index

> `readonly` **txout\_index**: [`SeriesEndpoint`](../interfaces/SeriesEndpoint.md)\<`T` \| `null`\>

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
