[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / SeriesDataBase

# Interface: SeriesDataBase\<T\>

Defined in: [modules/bitview-client/index.js:1846](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1846)

## Type Parameters

### T

`T`

## Properties

### data

> **data**: `T`[]

Defined in: [modules/bitview-client/index.js:1852](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1852)

The series data (`null` where a value is missing or undefined)

***

### end

> **end**: `number`

Defined in: [modules/bitview-client/index.js:1851](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1851)

End index (exclusive)

***

### entries

> **entries**: () => \[`number`, `T`\][]

Defined in: [modules/bitview-client/index.js:1856](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1856)

Get [index, value] pairs

#### Returns

\[`number`, `T`\][]

***

### index

> **index**: [`Index`](../type-aliases/Index.md)

Defined in: [modules/bitview-client/index.js:1848](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1848)

***

### indexes

> **indexes**: () => `number`[]

Defined in: [modules/bitview-client/index.js:1854](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1854)

#### Returns

`number`[]

***

### isDateBased

> **isDateBased**: `boolean`

Defined in: [modules/bitview-client/index.js:1853](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1853)

***

### keys

> **keys**: () => `number`[]

Defined in: [modules/bitview-client/index.js:1855](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1855)

Alias for indexes

#### Returns

`number`[]

***

### start

> **start**: `number`

Defined in: [modules/bitview-client/index.js:1850](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1850)

Start index (inclusive)

***

### toMap

> **toMap**: () => `Map`\<`number`, `T`\>

Defined in: [modules/bitview-client/index.js:1857](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1857)

Convert to Map<index, value>

#### Returns

`Map`\<`number`, `T`\>

***

### type

> **type**: `string`

Defined in: [modules/bitview-client/index.js:1849](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1849)

Value type (e.g. "Ratio", "Sats", "Cents")

***

### version

> **version**: `number`

Defined in: [modules/bitview-client/index.js:1847](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1847)
