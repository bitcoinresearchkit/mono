[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / SeriesLeafWithSchema

# Interface: SeriesLeafWithSchema

Defined in: [modules/bitview-client/index.js:1315](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1315)

## Properties

### description?

> `optional` **description?**: `string` \| `null`

Defined in: [modules/bitview-client/index.js:1321](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1321)

Human-readable metric definition, when documented.

***

### indexes

> **indexes**: [`Index`](../type-aliases/Index.md)[]

Defined in: [modules/bitview-client/index.js:1318](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1318)

***

### kind

> **kind**: `string`

Defined in: [modules/bitview-client/index.js:1317](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1317)

The value type, by its schema's name (e.g., "Sats", "Ratio").

***

### name

> **name**: `string`

Defined in: [modules/bitview-client/index.js:1316](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1316)

***

### nullable

> **nullable**: [`Index`](../type-aliases/Index.md)[]

Defined in: [modules/bitview-client/index.js:1319](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1319)

Indexes whose values can be null: a missing value (e.g. a period without blocks) or an
undefined one (e.g. NaN).

***

### type

> **type**: [`JsonType`](../type-aliases/JsonType.md)

Defined in: [modules/bitview-client/index.js:1322](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1322)

What the values are on the wire.
