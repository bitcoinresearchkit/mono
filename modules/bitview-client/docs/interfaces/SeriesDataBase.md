[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / SeriesDataBase

# Interface: SeriesDataBase\<T\>

Defined in: [modules/bitview-client/index.js:1744](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1744)

## Type Parameters

### T

`T`

## Properties

### data

> **data**: `T`[]

Defined in: [modules/bitview-client/index.js:1750](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1750)

The series data (`null` where a value is missing or undefined)

***

### end

> **end**: `number`

Defined in: [modules/bitview-client/index.js:1749](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1749)

End index (exclusive)

***

### entries

> **entries**: () => \[`number`, `T`\][]

Defined in: [modules/bitview-client/index.js:1754](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1754)

Get [index, value] pairs

#### Returns

\[`number`, `T`\][]

***

### index

> **index**: [`Index`](../type-aliases/Index.md)

Defined in: [modules/bitview-client/index.js:1746](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1746)

***

### indexes

> **indexes**: () => `number`[]

Defined in: [modules/bitview-client/index.js:1752](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1752)

#### Returns

`number`[]

***

### isDateBased

> **isDateBased**: `boolean`

Defined in: [modules/bitview-client/index.js:1751](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1751)

***

### keys

> **keys**: () => `number`[]

Defined in: [modules/bitview-client/index.js:1753](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1753)

Alias for indexes

#### Returns

`number`[]

***

### start

> **start**: `number`

Defined in: [modules/bitview-client/index.js:1748](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1748)

Start index (inclusive)

***

### toMap

> **toMap**: () => `Map`\<`number`, `T`\>

Defined in: [modules/bitview-client/index.js:1755](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1755)

Convert to Map<index, value>

#### Returns

`Map`\<`number`, `T`\>

***

### type

> **type**: `string`

Defined in: [modules/bitview-client/index.js:1747](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1747)

Value type (e.g. "Ratio", "Sats", "Cents")

***

### version

> **version**: `number`

Defined in: [modules/bitview-client/index.js:1745](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1745)
