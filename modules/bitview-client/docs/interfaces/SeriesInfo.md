[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / SeriesInfo

# Interface: SeriesInfo

Defined in: [modules/bitview-client/index.js:1272](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1272)

## Properties

### description?

> `optional` **description?**: `string` \| `null`

Defined in: [modules/bitview-client/index.js:1273](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1273)

Human-readable metric definition, when documented

***

### indexes

> **indexes**: [`Index`](../type-aliases/Index.md)[]

Defined in: [modules/bitview-client/index.js:1274](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1274)

***

### nullable

> **nullable**: [`Index`](../type-aliases/Index.md)[]

Defined in: [modules/bitview-client/index.js:1275](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1275)

Indexes whose values can be null: a missing value (e.g. a period without blocks) or an
undefined one (e.g. NaN)

***

### type

> **type**: `string`

Defined in: [modules/bitview-client/index.js:1277](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1277)

Value type (e.g. "Ratio", "Sats", "Cents")

***

### unit?

> `optional` **unit?**: `string` \| `null`

Defined in: [modules/bitview-client/index.js:1278](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1278)

What the value type measures, when documented (e.g. "A duration in days.")
