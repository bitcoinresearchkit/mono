[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / SeriesInfo

# Interface: SeriesInfo

Defined in: [modules/bitview-client/index.js:1220](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1220)

## Properties

### description?

> `optional` **description?**: `string` \| `null`

Defined in: [modules/bitview-client/index.js:1221](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1221)

Human-readable metric definition, when documented

***

### indexes

> **indexes**: [`Index`](../type-aliases/Index.md)[]

Defined in: [modules/bitview-client/index.js:1222](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1222)

***

### nullable

> **nullable**: [`Index`](../type-aliases/Index.md)[]

Defined in: [modules/bitview-client/index.js:1223](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1223)

Indexes whose values can be null: a missing value (e.g. a period without blocks) or an
undefined one (e.g. NaN)

***

### type

> **type**: `string`

Defined in: [modules/bitview-client/index.js:1225](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1225)

Value type (e.g. "Ratio", "Sats", "Cents")

***

### unit?

> `optional` **unit?**: `string` \| `null`

Defined in: [modules/bitview-client/index.js:1226](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1226)

What the value type measures, when documented (e.g. "A duration in days.")
