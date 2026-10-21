[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / SeriesLeafWithSchema

# Interface: SeriesLeafWithSchema

Defined in: [modules/bitview-client/index.js:1164](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1164)

## Properties

### description?

> `optional` **description?**: `string` \| `null`

Defined in: [modules/bitview-client/index.js:1170](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1170)

Human-readable metric definition, when documented.

***

### indexes

> **indexes**: [`Index`](../type-aliases/Index.md)[]

Defined in: [modules/bitview-client/index.js:1167](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1167)

***

### kind

> **kind**: `string`

Defined in: [modules/bitview-client/index.js:1166](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1166)

The value type, by its schema's name (e.g., "Sats", "Ratio").

***

### name

> **name**: `string`

Defined in: [modules/bitview-client/index.js:1165](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1165)

***

### nullable

> **nullable**: [`Index`](../type-aliases/Index.md)[]

Defined in: [modules/bitview-client/index.js:1168](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1168)

Indexes whose values can be null: a missing value (e.g. a period without blocks) or an
undefined one (e.g. NaN).

***

### type

> **type**: [`JsonType`](../type-aliases/JsonType.md)

Defined in: [modules/bitview-client/index.js:1171](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1171)

What the values are on the wire.
