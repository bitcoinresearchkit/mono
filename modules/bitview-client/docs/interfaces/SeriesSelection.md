[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / SeriesSelection

# Interface: SeriesSelection

Defined in: [modules/bitview-client/index.js:1293](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1293)

## Properties

### end?

> `optional` **end?**: [`RangeIndex`](../type-aliases/RangeIndex.md) \| `null`

Defined in: [modules/bitview-client/index.js:1297](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1297)

Exclusive end: integer index, date (YYYY-MM-DD), or timestamp (ISO 8601). Negative integers count from end. REST aliases: `to`, `t`, `e`

***

### format?

> `optional` **format?**: [`Format`](../type-aliases/Format.md)

Defined in: [modules/bitview-client/index.js:1299](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1299)

***

### index

> **index**: [`Index`](../type-aliases/Index.md)

Defined in: [modules/bitview-client/index.js:1295](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1295)

***

### limit?

> `optional` **limit?**: `number` \| `null`

Defined in: [modules/bitview-client/index.js:1298](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1298)

Maximum number of values to return (ignored if `end` is set); omitted, the rest of the range. REST aliases: `count`, `c`, `l`

***

### series

> **series**: `string`

Defined in: [modules/bitview-client/index.js:1294](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1294)

***

### start?

> `optional` **start?**: [`RangeIndex`](../type-aliases/RangeIndex.md) \| `null`

Defined in: [modules/bitview-client/index.js:1296](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1296)

Inclusive start: integer index, date (YYYY-MM-DD), or timestamp (ISO 8601). Negative integers count from end. REST aliases: `from`, `f`, `s`
