[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / BlockTemplateDiff

# Interface: BlockTemplateDiff

Defined in: [modules/bitview-client/index.js:306](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L306)

## Properties

### hash

> **hash**: `string`

Defined in: [modules/bitview-client/index.js:307](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L307)

Current next-block hash. Use as `since` on the next diff call.

***

### order

> **order**: [`BlockTemplateDiffEntry`](../type-aliases/BlockTemplateDiffEntry.md)[]

Defined in: [modules/bitview-client/index.js:310](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L310)

New template in order. Each entry is either an index into the
prior template's transactions or a full transaction body.

***

### removed

> **removed**: `string`[]

Defined in: [modules/bitview-client/index.js:312](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L312)

Txids that left the projected next block since `since`
(confirmed, evicted, replaced, or pushed past block 0).

***

### since

> **since**: `string`

Defined in: [modules/bitview-client/index.js:308](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L308)

Echoed prior hash the diff was computed against.

***

### stats

> **stats**: [`MempoolBlock`](MempoolBlock.md)

Defined in: [modules/bitview-client/index.js:309](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L309)

Stats of the new template; they can change while every transaction is retained.
