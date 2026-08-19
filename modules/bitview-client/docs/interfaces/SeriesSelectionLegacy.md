[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / SeriesSelectionLegacy

# Interface: SeriesSelectionLegacy

Defined in: [Developer/brk/modules/bitview-client/index.js:1230](https://github.com/bitcoinresearchkit/brk/blob/70a3e90b82b397fa1d5ca46c4ed613066ed3d195/modules/bitview-client/index.js#L1230)

## Properties

### end?

> `optional` **end?**: [`RangeIndex`](../type-aliases/RangeIndex.md) \| `null`

Defined in: [Developer/brk/modules/bitview-client/index.js:1234](https://github.com/bitcoinresearchkit/brk/blob/70a3e90b82b397fa1d5ca46c4ed613066ed3d195/modules/bitview-client/index.js#L1234)

Exclusive end: integer index, date (YYYY-MM-DD), or timestamp (ISO 8601). Negative integers count from end. Aliases: `to`, `t`, `e`

***

### format?

> `optional` **format?**: [`Format`](../type-aliases/Format.md)

Defined in: [Developer/brk/modules/bitview-client/index.js:1236](https://github.com/bitcoinresearchkit/brk/blob/70a3e90b82b397fa1d5ca46c4ed613066ed3d195/modules/bitview-client/index.js#L1236)

Format of the output

***

### ids

> **ids**: `string`

Defined in: [Developer/brk/modules/bitview-client/index.js:1232](https://github.com/bitcoinresearchkit/brk/blob/70a3e90b82b397fa1d5ca46c4ed613066ed3d195/modules/bitview-client/index.js#L1232)

***

### index

> **index**: [`Index`](../type-aliases/Index.md)

Defined in: [Developer/brk/modules/bitview-client/index.js:1231](https://github.com/bitcoinresearchkit/brk/blob/70a3e90b82b397fa1d5ca46c4ed613066ed3d195/modules/bitview-client/index.js#L1231)

***

### limit?

> `optional` **limit?**: `number` \| `null`

Defined in: [Developer/brk/modules/bitview-client/index.js:1235](https://github.com/bitcoinresearchkit/brk/blob/70a3e90b82b397fa1d5ca46c4ed613066ed3d195/modules/bitview-client/index.js#L1235)

Maximum number of values to return (ignored if `end` is set). Aliases: `count`, `c`, `l`

***

### start?

> `optional` **start?**: [`RangeIndex`](../type-aliases/RangeIndex.md) \| `null`

Defined in: [Developer/brk/modules/bitview-client/index.js:1233](https://github.com/bitcoinresearchkit/brk/blob/70a3e90b82b397fa1d5ca46c4ed613066ed3d195/modules/bitview-client/index.js#L1233)

Inclusive start: integer index, date (YYYY-MM-DD), or timestamp (ISO 8601). Negative integers count from end. Aliases: `from`, `f`, `s`
