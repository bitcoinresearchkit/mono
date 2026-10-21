[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / SeriesLeafWithSchema

# Interface: SeriesLeafWithSchema

Defined in: [modules/bitview-client/index.js:1159](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1159)

## Properties

### description?

> `optional` **description?**: `string` \| `null`

Defined in: [modules/bitview-client/index.js:1165](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1165)

Human-readable metric definition, when documented.

***

### indexes

> **indexes**: [`Index`](../type-aliases/Index.md)[]

Defined in: [modules/bitview-client/index.js:1162](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1162)

***

### kind

> **kind**: `string`

Defined in: [modules/bitview-client/index.js:1161](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1161)

The value type, by its schema's name (e.g., "Sats", "Ratio").

***

### name

> **name**: `string`

Defined in: [modules/bitview-client/index.js:1160](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1160)

***

### nullable

> **nullable**: [`Index`](../type-aliases/Index.md)[]

Defined in: [modules/bitview-client/index.js:1163](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1163)

Indexes whose values can be null: a missing value (e.g. a period without blocks) or an
undefined one (e.g. NaN).

***

### type

> **type**: [`JsonType`](../type-aliases/JsonType.md)

Defined in: [modules/bitview-client/index.js:1166](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1166)

What the values are on the wire.
