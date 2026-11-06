[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / DataRangeFormat

# Interface: DataRangeFormat

Defined in: [modules/bitview-client/index.js:470](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L470)

## Properties

### end?

> `optional` **end?**: [`RangeIndex`](../type-aliases/RangeIndex.md) \| `null`

Defined in: [modules/bitview-client/index.js:472](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L472)

Exclusive end: integer index, date (YYYY-MM-DD), or timestamp (ISO 8601). Negative integers count from end.

***

### format?

> `optional` **format?**: [`Format`](../type-aliases/Format.md)

Defined in: [modules/bitview-client/index.js:474](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L474)

***

### limit?

> `optional` **limit?**: `number` \| `null`

Defined in: [modules/bitview-client/index.js:473](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L473)

Maximum number of values to return; with `end` too, the tighter of the two applies. Omitted, the rest of the range.

***

### start?

> `optional` **start?**: [`RangeIndex`](../type-aliases/RangeIndex.md) \| `null`

Defined in: [modules/bitview-client/index.js:471](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L471)

Inclusive start: integer index, date (YYYY-MM-DD), or timestamp (ISO 8601). Negative integers count from end.
