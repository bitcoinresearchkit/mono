[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / SeriesDataBase

# Interface: SeriesDataBase\<T\>

Defined in: [modules/bitview-client/index.js:1960](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1960)

## Type Parameters

### T

`T`

## Properties

### data

> **data**: `T`[]

Defined in: [modules/bitview-client/index.js:1966](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1966)

The series data (`null` where a value is missing or undefined)

***

### end

> **end**: `number`

Defined in: [modules/bitview-client/index.js:1965](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1965)

End index (exclusive)

***

### entries

> **entries**: () => \[`number`, `T`\][]

Defined in: [modules/bitview-client/index.js:1970](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1970)

Get [index, value] pairs

#### Returns

\[`number`, `T`\][]

***

### index

> **index**: [`Index`](../type-aliases/Index.md)

Defined in: [modules/bitview-client/index.js:1962](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1962)

***

### indexes

> **indexes**: () => `number`[]

Defined in: [modules/bitview-client/index.js:1968](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1968)

#### Returns

`number`[]

***

### isDateBased

> **isDateBased**: `boolean`

Defined in: [modules/bitview-client/index.js:1967](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1967)

***

### keys

> **keys**: () => `number`[]

Defined in: [modules/bitview-client/index.js:1969](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1969)

Alias for indexes

#### Returns

`number`[]

***

### start

> **start**: `number`

Defined in: [modules/bitview-client/index.js:1964](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1964)

Start index (inclusive)

***

### toMap

> **toMap**: () => `Map`\<`number`, `T`\>

Defined in: [modules/bitview-client/index.js:1971](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1971)

Convert to Map<index, value>

#### Returns

`Map`\<`number`, `T`\>

***

### type

> **type**: `string`

Defined in: [modules/bitview-client/index.js:1963](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1963)

Value type (e.g. "Ratio", "Sats", "Cents")

***

### version

> **version**: `number`

Defined in: [modules/bitview-client/index.js:1961](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1961)
