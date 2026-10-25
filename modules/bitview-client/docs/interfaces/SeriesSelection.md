[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / SeriesSelection

# Interface: SeriesSelection

Defined in: [modules/bitview-client/index.js:1379](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1379)

## Properties

### end?

> `optional` **end?**: [`RangeIndex`](../type-aliases/RangeIndex.md) \| `null`

Defined in: [modules/bitview-client/index.js:1383](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1383)

Exclusive end: integer index, date (YYYY-MM-DD), or timestamp (ISO 8601). Negative integers count from end.

***

### format?

> `optional` **format?**: [`Format`](../type-aliases/Format.md)

Defined in: [modules/bitview-client/index.js:1385](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1385)

***

### index

> **index**: [`Index`](../type-aliases/Index.md)

Defined in: [modules/bitview-client/index.js:1381](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1381)

***

### limit?

> `optional` **limit?**: `number` \| `null`

Defined in: [modules/bitview-client/index.js:1384](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1384)

Maximum number of values to return; with `end` too, the tighter of the two applies. Omitted, the rest of the range.

***

### series

> **series**: `string`

Defined in: [modules/bitview-client/index.js:1380](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1380)

***

### start?

> `optional` **start?**: [`RangeIndex`](../type-aliases/RangeIndex.md) \| `null`

Defined in: [modules/bitview-client/index.js:1382](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1382)

Inclusive start: integer index, date (YYYY-MM-DD), or timestamp (ISO 8601). Negative integers count from end.
