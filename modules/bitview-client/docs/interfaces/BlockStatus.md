[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / BlockStatus

# Interface: BlockStatus

Defined in: [Developer/brk/modules/bitview-client/index.js:265](https://github.com/bitcoinresearchkit/brk/blob/70a3e90b82b397fa1d5ca46c4ed613066ed3d195/modules/bitview-client/index.js#L265)

## Properties

### height?

> `optional` **height?**: `number` \| `null`

Defined in: [Developer/brk/modules/bitview-client/index.js:267](https://github.com/bitcoinresearchkit/brk/blob/70a3e90b82b397fa1d5ca46c4ed613066ed3d195/modules/bitview-client/index.js#L267)

Block height (only if in best chain)

***

### inBestChain

> **inBestChain**: `boolean`

Defined in: [Developer/brk/modules/bitview-client/index.js:266](https://github.com/bitcoinresearchkit/brk/blob/70a3e90b82b397fa1d5ca46c4ed613066ed3d195/modules/bitview-client/index.js#L266)

Whether this block is in the best chain

***

### nextBest?

> `optional` **nextBest?**: `string` \| `null`

Defined in: [Developer/brk/modules/bitview-client/index.js:268](https://github.com/bitcoinresearchkit/brk/blob/70a3e90b82b397fa1d5ca46c4ed613066ed3d195/modules/bitview-client/index.js#L268)

Hash of the next block in the best chain (null if tip)
