[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / MempoolBlockContent

# Interface: MempoolBlockContent

Defined in: [modules/bitview-client/index.js:744](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L744)

## Properties

### hash

> **hash**: `string`

Defined in: [modules/bitview-client/index.js:745](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L745)

Pass as `since` to `GET /api/mempool-blocks/{index}/diff/{since}` to follow changes.

***

### stats

> **stats**: [`MempoolBlock`](MempoolBlock.md)

Defined in: [modules/bitview-client/index.js:746](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L746)

***

### transactions

> **transactions**: [`Transaction`](Transaction.md)[]

Defined in: [modules/bitview-client/index.js:747](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L747)

Full transaction bodies in `getblocktemplate` order.
