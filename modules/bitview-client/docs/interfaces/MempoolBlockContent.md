[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / MempoolBlockContent

# Interface: MempoolBlockContent

Defined in: modules/bitview-client/index.js:775

## Properties

### hash

> **hash**: `string`

Defined in: modules/bitview-client/index.js:776

Pass as `since` to `GET /api/mempool-blocks/{index}/diff/{since}` to follow changes.

***

### stats

> **stats**: [`MempoolBlock`](MempoolBlock.md)

Defined in: modules/bitview-client/index.js:777

***

### transactions

> **transactions**: [`Transaction`](Transaction.md)[]

Defined in: modules/bitview-client/index.js:778

Full transaction bodies in `getblocktemplate` order.
