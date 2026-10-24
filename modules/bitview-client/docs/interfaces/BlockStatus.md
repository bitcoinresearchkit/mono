[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / BlockStatus

# Interface: BlockStatus

Defined in: [modules/bitview-client/index.js:312](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L312)

## Properties

### height?

> `optional` **height?**: `number` \| `null`

Defined in: [modules/bitview-client/index.js:314](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L314)

Block height (null unless in the best chain)

***

### inBestChain

> **inBestChain**: `boolean`

Defined in: [modules/bitview-client/index.js:313](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L313)

***

### nextBest?

> `optional` **nextBest?**: `string` \| `null`

Defined in: [modules/bitview-client/index.js:315](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L315)

Hash of the next block in the best chain (null if tip)
