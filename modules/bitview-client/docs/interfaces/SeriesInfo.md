[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / SeriesInfo

# Interface: SeriesInfo

Defined in: [modules/bitview-client/index.js:1242](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1242)

## Properties

### description?

> `optional` **description?**: `string` \| `null`

Defined in: [modules/bitview-client/index.js:1243](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1243)

Human-readable metric definition, when documented

***

### indexes

> **indexes**: [`Index`](../type-aliases/Index.md)[]

Defined in: [modules/bitview-client/index.js:1244](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1244)

***

### nullable

> **nullable**: [`Index`](../type-aliases/Index.md)[]

Defined in: [modules/bitview-client/index.js:1245](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1245)

Indexes whose values can be null: a missing value (e.g. a period without blocks) or an
undefined one (e.g. NaN)

***

### type

> **type**: `string`

Defined in: [modules/bitview-client/index.js:1247](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1247)

Value type (e.g. "Ratio", "Sats", "Cents")

***

### unit?

> `optional` **unit?**: `string` \| `null`

Defined in: [modules/bitview-client/index.js:1248](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1248)

What the value type measures, when documented (e.g. "A duration in days.")
