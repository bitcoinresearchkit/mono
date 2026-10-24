[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / SeriesDataBase

# Interface: SeriesDataBase\<T\>

Defined in: [modules/bitview-client/index.js:1920](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1920)

## Type Parameters

### T

`T`

## Properties

### data

> **data**: `T`[]

Defined in: [modules/bitview-client/index.js:1926](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1926)

The series data (`null` where a value is missing or undefined)

***

### end

> **end**: `number`

Defined in: [modules/bitview-client/index.js:1925](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1925)

End index (exclusive)

***

### entries

> **entries**: () => \[`number`, `T`\][]

Defined in: [modules/bitview-client/index.js:1930](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1930)

Get [index, value] pairs

#### Returns

\[`number`, `T`\][]

***

### index

> **index**: [`Index`](../type-aliases/Index.md)

Defined in: [modules/bitview-client/index.js:1922](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1922)

***

### indexes

> **indexes**: () => `number`[]

Defined in: [modules/bitview-client/index.js:1928](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1928)

#### Returns

`number`[]

***

### isDateBased

> **isDateBased**: `boolean`

Defined in: [modules/bitview-client/index.js:1927](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1927)

***

### keys

> **keys**: () => `number`[]

Defined in: [modules/bitview-client/index.js:1929](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1929)

Alias for indexes

#### Returns

`number`[]

***

### start

> **start**: `number`

Defined in: [modules/bitview-client/index.js:1924](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1924)

Start index (inclusive)

***

### toMap

> **toMap**: () => `Map`\<`number`, `T`\>

Defined in: [modules/bitview-client/index.js:1931](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1931)

Convert to Map<index, value>

#### Returns

`Map`\<`number`, `T`\>

***

### type

> **type**: `string`

Defined in: [modules/bitview-client/index.js:1923](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1923)

Value type (e.g. "Ratio", "Sats", "Cents")

***

### version

> **version**: `number`

Defined in: [modules/bitview-client/index.js:1921](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1921)
