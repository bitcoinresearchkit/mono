[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / SeriesPattern

# Interface: SeriesPattern\<T\>

Defined in: [modules/bitview-client/index.js:1950](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1950)

## Type Parameters

### T

`T`

## Properties

### by

> **by**: `Readonly`\<`Partial`\<`Record`\<[`Index`](../type-aliases/Index.md), [`SeriesEndpoint`](SeriesEndpoint.md)\<`T`\>\>\>\>

Defined in: [modules/bitview-client/index.js:1952](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1952)

Index endpoints as lazy getters

***

### get

> **get**: (`index`) => [`SeriesEndpoint`](SeriesEndpoint.md)\<`T`\> \| `undefined`

Defined in: [modules/bitview-client/index.js:1954](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1954)

#### Parameters

##### index

[`Index`](../type-aliases/Index.md)

#### Returns

[`SeriesEndpoint`](SeriesEndpoint.md)\<`T`\> \| `undefined`

***

### indexes

> **indexes**: () => readonly [`Index`](../type-aliases/Index.md)[]

Defined in: [modules/bitview-client/index.js:1953](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1953)

#### Returns

readonly [`Index`](../type-aliases/Index.md)[]

***

### name

> **name**: `string`

Defined in: [modules/bitview-client/index.js:1951](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1951)
