[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / BlockStatus

# Interface: BlockStatus

Defined in: [modules/bitview-client/index.js:239](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L239)

## Properties

### height?

> `optional` **height?**: `number` \| `null`

Defined in: [modules/bitview-client/index.js:241](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L241)

Block height (only if in best chain)

***

### inBestChain

> **inBestChain**: `boolean`

Defined in: [modules/bitview-client/index.js:240](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L240)

***

### nextBest?

> `optional` **nextBest?**: `string` \| `null`

Defined in: [modules/bitview-client/index.js:242](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L242)

Hash of the next block in the best chain (null if tip)
