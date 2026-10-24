[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / SeriesInfo

# Interface: SeriesInfo

Defined in: [modules/bitview-client/index.js:1243](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1243)

## Properties

### description?

> `optional` **description?**: `string` \| `null`

Defined in: [modules/bitview-client/index.js:1244](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1244)

Human-readable metric definition, when documented

***

### indexes

> **indexes**: [`Index`](../type-aliases/Index.md)[]

Defined in: [modules/bitview-client/index.js:1245](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1245)

***

### nullable

> **nullable**: [`Index`](../type-aliases/Index.md)[]

Defined in: [modules/bitview-client/index.js:1246](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1246)

Indexes whose values can be null: a missing value (e.g. a period without blocks) or an
undefined one (e.g. NaN)

***

### type

> **type**: `string`

Defined in: [modules/bitview-client/index.js:1248](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1248)

Value type (e.g. "Ratio", "Sats", "Cents")

***

### unit?

> `optional` **unit?**: `string` \| `null`

Defined in: [modules/bitview-client/index.js:1249](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1249)

What the value type measures, when documented (e.g. "A duration in days.")
