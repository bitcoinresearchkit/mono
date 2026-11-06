[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / MempoolBlockContent

# Interface: MempoolBlockContent

Defined in: [modules/bitview-client/index.js:702](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L702)

## Properties

### hash

> **hash**: `string`

Defined in: [modules/bitview-client/index.js:703](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L703)

Pass as `since` to `GET /api/mempool-blocks/{index}/diff/{since}` to follow changes.

***

### stats

> **stats**: [`MempoolBlock`](MempoolBlock.md)

Defined in: [modules/bitview-client/index.js:704](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L704)

***

### transactions

> **transactions**: [`Transaction`](Transaction.md)[]

Defined in: [modules/bitview-client/index.js:705](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L705)

Full transaction bodies in block order (block 0's: `getblocktemplate`'s).
