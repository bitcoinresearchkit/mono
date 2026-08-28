[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / SeriesLeafWithSchema

# Interface: SeriesLeafWithSchema

Defined in: [Developer/brk/modules/bitview-client/index.js:1241](https://github.com/bitcoinresearchkit/brk/blob/a6ece0db382228669ac4d1a9c673190fe7a4897b/modules/bitview-client/index.js#L1241)

## Properties

### description?

> `optional` **description?**: `string` \| `null`

Defined in: [Developer/brk/modules/bitview-client/index.js:1245](https://github.com/bitcoinresearchkit/brk/blob/a6ece0db382228669ac4d1a9c673190fe7a4897b/modules/bitview-client/index.js#L1245)

Human-readable metric definition, when documented.

***

### indexes

> **indexes**: [`Index`](../type-aliases/Index.md)[]

Defined in: [Developer/brk/modules/bitview-client/index.js:1244](https://github.com/bitcoinresearchkit/brk/blob/a6ece0db382228669ac4d1a9c673190fe7a4897b/modules/bitview-client/index.js#L1244)

Available indexes for this series.

***

### kind

> **kind**: `string`

Defined in: [Developer/brk/modules/bitview-client/index.js:1243](https://github.com/bitcoinresearchkit/brk/blob/a6ece0db382228669ac4d1a9c673190fe7a4897b/modules/bitview-client/index.js#L1243)

The Rust type (e.g., "Sats", "StoredF64").

***

### name

> **name**: `string`

Defined in: [Developer/brk/modules/bitview-client/index.js:1242](https://github.com/bitcoinresearchkit/brk/blob/a6ece0db382228669ac4d1a9c673190fe7a4897b/modules/bitview-client/index.js#L1242)

The series name/identifier.

***

### type

> **type**: `string`

Defined in: [Developer/brk/modules/bitview-client/index.js:1246](https://github.com/bitcoinresearchkit/brk/blob/a6ece0db382228669ac4d1a9c673190fe7a4897b/modules/bitview-client/index.js#L1246)

JSON Schema type (e.g., "integer", "number", "string", "boolean", "array", "object").
