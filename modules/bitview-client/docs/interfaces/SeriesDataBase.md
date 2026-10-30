[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / SeriesDataBase

# Interface: SeriesDataBase\<T\>

Defined in: modules/bitview-client/index.js:1968

## Type Parameters

### T

`T`

## Properties

### data

> **data**: `T`[]

Defined in: modules/bitview-client/index.js:1974

The series data (`null` where a value is missing or undefined)

***

### end

> **end**: `number`

Defined in: modules/bitview-client/index.js:1973

End index (exclusive)

***

### entries

> **entries**: () => \[`number`, `T`\][]

Defined in: modules/bitview-client/index.js:1978

Get [index, value] pairs

#### Returns

\[`number`, `T`\][]

***

### index

> **index**: [`Index`](../type-aliases/Index.md)

Defined in: modules/bitview-client/index.js:1970

***

### indexes

> **indexes**: () => `number`[]

Defined in: modules/bitview-client/index.js:1976

#### Returns

`number`[]

***

### isDateBased

> **isDateBased**: `boolean`

Defined in: modules/bitview-client/index.js:1975

***

### keys

> **keys**: () => `number`[]

Defined in: modules/bitview-client/index.js:1977

Alias for indexes

#### Returns

`number`[]

***

### start

> **start**: `number`

Defined in: modules/bitview-client/index.js:1972

Start index (inclusive)

***

### toMap

> **toMap**: () => `Map`\<`number`, `T`\>

Defined in: modules/bitview-client/index.js:1979

Convert to Map<index, value>

#### Returns

`Map`\<`number`, `T`\>

***

### type

> **type**: `string`

Defined in: modules/bitview-client/index.js:1971

Value type (e.g. "Ratio", "Sats", "Cents")

***

### version

> **version**: `number`

Defined in: modules/bitview-client/index.js:1969
