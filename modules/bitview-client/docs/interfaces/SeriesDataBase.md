[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / SeriesDataBase

# Interface: SeriesDataBase\<T\>

Defined in: [modules/bitview-client/index.js:1864](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1864)

## Type Parameters

### T

`T`

## Properties

### data

> **data**: `T`[]

Defined in: [modules/bitview-client/index.js:1870](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1870)

The series data (`null` where a value is missing or undefined)

***

### end

> **end**: `number`

Defined in: [modules/bitview-client/index.js:1869](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1869)

End index (exclusive)

***

### entries

> **entries**: () => \[`number`, `T`\][]

Defined in: [modules/bitview-client/index.js:1874](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1874)

Get [index, value] pairs

#### Returns

\[`number`, `T`\][]

***

### index

> **index**: [`Index`](../type-aliases/Index.md)

Defined in: [modules/bitview-client/index.js:1866](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1866)

***

### indexes

> **indexes**: () => `number`[]

Defined in: [modules/bitview-client/index.js:1872](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1872)

#### Returns

`number`[]

***

### isDateBased

> **isDateBased**: `boolean`

Defined in: [modules/bitview-client/index.js:1871](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1871)

***

### keys

> **keys**: () => `number`[]

Defined in: [modules/bitview-client/index.js:1873](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1873)

Alias for indexes

#### Returns

`number`[]

***

### start

> **start**: `number`

Defined in: [modules/bitview-client/index.js:1868](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1868)

Start index (inclusive)

***

### toMap

> **toMap**: () => `Map`\<`number`, `T`\>

Defined in: [modules/bitview-client/index.js:1875](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1875)

Convert to Map<index, value>

#### Returns

`Map`\<`number`, `T`\>

***

### type

> **type**: `string`

Defined in: [modules/bitview-client/index.js:1867](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1867)

Value type (e.g. "Ratio", "Sats", "Cents")

***

### version

> **version**: `number`

Defined in: [modules/bitview-client/index.js:1865](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1865)
