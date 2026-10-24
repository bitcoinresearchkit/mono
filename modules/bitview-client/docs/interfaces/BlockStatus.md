[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / BlockStatus

# Interface: BlockStatus

Defined in: [modules/bitview-client/index.js:302](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L302)

## Properties

### height?

> `optional` **height?**: `number` \| `null`

Defined in: [modules/bitview-client/index.js:304](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L304)

Block height (null unless in the best chain)

***

### inBestChain

> **inBestChain**: `boolean`

Defined in: [modules/bitview-client/index.js:303](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L303)

***

### nextBest?

> `optional` **nextBest?**: `string` \| `null`

Defined in: [modules/bitview-client/index.js:305](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L305)

Hash of the next block in the best chain (null if tip)
