[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / SeriesInfo

# Interface: SeriesInfo

Defined in: [modules/bitview-client/index.js:1340](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1340)

## Properties

### description?

> `optional` **description?**: `string` \| `null`

Defined in: [modules/bitview-client/index.js:1341](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1341)

Human-readable metric definition, when documented

***

### indexes

> **indexes**: [`Index`](../type-aliases/Index.md)[]

Defined in: [modules/bitview-client/index.js:1342](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1342)

***

### nullable

> **nullable**: [`Index`](../type-aliases/Index.md)[]

Defined in: [modules/bitview-client/index.js:1343](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1343)

Indexes whose values can be null: a missing value (e.g. a period without blocks) or an
undefined one (e.g. NaN)

***

### type

> **type**: `string`

Defined in: [modules/bitview-client/index.js:1345](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1345)

Value type (e.g. "Ratio", "Sats", "Cents")

***

### unit?

> `optional` **unit?**: `string` \| `null`

Defined in: [modules/bitview-client/index.js:1346](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1346)

What the value type measures, when documented (e.g. "A duration in days.")
