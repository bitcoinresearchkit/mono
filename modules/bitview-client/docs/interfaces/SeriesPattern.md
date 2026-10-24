[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / SeriesPattern

# Interface: SeriesPattern\<T\>

Defined in: [modules/bitview-client/index.js:1992](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1992)

## Type Parameters

### T

`T`

## Properties

### by

> **by**: `Readonly`\<`Partial`\<`Record`\<[`Index`](../type-aliases/Index.md), [`SeriesEndpoint`](SeriesEndpoint.md)\<`T`\>\>\>\>

Defined in: [modules/bitview-client/index.js:1994](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1994)

Index endpoints as lazy getters

***

### get

> **get**: (`index`) => [`SeriesEndpoint`](SeriesEndpoint.md)\<`T`\> \| `undefined`

Defined in: [modules/bitview-client/index.js:1996](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1996)

#### Parameters

##### index

[`Index`](../type-aliases/Index.md)

#### Returns

[`SeriesEndpoint`](SeriesEndpoint.md)\<`T`\> \| `undefined`

***

### indexes

> **indexes**: () => readonly [`Index`](../type-aliases/Index.md)[]

Defined in: [modules/bitview-client/index.js:1995](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1995)

#### Returns

readonly [`Index`](../type-aliases/Index.md)[]

***

### name

> **name**: `string`

Defined in: [modules/bitview-client/index.js:1993](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1993)
