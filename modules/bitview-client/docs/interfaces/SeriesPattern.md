[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / SeriesPattern

# Interface: SeriesPattern\<T\>

Defined in: [modules/bitview-client/index.js:1937](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1937)

## Type Parameters

### T

`T`

## Properties

### by

> **by**: `Readonly`\<`Partial`\<`Record`\<[`Index`](../type-aliases/Index.md), [`SeriesEndpoint`](SeriesEndpoint.md)\<`T`\>\>\>\>

Defined in: [modules/bitview-client/index.js:1939](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1939)

Index endpoints as lazy getters

***

### get

> **get**: (`index`) => [`SeriesEndpoint`](SeriesEndpoint.md)\<`T`\> \| `undefined`

Defined in: [modules/bitview-client/index.js:1941](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1941)

#### Parameters

##### index

[`Index`](../type-aliases/Index.md)

#### Returns

[`SeriesEndpoint`](SeriesEndpoint.md)\<`T`\> \| `undefined`

***

### indexes

> **indexes**: () => readonly [`Index`](../type-aliases/Index.md)[]

Defined in: [modules/bitview-client/index.js:1940](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1940)

#### Returns

readonly [`Index`](../type-aliases/Index.md)[]

***

### name

> **name**: `string`

Defined in: [modules/bitview-client/index.js:1938](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1938)
