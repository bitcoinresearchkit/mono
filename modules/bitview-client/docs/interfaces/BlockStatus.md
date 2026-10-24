[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / BlockStatus

# Interface: BlockStatus

Defined in: [modules/bitview-client/index.js:282](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L282)

## Properties

### height?

> `optional` **height?**: `number` \| `null`

Defined in: [modules/bitview-client/index.js:284](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L284)

Block height (null unless in the best chain)

***

### inBestChain

> **inBestChain**: `boolean`

Defined in: [modules/bitview-client/index.js:283](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L283)

***

### nextBest?

> `optional` **nextBest?**: `string` \| `null`

Defined in: [modules/bitview-client/index.js:285](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L285)

Hash of the next block in the best chain (null if tip)
