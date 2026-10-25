[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / MempoolBlockDiff

# Interface: MempoolBlockDiff

Defined in: [modules/bitview-client/index.js:810](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L810)

## Properties

### hash

> **hash**: `string`

Defined in: [modules/bitview-client/index.js:811](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L811)

The block's current hash. Use as `since` on the next diff call.

***

### order

> **order**: [`MempoolBlockDiffEntry`](../type-aliases/MempoolBlockDiffEntry.md)[]

Defined in: [modules/bitview-client/index.js:814](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L814)

The new block in order. Each entry is either a run of the prior copy's transactions,
`[start, count]`, or a full transaction body.

***

### removed

> **removed**: `string`[]

Defined in: [modules/bitview-client/index.js:816](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L816)

Txids that left the projected next block since `since`
(confirmed, evicted, replaced, or pushed past block 0).

***

### since

> **since**: `string`

Defined in: [modules/bitview-client/index.js:812](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L812)

Echoed prior hash the diff was computed against.

***

### stats

> **stats**: [`MempoolBlock`](MempoolBlock.md)

Defined in: [modules/bitview-client/index.js:813](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L813)

Stats of the new block; they can change while every transaction is retained.
