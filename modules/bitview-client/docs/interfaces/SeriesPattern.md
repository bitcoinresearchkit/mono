[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / SeriesPattern

# Interface: SeriesPattern\<T\>

Defined in: [modules/bitview-client/index.js:1968](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1968)

## Type Parameters

### T

`T`

## Properties

### by

> **by**: `Readonly`\<`Partial`\<`Record`\<[`Index`](../type-aliases/Index.md), [`SeriesEndpoint`](SeriesEndpoint.md)\<`T`\>\>\>\>

Defined in: [modules/bitview-client/index.js:1970](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1970)

Index endpoints as lazy getters

***

### get

> **get**: (`index`) => [`SeriesEndpoint`](SeriesEndpoint.md)\<`T`\> \| `undefined`

Defined in: [modules/bitview-client/index.js:1972](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1972)

#### Parameters

##### index

[`Index`](../type-aliases/Index.md)

#### Returns

[`SeriesEndpoint`](SeriesEndpoint.md)\<`T`\> \| `undefined`

***

### indexes

> **indexes**: () => readonly [`Index`](../type-aliases/Index.md)[]

Defined in: [modules/bitview-client/index.js:1971](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1971)

#### Returns

readonly [`Index`](../type-aliases/Index.md)[]

***

### name

> **name**: `string`

Defined in: [modules/bitview-client/index.js:1969](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1969)
