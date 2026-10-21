[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / SeriesPattern

# Interface: SeriesPattern\<T\>

Defined in: [modules/bitview-client/index.js:1848](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1848)

## Type Parameters

### T

`T`

## Properties

### by

> **by**: `Readonly`\<`Partial`\<`Record`\<[`Index`](../type-aliases/Index.md), [`SeriesEndpoint`](SeriesEndpoint.md)\<`T`\>\>\>\>

Defined in: [modules/bitview-client/index.js:1850](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1850)

Index endpoints as lazy getters

***

### get

> **get**: (`index`) => [`SeriesEndpoint`](SeriesEndpoint.md)\<`T`\> \| `undefined`

Defined in: [modules/bitview-client/index.js:1852](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1852)

#### Parameters

##### index

[`Index`](../type-aliases/Index.md)

#### Returns

[`SeriesEndpoint`](SeriesEndpoint.md)\<`T`\> \| `undefined`

***

### indexes

> **indexes**: () => readonly [`Index`](../type-aliases/Index.md)[]

Defined in: [modules/bitview-client/index.js:1851](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1851)

#### Returns

readonly [`Index`](../type-aliases/Index.md)[]

***

### name

> **name**: `string`

Defined in: [modules/bitview-client/index.js:1849](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1849)
