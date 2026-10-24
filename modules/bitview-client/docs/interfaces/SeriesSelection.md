[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / SeriesSelection

# Interface: SeriesSelection

Defined in: [modules/bitview-client/index.js:1281](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1281)

## Properties

### end?

> `optional` **end?**: [`RangeIndex`](../type-aliases/RangeIndex.md) \| `null`

Defined in: [modules/bitview-client/index.js:1285](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1285)

Exclusive end: integer index, date (YYYY-MM-DD), or timestamp (ISO 8601). Negative integers count from end. REST aliases: `to`, `t`, `e`

***

### format?

> `optional` **format?**: [`Format`](../type-aliases/Format.md)

Defined in: [modules/bitview-client/index.js:1287](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1287)

***

### index

> **index**: [`Index`](../type-aliases/Index.md)

Defined in: [modules/bitview-client/index.js:1283](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1283)

***

### limit?

> `optional` **limit?**: `number` \| `null`

Defined in: [modules/bitview-client/index.js:1286](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1286)

Maximum number of values to return (ignored if `end` is set); omitted, the rest of the range. REST aliases: `count`, `c`, `l`

***

### series

> **series**: `string`

Defined in: [modules/bitview-client/index.js:1282](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1282)

***

### start?

> `optional` **start?**: [`RangeIndex`](../type-aliases/RangeIndex.md) \| `null`

Defined in: [modules/bitview-client/index.js:1284](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1284)

Inclusive start: integer index, date (YYYY-MM-DD), or timestamp (ISO 8601). Negative integers count from end. REST aliases: `from`, `f`, `s`
