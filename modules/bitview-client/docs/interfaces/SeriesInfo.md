[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / SeriesInfo

# Interface: SeriesInfo

Defined in: [modules/bitview-client/index.js:1153](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1153)

## Properties

### description?

> `optional` **description?**: `string` \| `null`

Defined in: [modules/bitview-client/index.js:1154](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1154)

Human-readable metric definition, when documented

***

### indexes

> **indexes**: [`Index`](../type-aliases/Index.md)[]

Defined in: [modules/bitview-client/index.js:1155](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1155)

***

### nullable

> **nullable**: [`Index`](../type-aliases/Index.md)[]

Defined in: [modules/bitview-client/index.js:1156](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1156)

Indexes whose values can be null: a missing value (e.g. a period without blocks) or an
undefined one (e.g. NaN)

***

### type

> **type**: `string`

Defined in: [modules/bitview-client/index.js:1158](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1158)

Value type (e.g. "Ratio", "Sats", "Cents")

***

### unit?

> `optional` **unit?**: `string` \| `null`

Defined in: [modules/bitview-client/index.js:1159](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1159)

What the value type measures, when documented (e.g. "A duration in days.")
