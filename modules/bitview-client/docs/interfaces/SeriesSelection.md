[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / SeriesSelection

# Interface: SeriesSelection

Defined in: [modules/bitview-client/index.js:1259](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1259)

## Properties

### end?

> `optional` **end?**: [`RangeIndex`](../type-aliases/RangeIndex.md) \| `null`

Defined in: [modules/bitview-client/index.js:1263](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1263)

Exclusive end: integer index, date (YYYY-MM-DD), or timestamp (ISO 8601). Negative integers count from end. REST aliases: `to`, `t`, `e`

***

### format?

> `optional` **format?**: [`Format`](../type-aliases/Format.md)

Defined in: [modules/bitview-client/index.js:1265](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1265)

***

### index

> **index**: [`Index`](../type-aliases/Index.md)

Defined in: [modules/bitview-client/index.js:1261](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1261)

***

### limit?

> `optional` **limit?**: `number` \| `null`

Defined in: [modules/bitview-client/index.js:1264](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1264)

Maximum number of values to return (ignored if `end` is set); omitted, the rest of the range. REST aliases: `count`, `c`, `l`

***

### series

> **series**: `string`

Defined in: [modules/bitview-client/index.js:1260](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1260)

***

### start?

> `optional` **start?**: [`RangeIndex`](../type-aliases/RangeIndex.md) \| `null`

Defined in: [modules/bitview-client/index.js:1262](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1262)

Inclusive start: integer index, date (YYYY-MM-DD), or timestamp (ISO 8601). Negative integers count from end. REST aliases: `from`, `f`, `s`
