[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / SeriesDataBase

# Interface: SeriesDataBase\<T\>

Defined in: [modules/bitview-client/index.js:1811](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1811)

## Type Parameters

### T

`T`

## Properties

### data

> **data**: `T`[]

Defined in: [modules/bitview-client/index.js:1817](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1817)

The series data (`null` where a value is missing or undefined)

***

### end

> **end**: `number`

Defined in: [modules/bitview-client/index.js:1816](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1816)

End index (exclusive)

***

### entries

> **entries**: () => \[`number`, `T`\][]

Defined in: [modules/bitview-client/index.js:1821](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1821)

Get [index, value] pairs

#### Returns

\[`number`, `T`\][]

***

### index

> **index**: [`Index`](../type-aliases/Index.md)

Defined in: [modules/bitview-client/index.js:1813](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1813)

***

### indexes

> **indexes**: () => `number`[]

Defined in: [modules/bitview-client/index.js:1819](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1819)

#### Returns

`number`[]

***

### isDateBased

> **isDateBased**: `boolean`

Defined in: [modules/bitview-client/index.js:1818](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1818)

***

### keys

> **keys**: () => `number`[]

Defined in: [modules/bitview-client/index.js:1820](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1820)

Alias for indexes

#### Returns

`number`[]

***

### start

> **start**: `number`

Defined in: [modules/bitview-client/index.js:1815](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1815)

Start index (inclusive)

***

### toMap

> **toMap**: () => `Map`\<`number`, `T`\>

Defined in: [modules/bitview-client/index.js:1822](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1822)

Convert to Map<index, value>

#### Returns

`Map`\<`number`, `T`\>

***

### type

> **type**: `string`

Defined in: [modules/bitview-client/index.js:1814](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1814)

Value type (e.g. "Ratio", "Sats", "Cents")

***

### version

> **version**: `number`

Defined in: [modules/bitview-client/index.js:1812](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1812)
