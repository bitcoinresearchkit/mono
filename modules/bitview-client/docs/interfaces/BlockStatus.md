[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / BlockStatus

# Interface: BlockStatus

Defined in: [modules/bitview-client/index.js:297](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L297)

## Properties

### height?

> `optional` **height?**: `number` \| `null`

Defined in: [modules/bitview-client/index.js:299](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L299)

Block height (null unless in the best chain)

***

### inBestChain

> **inBestChain**: `boolean`

Defined in: [modules/bitview-client/index.js:298](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L298)

***

### nextBest?

> `optional` **nextBest?**: `string` \| `null`

Defined in: [modules/bitview-client/index.js:300](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L300)

Hash of the next block in the best chain (null if tip)
