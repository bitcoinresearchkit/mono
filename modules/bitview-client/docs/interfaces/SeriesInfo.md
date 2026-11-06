[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / SeriesInfo

# Interface: SeriesInfo

Defined in: [modules/bitview-client/index.js:1350](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1350)

## Properties

### description?

> `optional` **description?**: `string` \| `null`

Defined in: [modules/bitview-client/index.js:1351](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1351)

Human-readable metric definition, when documented

***

### indexes

> **indexes**: [`Index`](../type-aliases/Index.md)[]

Defined in: [modules/bitview-client/index.js:1352](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1352)

***

### nullable

> **nullable**: [`Index`](../type-aliases/Index.md)[]

Defined in: [modules/bitview-client/index.js:1353](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1353)

Indexes whose values can be null: a missing value (e.g. a period without blocks) or an
  undefined one (e.g. NaN)

***

### type

> **type**: `string`

Defined in: [modules/bitview-client/index.js:1355](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1355)

Value type (e.g. "Ratio", "Sats", "Cents")

***

### unit?

> `optional` **unit?**: `string` \| `null`

Defined in: [modules/bitview-client/index.js:1356](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1356)

What the value type measures, when documented (e.g. "A duration in days.")
