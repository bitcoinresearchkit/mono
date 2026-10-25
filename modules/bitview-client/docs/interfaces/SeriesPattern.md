[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / SeriesPattern

# Interface: SeriesPattern\<T\>

Defined in: [modules/bitview-client/index.js:2064](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L2064)

## Type Parameters

### T

`T`

## Properties

### by

> **by**: `Readonly`\<`Partial`\<`Record`\<[`Index`](../type-aliases/Index.md), [`SeriesEndpoint`](SeriesEndpoint.md)\<`T`\>\>\>\>

Defined in: [modules/bitview-client/index.js:2066](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L2066)

Index endpoints as lazy getters

***

### get

> **get**: (`index`) => [`SeriesEndpoint`](SeriesEndpoint.md)\<`T`\> \| `undefined`

Defined in: [modules/bitview-client/index.js:2068](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L2068)

#### Parameters

##### index

[`Index`](../type-aliases/Index.md)

#### Returns

[`SeriesEndpoint`](SeriesEndpoint.md)\<`T`\> \| `undefined`

***

### indexes

> **indexes**: () => readonly [`Index`](../type-aliases/Index.md)[]

Defined in: [modules/bitview-client/index.js:2067](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L2067)

#### Returns

readonly [`Index`](../type-aliases/Index.md)[]

***

### name

> **name**: `string`

Defined in: [modules/bitview-client/index.js:2065](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L2065)
