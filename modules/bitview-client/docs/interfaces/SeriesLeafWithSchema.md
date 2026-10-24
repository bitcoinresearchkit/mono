[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / SeriesLeafWithSchema

# Interface: SeriesLeafWithSchema

Defined in: [modules/bitview-client/index.js:1283](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1283)

## Properties

### description?

> `optional` **description?**: `string` \| `null`

Defined in: [modules/bitview-client/index.js:1289](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1289)

Human-readable metric definition, when documented.

***

### indexes

> **indexes**: [`Index`](../type-aliases/Index.md)[]

Defined in: [modules/bitview-client/index.js:1286](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1286)

***

### kind

> **kind**: `string`

Defined in: [modules/bitview-client/index.js:1285](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1285)

The value type, by its schema's name (e.g., "Sats", "Ratio").

***

### name

> **name**: `string`

Defined in: [modules/bitview-client/index.js:1284](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1284)

***

### nullable

> **nullable**: [`Index`](../type-aliases/Index.md)[]

Defined in: [modules/bitview-client/index.js:1287](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1287)

Indexes whose values can be null: a missing value (e.g. a period without blocks) or an
undefined one (e.g. NaN).

***

### type

> **type**: [`JsonType`](../type-aliases/JsonType.md)

Defined in: [modules/bitview-client/index.js:1290](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1290)

What the values are on the wire.
