[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / SeriesPattern

# Interface: SeriesPattern\<T\>

Defined in: [modules/bitview-client/index.js:2024](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L2024)

## Type Parameters

### T

`T`

## Properties

### by

> **by**: `Readonly`\<`Partial`\<`Record`\<[`Index`](../type-aliases/Index.md), [`SeriesEndpoint`](SeriesEndpoint.md)\<`T`\>\>\>\>

Defined in: [modules/bitview-client/index.js:2026](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L2026)

Index endpoints as lazy getters

***

### get

> **get**: (`index`) => [`SeriesEndpoint`](SeriesEndpoint.md)\<`T`\> \| `undefined`

Defined in: [modules/bitview-client/index.js:2028](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L2028)

#### Parameters

##### index

[`Index`](../type-aliases/Index.md)

#### Returns

[`SeriesEndpoint`](SeriesEndpoint.md)\<`T`\> \| `undefined`

***

### indexes

> **indexes**: () => readonly [`Index`](../type-aliases/Index.md)[]

Defined in: [modules/bitview-client/index.js:2027](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L2027)

#### Returns

readonly [`Index`](../type-aliases/Index.md)[]

***

### name

> **name**: `string`

Defined in: [modules/bitview-client/index.js:2025](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L2025)
