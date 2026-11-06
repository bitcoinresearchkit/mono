[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / SeriesPattern

# Interface: SeriesPattern\<T\>

Defined in: [modules/bitview-client/index.js:2122](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L2122)

## Type Parameters

### T

`T`

## Properties

### by

> **by**: `Readonly`\<`Partial`\<`Record`\<[`Index`](../type-aliases/Index.md), [`SeriesEndpoint`](SeriesEndpoint.md)\<`T`\>\>\>\>

Defined in: [modules/bitview-client/index.js:2124](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L2124)

Index endpoints as lazy getters

***

### get

> **get**: (`index`) => [`SeriesEndpoint`](SeriesEndpoint.md)\<`T`\> \| `undefined`

Defined in: [modules/bitview-client/index.js:2126](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L2126)

#### Parameters

##### index

[`Index`](../type-aliases/Index.md)

#### Returns

[`SeriesEndpoint`](SeriesEndpoint.md)\<`T`\> \| `undefined`

***

### indexes

> **indexes**: () => readonly [`Index`](../type-aliases/Index.md)[]

Defined in: [modules/bitview-client/index.js:2125](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L2125)

#### Returns

readonly [`Index`](../type-aliases/Index.md)[]

***

### name

> **name**: `string`

Defined in: [modules/bitview-client/index.js:2123](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L2123)
