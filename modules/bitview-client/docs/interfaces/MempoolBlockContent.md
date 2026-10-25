[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / MempoolBlockContent

# Interface: MempoolBlockContent

Defined in: [modules/bitview-client/index.js:773](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L773)

## Properties

### hash

> **hash**: `string`

Defined in: [modules/bitview-client/index.js:774](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L774)

Pass as `since` to `GET /api/mempool-blocks/{index}/diff/{since}` to follow changes.

***

### stats

> **stats**: [`MempoolBlock`](MempoolBlock.md)

Defined in: [modules/bitview-client/index.js:775](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L775)

***

### transactions

> **transactions**: [`Transaction`](Transaction.md)[]

Defined in: [modules/bitview-client/index.js:776](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L776)

Full transaction bodies in `getblocktemplate` order.
