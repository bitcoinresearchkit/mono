[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / SeriesLeafWithSchema

# Interface: SeriesLeafWithSchema

Defined in: [modules/bitview-client/index.js:1351](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1351)

## Properties

### description?

> `optional` **description?**: `string` \| `null`

Defined in: [modules/bitview-client/index.js:1357](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1357)

Human-readable metric definition, when documented.

***

### indexes

> **indexes**: [`Index`](../type-aliases/Index.md)[]

Defined in: [modules/bitview-client/index.js:1354](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1354)

***

### kind

> **kind**: `string`

Defined in: [modules/bitview-client/index.js:1353](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1353)

The value type, by its schema's name (e.g., "Sats", "Ratio").

***

### name

> **name**: `string`

Defined in: [modules/bitview-client/index.js:1352](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1352)

***

### nullable

> **nullable**: [`Index`](../type-aliases/Index.md)[]

Defined in: [modules/bitview-client/index.js:1355](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1355)

Indexes whose values can be null: a missing value (e.g. a period without blocks) or an
undefined one (e.g. NaN).

***

### type

> **type**: [`JsonType`](../type-aliases/JsonType.md)

Defined in: [modules/bitview-client/index.js:1358](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1358)

What the values are on the wire.
