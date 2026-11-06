[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / BlockStatus

# Interface: BlockStatus

Defined in: [modules/bitview-client/index.js:255](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L255)

## Properties

### height?

> `optional` **height?**: `number` \| `null`

Defined in: [modules/bitview-client/index.js:257](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L257)

Block height (null unless in the best chain)

***

### inBestChain

> **inBestChain**: `boolean`

Defined in: [modules/bitview-client/index.js:256](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L256)

***

### nextBest?

> `optional` **nextBest?**: `string` \| `null`

Defined in: [modules/bitview-client/index.js:258](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L258)

Hash of the next block in the best chain (null if tip)
