[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / MempoolBlockContent

# Interface: MempoolBlockContent

Defined in: [modules/bitview-client/index.js:783](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L783)

## Properties

### hash

> **hash**: `string`

Defined in: [modules/bitview-client/index.js:784](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L784)

Pass as `since` to `GET /api/mempool-blocks/{index}/diff/{since}` to follow changes.

***

### stats

> **stats**: [`MempoolBlock`](MempoolBlock.md)

Defined in: [modules/bitview-client/index.js:785](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L785)

***

### transactions

> **transactions**: [`Transaction`](Transaction.md)[]

Defined in: [modules/bitview-client/index.js:786](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L786)

Full transaction bodies in `getblocktemplate` order.
