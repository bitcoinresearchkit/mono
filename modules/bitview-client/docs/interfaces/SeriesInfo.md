[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / SeriesInfo

# Interface: SeriesInfo

Defined in: [modules/bitview-client/index.js:1254](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1254)

## Properties

### description?

> `optional` **description?**: `string` \| `null`

Defined in: [modules/bitview-client/index.js:1255](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1255)

Human-readable metric definition, when documented

***

### indexes

> **indexes**: [`Index`](../type-aliases/Index.md)[]

Defined in: [modules/bitview-client/index.js:1256](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1256)

***

### nullable

> **nullable**: [`Index`](../type-aliases/Index.md)[]

Defined in: [modules/bitview-client/index.js:1257](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1257)

Indexes whose values can be null: a missing value (e.g. a period without blocks) or an
undefined one (e.g. NaN)

***

### type

> **type**: `string`

Defined in: [modules/bitview-client/index.js:1259](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1259)

Value type (e.g. "Ratio", "Sats", "Cents")

***

### unit?

> `optional` **unit?**: `string` \| `null`

Defined in: [modules/bitview-client/index.js:1260](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1260)

What the value type measures, when documented (e.g. "A duration in days.")
