[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / SeriesPattern21

# Type Alias: SeriesPattern21\<T\>

> **SeriesPattern21**\<`T`\> = `object`

Defined in: [Developer/mono/modules/bitview-client/index.js:2611](https://github.com/bitcoinresearchkit/brk/blob/5fc2a239df8aa10fda5a85d6a97dfb7da3f00c2f/modules/bitview-client/index.js#L2611)

## Type Parameters

### T

`T`

## Type Declaration

### by

> **by**: `object`

#### by.txout\_index

> `readonly` **txout\_index**: [`SeriesEndpoint`](../interfaces/SeriesEndpoint.md)\<`T`\>

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
