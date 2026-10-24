[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / SeriesSelection

# Interface: SeriesSelection

Defined in: [modules/bitview-client/index.js:1282](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1282)

## Properties

### end?

> `optional` **end?**: [`RangeIndex`](../type-aliases/RangeIndex.md) \| `null`

Defined in: [modules/bitview-client/index.js:1286](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1286)

Exclusive end: integer index, date (YYYY-MM-DD), or timestamp (ISO 8601). Negative integers count from end.

***

### format?

> `optional` **format?**: [`Format`](../type-aliases/Format.md)

Defined in: [modules/bitview-client/index.js:1288](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1288)

***

### index

> **index**: [`Index`](../type-aliases/Index.md)

Defined in: [modules/bitview-client/index.js:1284](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1284)

***

### limit?

> `optional` **limit?**: `number` \| `null`

Defined in: [modules/bitview-client/index.js:1287](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1287)

Maximum number of values to return (ignored if `end` is set); omitted, the rest of the range.

***

### series

> **series**: `string`

Defined in: [modules/bitview-client/index.js:1283](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1283)

***

### start?

> `optional` **start?**: [`RangeIndex`](../type-aliases/RangeIndex.md) \| `null`

Defined in: [modules/bitview-client/index.js:1285](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1285)

Inclusive start: integer index, date (YYYY-MM-DD), or timestamp (ISO 8601). Negative integers count from end.
