[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / SeriesPattern

# Interface: SeriesPattern\<T\>

Defined in: [modules/bitview-client/index.js:2042](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L2042)

## Type Parameters

### T

`T`

## Properties

### by

> **by**: `Readonly`\<`Partial`\<`Record`\<[`Index`](../type-aliases/Index.md), [`SeriesEndpoint`](SeriesEndpoint.md)\<`T`\>\>\>\>

Defined in: [modules/bitview-client/index.js:2044](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L2044)

Index endpoints as lazy getters

***

### get

> **get**: (`index`) => [`SeriesEndpoint`](SeriesEndpoint.md)\<`T`\> \| `undefined`

Defined in: [modules/bitview-client/index.js:2046](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L2046)

#### Parameters

##### index

[`Index`](../type-aliases/Index.md)

#### Returns

[`SeriesEndpoint`](SeriesEndpoint.md)\<`T`\> \| `undefined`

***

### indexes

> **indexes**: () => readonly [`Index`](../type-aliases/Index.md)[]

Defined in: [modules/bitview-client/index.js:2045](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L2045)

#### Returns

readonly [`Index`](../type-aliases/Index.md)[]

***

### name

> **name**: `string`

Defined in: [modules/bitview-client/index.js:2043](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L2043)
