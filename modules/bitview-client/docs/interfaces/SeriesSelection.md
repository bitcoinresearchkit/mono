[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / SeriesSelection

# Interface: SeriesSelection

Defined in: [modules/bitview-client/index.js:1389](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1389)

## Properties

### end?

> `optional` **end?**: [`RangeIndex`](../type-aliases/RangeIndex.md) \| `null`

Defined in: [modules/bitview-client/index.js:1393](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1393)

Exclusive end: integer index, date (YYYY-MM-DD), or timestamp (ISO 8601). Negative integers count from end.

***

### format?

> `optional` **format?**: [`Format`](../type-aliases/Format.md)

Defined in: [modules/bitview-client/index.js:1395](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1395)

***

### index

> **index**: [`Index`](../type-aliases/Index.md)

Defined in: [modules/bitview-client/index.js:1391](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1391)

***

### limit?

> `optional` **limit?**: `number` \| `null`

Defined in: [modules/bitview-client/index.js:1394](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1394)

Maximum number of values to return; with `end` too, the tighter of the two applies. Omitted, the rest of the range.

***

### series

> **series**: `string`

Defined in: [modules/bitview-client/index.js:1390](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1390)

***

### start?

> `optional` **start?**: [`RangeIndex`](../type-aliases/RangeIndex.md) \| `null`

Defined in: [modules/bitview-client/index.js:1392](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1392)

Inclusive start: integer index, date (YYYY-MM-DD), or timestamp (ISO 8601). Negative integers count from end.
