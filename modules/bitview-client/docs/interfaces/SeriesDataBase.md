[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / SeriesDataBase

# Interface: SeriesDataBase\<T\>

Defined in: [modules/bitview-client/index.js:1837](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1837)

## Type Parameters

### T

`T`

## Properties

### data

> **data**: readonly [`Frozen`](../type-aliases/Frozen.md)\<`T`\>[]

Defined in: [modules/bitview-client/index.js:1843](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1843)

The series data (`null` where a value is missing or undefined), frozen when the memory cache shares it

***

### end

> **end**: `number`

Defined in: [modules/bitview-client/index.js:1842](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1842)

End index (exclusive)

***

### entries

> **entries**: () => \[`number`, [`Frozen`](../type-aliases/Frozen.md)\<`T`\>\][]

Defined in: [modules/bitview-client/index.js:1847](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1847)

Get [index, value] pairs

#### Returns

\[`number`, [`Frozen`](../type-aliases/Frozen.md)\<`T`\>\][]

***

### index

> **index**: [`Index`](../type-aliases/Index.md)

Defined in: [modules/bitview-client/index.js:1839](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1839)

***

### indexes

> **indexes**: () => `number`[]

Defined in: [modules/bitview-client/index.js:1845](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1845)

#### Returns

`number`[]

***

### isDateBased

> **isDateBased**: `boolean`

Defined in: [modules/bitview-client/index.js:1844](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1844)

***

### keys

> **keys**: () => `number`[]

Defined in: [modules/bitview-client/index.js:1846](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1846)

Alias for indexes

#### Returns

`number`[]

***

### start

> **start**: `number`

Defined in: [modules/bitview-client/index.js:1841](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1841)

Start index (inclusive)

***

### toMap

> **toMap**: () => `Map`\<`number`, [`Frozen`](../type-aliases/Frozen.md)\<`T`\>\>

Defined in: [modules/bitview-client/index.js:1848](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1848)

Convert to Map<index, value>

#### Returns

`Map`\<`number`, [`Frozen`](../type-aliases/Frozen.md)\<`T`\>\>

***

### type

> **type**: `string`

Defined in: [modules/bitview-client/index.js:1840](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1840)

Value type (e.g. "Ratio", "Sats", "Cents")

***

### version

> **version**: `number`

Defined in: [modules/bitview-client/index.js:1838](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1838)
