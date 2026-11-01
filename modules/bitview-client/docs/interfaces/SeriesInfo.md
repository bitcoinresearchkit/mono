[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / SeriesInfo

# Interface: SeriesInfo

Defined in: modules/bitview-client/index.js:1342

## Properties

### description?

> `optional` **description?**: `string` \| `null`

Defined in: modules/bitview-client/index.js:1343

Human-readable metric definition, when documented

***

### indexes

> **indexes**: [`Index`](../type-aliases/Index.md)[]

Defined in: modules/bitview-client/index.js:1344

***

### nullable

> **nullable**: [`Index`](../type-aliases/Index.md)[]

Defined in: modules/bitview-client/index.js:1345

Indexes whose values can be null: a missing value (e.g. a period without blocks) or an
undefined one (e.g. NaN)

***

### type

> **type**: `string`

Defined in: modules/bitview-client/index.js:1347

Value type (e.g. "Ratio", "Sats", "Cents")

***

### unit?

> `optional` **unit?**: `string` \| `null`

Defined in: modules/bitview-client/index.js:1348

What the value type measures, when documented (e.g. "A duration in days.")
