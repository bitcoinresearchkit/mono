[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / SeriesLeafWithSchema

# Interface: SeriesLeafWithSchema

Defined in: [modules/bitview-client/index.js:1265](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1265)

## Properties

### description?

> `optional` **description?**: `string` \| `null`

Defined in: [modules/bitview-client/index.js:1271](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1271)

Human-readable metric definition, when documented.

***

### indexes

> **indexes**: [`Index`](../type-aliases/Index.md)[]

Defined in: [modules/bitview-client/index.js:1268](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1268)

***

### kind

> **kind**: `string`

Defined in: [modules/bitview-client/index.js:1267](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1267)

The value type, by its schema's name (e.g., "Sats", "Ratio").

***

### name

> **name**: `string`

Defined in: [modules/bitview-client/index.js:1266](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1266)

***

### nullable

> **nullable**: [`Index`](../type-aliases/Index.md)[]

Defined in: [modules/bitview-client/index.js:1269](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1269)

Indexes whose values can be null: a missing value (e.g. a period without blocks) or an
undefined one (e.g. NaN).

***

### type

> **type**: [`JsonType`](../type-aliases/JsonType.md)

Defined in: [modules/bitview-client/index.js:1272](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1272)

What the values are on the wire.
