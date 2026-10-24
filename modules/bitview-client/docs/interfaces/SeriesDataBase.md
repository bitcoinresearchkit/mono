[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / SeriesDataBase

# Interface: SeriesDataBase\<T\>

Defined in: [modules/bitview-client/index.js:1888](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1888)

## Type Parameters

### T

`T`

## Properties

### data

> **data**: `T`[]

Defined in: [modules/bitview-client/index.js:1894](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1894)

The series data (`null` where a value is missing or undefined)

***

### end

> **end**: `number`

Defined in: [modules/bitview-client/index.js:1893](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1893)

End index (exclusive)

***

### entries

> **entries**: () => \[`number`, `T`\][]

Defined in: [modules/bitview-client/index.js:1898](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1898)

Get [index, value] pairs

#### Returns

\[`number`, `T`\][]

***

### index

> **index**: [`Index`](../type-aliases/Index.md)

Defined in: [modules/bitview-client/index.js:1890](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1890)

***

### indexes

> **indexes**: () => `number`[]

Defined in: [modules/bitview-client/index.js:1896](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1896)

#### Returns

`number`[]

***

### isDateBased

> **isDateBased**: `boolean`

Defined in: [modules/bitview-client/index.js:1895](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1895)

***

### keys

> **keys**: () => `number`[]

Defined in: [modules/bitview-client/index.js:1897](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1897)

Alias for indexes

#### Returns

`number`[]

***

### start

> **start**: `number`

Defined in: [modules/bitview-client/index.js:1892](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1892)

Start index (inclusive)

***

### toMap

> **toMap**: () => `Map`\<`number`, `T`\>

Defined in: [modules/bitview-client/index.js:1899](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1899)

Convert to Map<index, value>

#### Returns

`Map`\<`number`, `T`\>

***

### type

> **type**: `string`

Defined in: [modules/bitview-client/index.js:1891](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1891)

Value type (e.g. "Ratio", "Sats", "Cents")

***

### version

> **version**: `number`

Defined in: [modules/bitview-client/index.js:1889](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1889)
