[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / SeriesInfo

# Interface: SeriesInfo

Defined in: [modules/bitview-client/index.js:1322](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1322)

## Properties

### description?

> `optional` **description?**: `string` \| `null`

Defined in: [modules/bitview-client/index.js:1323](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1323)

Human-readable metric definition, when documented

***

### indexes

> **indexes**: [`Index`](../type-aliases/Index.md)[]

Defined in: [modules/bitview-client/index.js:1324](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1324)

***

### nullable

> **nullable**: [`Index`](../type-aliases/Index.md)[]

Defined in: [modules/bitview-client/index.js:1325](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1325)

Indexes whose values can be null: a missing value (e.g. a period without blocks) or an
undefined one (e.g. NaN)

***

### type

> **type**: `string`

Defined in: [modules/bitview-client/index.js:1327](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1327)

Value type (e.g. "Ratio", "Sats", "Cents")

***

### unit?

> `optional` **unit?**: `string` \| `null`

Defined in: [modules/bitview-client/index.js:1328](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1328)

What the value type measures, when documented (e.g. "A duration in days.")
