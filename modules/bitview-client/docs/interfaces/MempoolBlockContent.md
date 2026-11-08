[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / MempoolBlockContent

# Interface: MempoolBlockContent

Defined in: [modules/bitview-client/index.js:766](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L766)

## Properties

### hash

> **hash**: `string`

Defined in: [modules/bitview-client/index.js:767](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L767)

Pass as `since` to `GET /api/mempool-blocks/{index}/diff/{since}` to follow changes.

***

### stats

> **stats**: [`MempoolBlock`](MempoolBlock.md)

Defined in: [modules/bitview-client/index.js:768](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L768)

***

### transactions

> **transactions**: [`Transaction`](Transaction.md)[]

Defined in: [modules/bitview-client/index.js:769](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L769)

Full transaction bodies in block order (block 0's: `getblocktemplate`'s).
