[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / SeriesSelection

# Interface: SeriesSelection

Defined in: [modules/bitview-client/index.js:1343](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1343)

## Properties

### end?

> `optional` **end?**: [`RangeIndex`](../type-aliases/RangeIndex.md) \| `null`

Defined in: [modules/bitview-client/index.js:1347](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1347)

Exclusive end: integer index, date (YYYY-MM-DD), or timestamp (ISO 8601). Negative integers count from end.

***

### format?

> `optional` **format?**: [`Format`](../type-aliases/Format.md)

Defined in: [modules/bitview-client/index.js:1349](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1349)

***

### index

> **index**: [`Index`](../type-aliases/Index.md)

Defined in: [modules/bitview-client/index.js:1345](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1345)

***

### limit?

> `optional` **limit?**: `number` \| `null`

Defined in: [modules/bitview-client/index.js:1348](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1348)

Maximum number of values to return; with `end` too, the tighter of the two applies. Omitted, the rest of the range.

***

### series

> **series**: `string`

Defined in: [modules/bitview-client/index.js:1344](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1344)

***

### start?

> `optional` **start?**: [`RangeIndex`](../type-aliases/RangeIndex.md) \| `null`

Defined in: [modules/bitview-client/index.js:1346](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1346)

Inclusive start: integer index, date (YYYY-MM-DD), or timestamp (ISO 8601). Negative integers count from end.
