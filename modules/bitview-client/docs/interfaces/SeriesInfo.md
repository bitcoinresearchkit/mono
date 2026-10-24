[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / SeriesInfo

# Interface: SeriesInfo

Defined in: [modules/bitview-client/index.js:1304](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1304)

## Properties

### description?

> `optional` **description?**: `string` \| `null`

Defined in: [modules/bitview-client/index.js:1305](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1305)

Human-readable metric definition, when documented

***

### indexes

> **indexes**: [`Index`](../type-aliases/Index.md)[]

Defined in: [modules/bitview-client/index.js:1306](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1306)

***

### nullable

> **nullable**: [`Index`](../type-aliases/Index.md)[]

Defined in: [modules/bitview-client/index.js:1307](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1307)

Indexes whose values can be null: a missing value (e.g. a period without blocks) or an
undefined one (e.g. NaN)

***

### type

> **type**: `string`

Defined in: [modules/bitview-client/index.js:1309](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1309)

Value type (e.g. "Ratio", "Sats", "Cents")

***

### unit?

> `optional` **unit?**: `string` \| `null`

Defined in: [modules/bitview-client/index.js:1310](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1310)

What the value type measures, when documented (e.g. "A duration in days.")
