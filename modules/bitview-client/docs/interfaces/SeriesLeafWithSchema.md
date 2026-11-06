[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / SeriesLeafWithSchema

# Interface: SeriesLeafWithSchema

Defined in: [modules/bitview-client/index.js:1361](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1361)

## Properties

### description?

> `optional` **description?**: `string` \| `null`

Defined in: [modules/bitview-client/index.js:1367](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1367)

Human-readable metric definition, when documented.

***

### indexes

> **indexes**: [`Index`](../type-aliases/Index.md)[]

Defined in: [modules/bitview-client/index.js:1364](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1364)

***

### kind

> **kind**: `string`

Defined in: [modules/bitview-client/index.js:1363](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1363)

The value type, by its schema's name (e.g., "Sats", "Ratio").

***

### name

> **name**: `string`

Defined in: [modules/bitview-client/index.js:1362](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1362)

***

### nullable

> **nullable**: [`Index`](../type-aliases/Index.md)[]

Defined in: [modules/bitview-client/index.js:1365](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1365)

Indexes whose values can be null: a missing value (e.g. a period without blocks) or an
  undefined one (e.g. NaN).

***

### type

> **type**: [`JsonType`](../type-aliases/JsonType.md)

Defined in: [modules/bitview-client/index.js:1368](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1368)

What the values are on the wire.
