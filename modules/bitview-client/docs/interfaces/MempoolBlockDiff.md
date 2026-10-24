[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / MempoolBlockDiff

# Interface: MempoolBlockDiff

Defined in: [modules/bitview-client/index.js:761](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L761)

## Properties

### hash

> **hash**: `string`

Defined in: [modules/bitview-client/index.js:762](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L762)

The block's current hash. Use as `since` on the next diff call.

***

### order

> **order**: [`MempoolBlockDiffEntry`](../type-aliases/MempoolBlockDiffEntry.md)[]

Defined in: [modules/bitview-client/index.js:765](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L765)

The new block in order. Each entry is either a run of the prior copy's transactions,
`[start, count]`, or a full transaction body.

***

### removed

> **removed**: `string`[]

Defined in: [modules/bitview-client/index.js:767](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L767)

Txids that left the projected next block since `since`
(confirmed, evicted, replaced, or pushed past block 0).

***

### since

> **since**: `string`

Defined in: [modules/bitview-client/index.js:763](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L763)

Echoed prior hash the diff was computed against.

***

### stats

> **stats**: [`MempoolBlock`](MempoolBlock.md)

Defined in: [modules/bitview-client/index.js:764](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L764)

Stats of the new block; they can change while every transaction is retained.
