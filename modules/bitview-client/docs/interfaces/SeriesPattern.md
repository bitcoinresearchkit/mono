[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / SeriesPattern

# Interface: SeriesPattern\<T\>

Defined in: [modules/bitview-client/index.js:1915](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1915)

## Type Parameters

### T

`T`

## Properties

### by

> **by**: `Readonly`\<`Partial`\<`Record`\<[`Index`](../type-aliases/Index.md), [`SeriesEndpoint`](SeriesEndpoint.md)\<`T`\>\>\>\>

Defined in: [modules/bitview-client/index.js:1917](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1917)

Index endpoints as lazy getters

***

### get

> **get**: (`index`) => [`SeriesEndpoint`](SeriesEndpoint.md)\<`T`\> \| `undefined`

Defined in: [modules/bitview-client/index.js:1919](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1919)

#### Parameters

##### index

[`Index`](../type-aliases/Index.md)

#### Returns

[`SeriesEndpoint`](SeriesEndpoint.md)\<`T`\> \| `undefined`

***

### indexes

> **indexes**: () => readonly [`Index`](../type-aliases/Index.md)[]

Defined in: [modules/bitview-client/index.js:1918](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1918)

#### Returns

readonly [`Index`](../type-aliases/Index.md)[]

***

### name

> **name**: `string`

Defined in: [modules/bitview-client/index.js:1916](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1916)
