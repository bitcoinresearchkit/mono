[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / DataRangeFormat

# Interface: DataRangeFormat

Defined in: [modules/bitview-client/index.js:459](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L459)

## Properties

### end?

> `optional` **end?**: [`RangeIndex`](../type-aliases/RangeIndex.md) \| `null`

Defined in: [modules/bitview-client/index.js:461](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L461)

Exclusive end: integer index, date (YYYY-MM-DD), or timestamp (ISO 8601). Negative integers count from end. REST aliases: `to`, `t`, `e`

***

### format?

> `optional` **format?**: [`Format`](../type-aliases/Format.md)

Defined in: [modules/bitview-client/index.js:463](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L463)

***

### limit?

> `optional` **limit?**: `number` \| `null`

Defined in: [modules/bitview-client/index.js:462](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L462)

Maximum number of values to return (ignored if `end` is set); omitted, the rest of the range. REST aliases: `count`, `c`, `l`

***

### start?

> `optional` **start?**: [`RangeIndex`](../type-aliases/RangeIndex.md) \| `null`

Defined in: [modules/bitview-client/index.js:460](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L460)

Inclusive start: integer index, date (YYYY-MM-DD), or timestamp (ISO 8601). Negative integers count from end. REST aliases: `from`, `f`, `s`
