[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / BlockStatus

# Interface: BlockStatus

Defined in: [Developer/brk/modules/bitview-client/index.js:269](https://github.com/bitcoinresearchkit/brk/blob/a6ece0db382228669ac4d1a9c673190fe7a4897b/modules/bitview-client/index.js#L269)

## Properties

### height?

> `optional` **height?**: `number` \| `null`

Defined in: [Developer/brk/modules/bitview-client/index.js:271](https://github.com/bitcoinresearchkit/brk/blob/a6ece0db382228669ac4d1a9c673190fe7a4897b/modules/bitview-client/index.js#L271)

Block height (only if in best chain)

***

### inBestChain

> **inBestChain**: `boolean`

Defined in: [Developer/brk/modules/bitview-client/index.js:270](https://github.com/bitcoinresearchkit/brk/blob/a6ece0db382228669ac4d1a9c673190fe7a4897b/modules/bitview-client/index.js#L270)

Whether this block is in the best chain

***

### nextBest?

> `optional` **nextBest?**: `string` \| `null`

Defined in: [Developer/brk/modules/bitview-client/index.js:272](https://github.com/bitcoinresearchkit/brk/blob/a6ece0db382228669ac4d1a9c673190fe7a4897b/modules/bitview-client/index.js#L272)

Hash of the next block in the best chain (null if tip)
