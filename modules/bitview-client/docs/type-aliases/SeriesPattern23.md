[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / SeriesPattern23

# Type Alias: SeriesPattern23\<T\>

> **SeriesPattern23**\<`T`\> = `object`

Defined in: [Developer/mono/modules/bitview-client/index.js:2617](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L2617)

## Type Parameters

### T

`T`

## Type Declaration

### by

> **by**: `object`

#### by.op\_return\_index

> `readonly` **op\_return\_index**: [`SeriesEndpoint`](../interfaces/SeriesEndpoint.md)\<`T`\>

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
