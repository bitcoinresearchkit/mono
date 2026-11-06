[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / SeriesDataBase

# Interface: SeriesDataBase\<T\>

Defined in: [modules/bitview-client/index.js:2018](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L2018)

## Type Parameters

### T

`T`

## Properties

### data

> **data**: readonly [`Frozen`](../type-aliases/Frozen.md)\<`T`\>[]

Defined in: [modules/bitview-client/index.js:2024](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L2024)

The series data (`null` where a value is missing or undefined), frozen when the memory cache shares it

***

### end

> **end**: `number`

Defined in: [modules/bitview-client/index.js:2023](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L2023)

End index (exclusive)

***

### entries

> **entries**: () => \[`number`, [`Frozen`](../type-aliases/Frozen.md)\<`T`\>\][]

Defined in: [modules/bitview-client/index.js:2028](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L2028)

Get [index, value] pairs

#### Returns

\[`number`, [`Frozen`](../type-aliases/Frozen.md)\<`T`\>\][]

***

### index

> **index**: [`Index`](../type-aliases/Index.md)

Defined in: [modules/bitview-client/index.js:2020](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L2020)

***

### indexes

> **indexes**: () => `number`[]

Defined in: [modules/bitview-client/index.js:2026](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L2026)

#### Returns

`number`[]

***

### isDateBased

> **isDateBased**: `boolean`

Defined in: [modules/bitview-client/index.js:2025](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L2025)

***

### keys

> **keys**: () => `number`[]

Defined in: [modules/bitview-client/index.js:2027](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L2027)

Alias for indexes

#### Returns

`number`[]

***

### start

> **start**: `number`

Defined in: [modules/bitview-client/index.js:2022](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L2022)

Start index (inclusive)

***

### toMap

> **toMap**: () => `Map`\<`number`, [`Frozen`](../type-aliases/Frozen.md)\<`T`\>\>

Defined in: [modules/bitview-client/index.js:2029](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L2029)

Convert to Map<index, value>

#### Returns

`Map`\<`number`, [`Frozen`](../type-aliases/Frozen.md)\<`T`\>\>

***

### type

> **type**: `string`

Defined in: [modules/bitview-client/index.js:2021](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L2021)

Value type (e.g. "Ratio", "Sats", "Cents")

***

### version

> **version**: `number`

Defined in: [modules/bitview-client/index.js:2019](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L2019)
