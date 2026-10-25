[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / SeriesLeafWithSchema

# Interface: SeriesLeafWithSchema

Defined in: [modules/bitview-client/index.js:1333](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1333)

## Properties

### description?

> `optional` **description?**: `string` \| `null`

Defined in: [modules/bitview-client/index.js:1339](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1339)

Human-readable metric definition, when documented.

***

### indexes

> **indexes**: [`Index`](../type-aliases/Index.md)[]

Defined in: [modules/bitview-client/index.js:1336](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1336)

***

### kind

> **kind**: `string`

Defined in: [modules/bitview-client/index.js:1335](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1335)

The value type, by its schema's name (e.g., "Sats", "Ratio").

***

### name

> **name**: `string`

Defined in: [modules/bitview-client/index.js:1334](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1334)

***

### nullable

> **nullable**: [`Index`](../type-aliases/Index.md)[]

Defined in: [modules/bitview-client/index.js:1337](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1337)

Indexes whose values can be null: a missing value (e.g. a period without blocks) or an
undefined one (e.g. NaN).

***

### type

> **type**: [`JsonType`](../type-aliases/JsonType.md)

Defined in: [modules/bitview-client/index.js:1340](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1340)

What the values are on the wire.
