[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / SeriesPattern

# Interface: SeriesPattern\<T\>

Defined in: [modules/bitview-client/index.js:1838](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1838)

## Type Parameters

### T

`T`

## Properties

### by

> **by**: `Readonly`\<`Partial`\<`Record`\<[`Index`](../type-aliases/Index.md), [`SeriesEndpoint`](SeriesEndpoint.md)\<`T`\>\>\>\>

Defined in: [modules/bitview-client/index.js:1840](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1840)

Index endpoints as lazy getters

***

### get

> **get**: (`index`) => [`SeriesEndpoint`](SeriesEndpoint.md)\<`T`\> \| `undefined`

Defined in: [modules/bitview-client/index.js:1842](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1842)

#### Parameters

##### index

[`Index`](../type-aliases/Index.md)

#### Returns

[`SeriesEndpoint`](SeriesEndpoint.md)\<`T`\> \| `undefined`

***

### indexes

> **indexes**: () => readonly [`Index`](../type-aliases/Index.md)[]

Defined in: [modules/bitview-client/index.js:1841](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1841)

#### Returns

readonly [`Index`](../type-aliases/Index.md)[]

***

### name

> **name**: `string`

Defined in: [modules/bitview-client/index.js:1839](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1839)
