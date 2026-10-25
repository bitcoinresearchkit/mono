[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / SeriesDataBase

# Interface: SeriesDataBase\<T\>

Defined in: [modules/bitview-client/index.js:1938](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1938)

## Type Parameters

### T

`T`

## Properties

### data

> **data**: `T`[]

Defined in: [modules/bitview-client/index.js:1944](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1944)

The series data (`null` where a value is missing or undefined)

***

### end

> **end**: `number`

Defined in: [modules/bitview-client/index.js:1943](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1943)

End index (exclusive)

***

### entries

> **entries**: () => \[`number`, `T`\][]

Defined in: [modules/bitview-client/index.js:1948](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1948)

Get [index, value] pairs

#### Returns

\[`number`, `T`\][]

***

### index

> **index**: [`Index`](../type-aliases/Index.md)

Defined in: [modules/bitview-client/index.js:1940](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1940)

***

### indexes

> **indexes**: () => `number`[]

Defined in: [modules/bitview-client/index.js:1946](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1946)

#### Returns

`number`[]

***

### isDateBased

> **isDateBased**: `boolean`

Defined in: [modules/bitview-client/index.js:1945](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1945)

***

### keys

> **keys**: () => `number`[]

Defined in: [modules/bitview-client/index.js:1947](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1947)

Alias for indexes

#### Returns

`number`[]

***

### start

> **start**: `number`

Defined in: [modules/bitview-client/index.js:1942](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1942)

Start index (inclusive)

***

### toMap

> **toMap**: () => `Map`\<`number`, `T`\>

Defined in: [modules/bitview-client/index.js:1949](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1949)

Convert to Map<index, value>

#### Returns

`Map`\<`number`, `T`\>

***

### type

> **type**: `string`

Defined in: [modules/bitview-client/index.js:1941](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1941)

Value type (e.g. "Ratio", "Sats", "Cents")

***

### version

> **version**: `number`

Defined in: [modules/bitview-client/index.js:1939](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1939)
