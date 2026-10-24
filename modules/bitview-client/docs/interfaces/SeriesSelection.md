[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / SeriesSelection

# Interface: SeriesSelection

Defined in: [modules/bitview-client/index.js:1287](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1287)

## Properties

### end?

> `optional` **end?**: [`RangeIndex`](../type-aliases/RangeIndex.md) \| `null`

Defined in: [modules/bitview-client/index.js:1291](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1291)

Exclusive end: integer index, date (YYYY-MM-DD), or timestamp (ISO 8601). Negative integers count from end.

***

### format?

> `optional` **format?**: [`Format`](../type-aliases/Format.md)

Defined in: [modules/bitview-client/index.js:1293](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1293)

***

### index

> **index**: [`Index`](../type-aliases/Index.md)

Defined in: [modules/bitview-client/index.js:1289](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1289)

***

### limit?

> `optional` **limit?**: `number` \| `null`

Defined in: [modules/bitview-client/index.js:1292](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1292)

Maximum number of values to return; with `end` too, the tighter of the two applies. Omitted, the rest of the range.

***

### series

> **series**: `string`

Defined in: [modules/bitview-client/index.js:1288](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1288)

***

### start?

> `optional` **start?**: [`RangeIndex`](../type-aliases/RangeIndex.md) \| `null`

Defined in: [modules/bitview-client/index.js:1290](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1290)

Inclusive start: integer index, date (YYYY-MM-DD), or timestamp (ISO 8601). Negative integers count from end.
