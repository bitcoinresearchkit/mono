[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / SeriesPattern

# Interface: SeriesPattern\<T\>

Defined in: [modules/bitview-client/index.js:1963](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1963)

## Type Parameters

### T

`T`

## Properties

### by

> **by**: `Readonly`\<`Partial`\<`Record`\<[`Index`](../type-aliases/Index.md), [`SeriesEndpoint`](SeriesEndpoint.md)\<`T`\>\>\>\>

Defined in: [modules/bitview-client/index.js:1965](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1965)

Index endpoints as lazy getters

***

### get

> **get**: (`index`) => [`SeriesEndpoint`](SeriesEndpoint.md)\<`T`\> \| `undefined`

Defined in: [modules/bitview-client/index.js:1967](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1967)

#### Parameters

##### index

[`Index`](../type-aliases/Index.md)

#### Returns

[`SeriesEndpoint`](SeriesEndpoint.md)\<`T`\> \| `undefined`

***

### indexes

> **indexes**: () => readonly [`Index`](../type-aliases/Index.md)[]

Defined in: [modules/bitview-client/index.js:1966](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1966)

#### Returns

readonly [`Index`](../type-aliases/Index.md)[]

***

### name

> **name**: `string`

Defined in: [modules/bitview-client/index.js:1964](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1964)
