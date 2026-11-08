[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / SeriesPattern24

# Type Alias: SeriesPattern24\<T\>

> **SeriesPattern24**\<`T`\> = `object`

Defined in: [modules/bitview-client/index.js:2637](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L2637)

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
