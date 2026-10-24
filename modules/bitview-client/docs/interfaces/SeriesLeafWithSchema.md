[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / SeriesLeafWithSchema

# Interface: SeriesLeafWithSchema

Defined in: [modules/bitview-client/index.js:1231](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1231)

## Properties

### description?

> `optional` **description?**: `string` \| `null`

Defined in: [modules/bitview-client/index.js:1237](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1237)

Human-readable metric definition, when documented.

***

### indexes

> **indexes**: [`Index`](../type-aliases/Index.md)[]

Defined in: [modules/bitview-client/index.js:1234](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1234)

***

### kind

> **kind**: `string`

Defined in: [modules/bitview-client/index.js:1233](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1233)

The value type, by its schema's name (e.g., "Sats", "Ratio").

***

### name

> **name**: `string`

Defined in: [modules/bitview-client/index.js:1232](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1232)

***

### nullable

> **nullable**: [`Index`](../type-aliases/Index.md)[]

Defined in: [modules/bitview-client/index.js:1235](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1235)

Indexes whose values can be null: a missing value (e.g. a period without blocks) or an
undefined one (e.g. NaN).

***

### type

> **type**: [`JsonType`](../type-aliases/JsonType.md)

Defined in: [modules/bitview-client/index.js:1238](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1238)

What the values are on the wire.
