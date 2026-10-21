[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / SeriesSelection

# Interface: SeriesSelection

Defined in: [modules/bitview-client/index.js:1192](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1192)

## Properties

### end?

> `optional` **end?**: [`RangeIndex`](../type-aliases/RangeIndex.md) \| `null`

Defined in: [modules/bitview-client/index.js:1196](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1196)

Exclusive end: integer index, date (YYYY-MM-DD), or timestamp (ISO 8601). Negative integers count from end. REST aliases: `to`, `t`, `e`

***

### format?

> `optional` **format?**: [`Format`](../type-aliases/Format.md)

Defined in: [modules/bitview-client/index.js:1198](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1198)

***

### index

> **index**: [`Index`](../type-aliases/Index.md)

Defined in: [modules/bitview-client/index.js:1194](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1194)

***

### limit?

> `optional` **limit?**: `number` \| `null`

Defined in: [modules/bitview-client/index.js:1197](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1197)

Maximum number of values to return (ignored if `end` is set); omitted, the rest of the range. REST aliases: `count`, `c`, `l`

***

### series

> **series**: `string`

Defined in: [modules/bitview-client/index.js:1193](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1193)

***

### start?

> `optional` **start?**: [`RangeIndex`](../type-aliases/RangeIndex.md) \| `null`

Defined in: [modules/bitview-client/index.js:1195](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1195)

Inclusive start: integer index, date (YYYY-MM-DD), or timestamp (ISO 8601). Negative integers count from end. REST aliases: `from`, `f`, `s`
