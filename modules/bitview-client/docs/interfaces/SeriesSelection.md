[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / SeriesSelection

# Interface: SeriesSelection

Defined in: [modules/bitview-client/index.js:1311](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1311)

## Properties

### end?

> `optional` **end?**: [`RangeIndex`](../type-aliases/RangeIndex.md) \| `null`

Defined in: [modules/bitview-client/index.js:1315](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1315)

Exclusive end: integer index, date (YYYY-MM-DD), or timestamp (ISO 8601). Negative integers count from end.

***

### format?

> `optional` **format?**: [`Format`](../type-aliases/Format.md)

Defined in: [modules/bitview-client/index.js:1317](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1317)

***

### index

> **index**: [`Index`](../type-aliases/Index.md)

Defined in: [modules/bitview-client/index.js:1313](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1313)

***

### limit?

> `optional` **limit?**: `number` \| `null`

Defined in: [modules/bitview-client/index.js:1316](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1316)

Maximum number of values to return; with `end` too, the tighter of the two applies. Omitted, the rest of the range.

***

### series

> **series**: `string`

Defined in: [modules/bitview-client/index.js:1312](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1312)

***

### start?

> `optional` **start?**: [`RangeIndex`](../type-aliases/RangeIndex.md) \| `null`

Defined in: [modules/bitview-client/index.js:1314](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1314)

Inclusive start: integer index, date (YYYY-MM-DD), or timestamp (ISO 8601). Negative integers count from end.
