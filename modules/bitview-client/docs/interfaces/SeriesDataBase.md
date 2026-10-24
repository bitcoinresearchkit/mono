[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / SeriesDataBase

# Interface: SeriesDataBase\<T\>

Defined in: [modules/bitview-client/index.js:1859](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1859)

## Type Parameters

### T

`T`

## Properties

### data

> **data**: `T`[]

Defined in: [modules/bitview-client/index.js:1865](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1865)

The series data (`null` where a value is missing or undefined)

***

### end

> **end**: `number`

Defined in: [modules/bitview-client/index.js:1864](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1864)

End index (exclusive)

***

### entries

> **entries**: () => \[`number`, `T`\][]

Defined in: [modules/bitview-client/index.js:1869](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1869)

Get [index, value] pairs

#### Returns

\[`number`, `T`\][]

***

### index

> **index**: [`Index`](../type-aliases/Index.md)

Defined in: [modules/bitview-client/index.js:1861](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1861)

***

### indexes

> **indexes**: () => `number`[]

Defined in: [modules/bitview-client/index.js:1867](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1867)

#### Returns

`number`[]

***

### isDateBased

> **isDateBased**: `boolean`

Defined in: [modules/bitview-client/index.js:1866](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1866)

***

### keys

> **keys**: () => `number`[]

Defined in: [modules/bitview-client/index.js:1868](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1868)

Alias for indexes

#### Returns

`number`[]

***

### start

> **start**: `number`

Defined in: [modules/bitview-client/index.js:1863](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1863)

Start index (inclusive)

***

### toMap

> **toMap**: () => `Map`\<`number`, `T`\>

Defined in: [modules/bitview-client/index.js:1870](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1870)

Convert to Map<index, value>

#### Returns

`Map`\<`number`, `T`\>

***

### type

> **type**: `string`

Defined in: [modules/bitview-client/index.js:1862](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1862)

Value type (e.g. "Ratio", "Sats", "Cents")

***

### version

> **version**: `number`

Defined in: [modules/bitview-client/index.js:1860](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1860)
