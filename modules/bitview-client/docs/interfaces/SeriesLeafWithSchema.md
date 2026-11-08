[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / SeriesLeafWithSchema

# Interface: SeriesLeafWithSchema

Defined in: [modules/bitview-client/index.js:1320](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1320)

## Properties

### description?

> `optional` **description?**: `string` \| `null`

Defined in: [modules/bitview-client/index.js:1326](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1326)

Human-readable metric definition, when documented.

***

### indexes

> **indexes**: [`Index`](../type-aliases/Index.md)[]

Defined in: [modules/bitview-client/index.js:1323](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1323)

***

### kind

> **kind**: `string`

Defined in: [modules/bitview-client/index.js:1322](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1322)

The value type, by its schema's name (e.g., "Sats", "Ratio").

***

### name

> **name**: `string`

Defined in: [modules/bitview-client/index.js:1321](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1321)

***

### nullable

> **nullable**: [`Index`](../type-aliases/Index.md)[]

Defined in: [modules/bitview-client/index.js:1324](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1324)

Indexes whose values can be null: a missing value (e.g. a period without blocks) or an
  undefined one (e.g. NaN).

***

### type

> **type**: [`JsonType`](../type-aliases/JsonType.md)

Defined in: [modules/bitview-client/index.js:1327](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1327)

What the values are on the wire.
