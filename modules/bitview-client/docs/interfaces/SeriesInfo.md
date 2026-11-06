[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / SeriesInfo

# Interface: SeriesInfo

Defined in: [modules/bitview-client/index.js:1239](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1239)

## Properties

### description?

> `optional` **description?**: `string` \| `null`

Defined in: [modules/bitview-client/index.js:1240](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1240)

Human-readable metric definition, when documented

***

### indexes

> **indexes**: [`Index`](../type-aliases/Index.md)[]

Defined in: [modules/bitview-client/index.js:1241](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1241)

***

### nullable

> **nullable**: [`Index`](../type-aliases/Index.md)[]

Defined in: [modules/bitview-client/index.js:1242](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1242)

Indexes whose values can be null: a missing value (e.g. a period without blocks) or an
  undefined one (e.g. NaN)

***

### type

> **type**: `string`

Defined in: [modules/bitview-client/index.js:1244](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1244)

Value type (e.g. "Ratio", "Sats", "Cents")

***

### unit?

> `optional` **unit?**: `string` \| `null`

Defined in: [modules/bitview-client/index.js:1245](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1245)

What the value type measures, when documented (e.g. "A duration in days.")
