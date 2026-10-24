[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / SeriesLeafWithSchema

# Interface: SeriesLeafWithSchema

Defined in: [modules/bitview-client/index.js:1254](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1254)

## Properties

### description?

> `optional` **description?**: `string` \| `null`

Defined in: [modules/bitview-client/index.js:1260](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1260)

Human-readable metric definition, when documented.

***

### indexes

> **indexes**: [`Index`](../type-aliases/Index.md)[]

Defined in: [modules/bitview-client/index.js:1257](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1257)

***

### kind

> **kind**: `string`

Defined in: [modules/bitview-client/index.js:1256](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1256)

The value type, by its schema's name (e.g., "Sats", "Ratio").

***

### name

> **name**: `string`

Defined in: [modules/bitview-client/index.js:1255](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1255)

***

### nullable

> **nullable**: [`Index`](../type-aliases/Index.md)[]

Defined in: [modules/bitview-client/index.js:1258](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1258)

Indexes whose values can be null: a missing value (e.g. a period without blocks) or an
undefined one (e.g. NaN).

***

### type

> **type**: [`JsonType`](../type-aliases/JsonType.md)

Defined in: [modules/bitview-client/index.js:1261](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1261)

What the values are on the wire.
