[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / SeriesLeafWithSchema

# Interface: SeriesLeafWithSchema

Defined in: modules/bitview-client/index.js:1353

## Properties

### description?

> `optional` **description?**: `string` \| `null`

Defined in: modules/bitview-client/index.js:1359

Human-readable metric definition, when documented.

***

### indexes

> **indexes**: [`Index`](../type-aliases/Index.md)[]

Defined in: modules/bitview-client/index.js:1356

***

### kind

> **kind**: `string`

Defined in: modules/bitview-client/index.js:1355

The value type, by its schema's name (e.g., "Sats", "Ratio").

***

### name

> **name**: `string`

Defined in: modules/bitview-client/index.js:1354

***

### nullable

> **nullable**: [`Index`](../type-aliases/Index.md)[]

Defined in: modules/bitview-client/index.js:1357

Indexes whose values can be null: a missing value (e.g. a period without blocks) or an
undefined one (e.g. NaN).

***

### type

> **type**: [`JsonType`](../type-aliases/JsonType.md)

Defined in: modules/bitview-client/index.js:1360

What the values are on the wire.
