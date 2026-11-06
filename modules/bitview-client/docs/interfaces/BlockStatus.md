[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / BlockStatus

# Interface: BlockStatus

Defined in: [modules/bitview-client/index.js:308](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L308)

## Properties

### height?

> `optional` **height?**: `number` \| `null`

Defined in: [modules/bitview-client/index.js:310](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L310)

Block height (null unless in the best chain)

***

### inBestChain

> **inBestChain**: `boolean`

Defined in: [modules/bitview-client/index.js:309](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L309)

***

### nextBest?

> `optional` **nextBest?**: `string` \| `null`

Defined in: [modules/bitview-client/index.js:311](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L311)

Hash of the next block in the best chain (null if tip)
