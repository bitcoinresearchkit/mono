[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / MempoolBlockDiff

# Interface: MempoolBlockDiff

Defined in: [modules/bitview-client/index.js:778](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L778)

## Properties

### hash

> **hash**: `string`

Defined in: [modules/bitview-client/index.js:779](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L779)

The block's current hash. Use as `since` on the next diff call.

***

### order

> **order**: [`MempoolBlockDiffEntry`](../type-aliases/MempoolBlockDiffEntry.md)[]

Defined in: [modules/bitview-client/index.js:782](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L782)

The new block in order. Each entry is either a run of the prior copy's transactions,
  `[start, count]`, or a full transaction body.

***

### removed

> **removed**: `string`[]

Defined in: [modules/bitview-client/index.js:784](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L784)

Txids that left the projected block since `since` (confirmed, evicted, replaced, or moved
  to another block).

***

### since

> **since**: `string`

Defined in: [modules/bitview-client/index.js:780](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L780)

Echoed prior hash the diff was computed against.

***

### stats

> **stats**: [`MempoolBlock`](MempoolBlock.md)

Defined in: [modules/bitview-client/index.js:781](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L781)

Stats of the new block; they can change while every transaction is retained.
