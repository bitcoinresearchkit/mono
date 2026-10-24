[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / MempoolBlockContent

# Interface: MempoolBlockContent

Defined in: [modules/bitview-client/index.js:749](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L749)

## Properties

### hash

> **hash**: `string`

Defined in: [modules/bitview-client/index.js:750](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L750)

Pass as `since` to `GET /api/mempool-blocks/{index}/diff/{since}` to follow changes.

***

### stats

> **stats**: [`MempoolBlock`](MempoolBlock.md)

Defined in: [modules/bitview-client/index.js:751](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L751)

***

### transactions

> **transactions**: [`Transaction`](Transaction.md)[]

Defined in: [modules/bitview-client/index.js:752](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L752)

Full transaction bodies in `getblocktemplate` order.
