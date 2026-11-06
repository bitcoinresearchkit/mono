[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / MempoolBlockDiff

# Interface: MempoolBlockDiff

Defined in: [modules/bitview-client/index.js:714](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L714)

## Properties

### hash

> **hash**: `string`

Defined in: [modules/bitview-client/index.js:715](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L715)

The block's current hash. Use as `since` on the next diff call.

***

### order

> **order**: [`MempoolBlockDiffEntry`](../type-aliases/MempoolBlockDiffEntry.md)[]

Defined in: [modules/bitview-client/index.js:718](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L718)

The new block in order. Each entry is either a run of the prior copy's transactions,
  `[start, count]`, or a full transaction body.

***

### removed

> **removed**: `string`[]

Defined in: [modules/bitview-client/index.js:720](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L720)

Txids that left the projected block since `since` (confirmed, evicted, replaced, or moved
  to another block).

***

### since

> **since**: `string`

Defined in: [modules/bitview-client/index.js:716](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L716)

Echoed prior hash the diff was computed against.

***

### stats

> **stats**: [`MempoolBlock`](MempoolBlock.md)

Defined in: [modules/bitview-client/index.js:717](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L717)

Stats of the new block; they can change while every transaction is retained.
