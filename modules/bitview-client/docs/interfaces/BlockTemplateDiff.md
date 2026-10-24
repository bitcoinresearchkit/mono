[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / BlockTemplateDiff

# Interface: BlockTemplateDiff

Defined in: [modules/bitview-client/index.js:321](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L321)

## Properties

### hash

> **hash**: `string`

Defined in: [modules/bitview-client/index.js:322](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L322)

Current next-block hash. Use as `since` on the next diff call.

***

### order

> **order**: [`BlockTemplateDiffEntry`](../type-aliases/BlockTemplateDiffEntry.md)[]

Defined in: [modules/bitview-client/index.js:325](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L325)

New template in order. Each entry is either an index into the
prior template's transactions or a full transaction body.

***

### removed

> **removed**: `string`[]

Defined in: [modules/bitview-client/index.js:327](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L327)

Txids that left the projected next block since `since`
(confirmed, evicted, replaced, or pushed past block 0).

***

### since

> **since**: `string`

Defined in: [modules/bitview-client/index.js:323](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L323)

Echoed prior hash the diff was computed against.

***

### stats

> **stats**: [`MempoolBlock`](MempoolBlock.md)

Defined in: [modules/bitview-client/index.js:324](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L324)

Stats of the new template; they can change while every transaction is retained.
