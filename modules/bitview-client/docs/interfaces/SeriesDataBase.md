[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / SeriesDataBase

# Interface: SeriesDataBase\<T\>

Defined in: [modules/bitview-client/index.js:1834](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1834)

## Type Parameters

### T

`T`

## Properties

### data

> **data**: `T`[]

Defined in: [modules/bitview-client/index.js:1840](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1840)

The series data (`null` where a value is missing or undefined)

***

### end

> **end**: `number`

Defined in: [modules/bitview-client/index.js:1839](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1839)

End index (exclusive)

***

### entries

> **entries**: () => \[`number`, `T`\][]

Defined in: [modules/bitview-client/index.js:1844](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1844)

Get [index, value] pairs

#### Returns

\[`number`, `T`\][]

***

### index

> **index**: [`Index`](../type-aliases/Index.md)

Defined in: [modules/bitview-client/index.js:1836](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1836)

***

### indexes

> **indexes**: () => `number`[]

Defined in: [modules/bitview-client/index.js:1842](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1842)

#### Returns

`number`[]

***

### isDateBased

> **isDateBased**: `boolean`

Defined in: [modules/bitview-client/index.js:1841](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1841)

***

### keys

> **keys**: () => `number`[]

Defined in: [modules/bitview-client/index.js:1843](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1843)

Alias for indexes

#### Returns

`number`[]

***

### start

> **start**: `number`

Defined in: [modules/bitview-client/index.js:1838](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1838)

Start index (inclusive)

***

### toMap

> **toMap**: () => `Map`\<`number`, `T`\>

Defined in: [modules/bitview-client/index.js:1845](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1845)

Convert to Map<index, value>

#### Returns

`Map`\<`number`, `T`\>

***

### type

> **type**: `string`

Defined in: [modules/bitview-client/index.js:1837](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1837)

Value type (e.g. "Ratio", "Sats", "Cents")

***

### version

> **version**: `number`

Defined in: [modules/bitview-client/index.js:1835](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1835)
