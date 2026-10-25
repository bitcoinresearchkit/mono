[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / MempoolBlockContent

# Interface: MempoolBlockContent

Defined in: [modules/bitview-client/index.js:798](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L798)

## Properties

### hash

> **hash**: `string`

Defined in: [modules/bitview-client/index.js:799](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L799)

Pass as `since` to `GET /api/mempool-blocks/{index}/diff/{since}` to follow changes.

***

### stats

> **stats**: [`MempoolBlock`](MempoolBlock.md)

Defined in: [modules/bitview-client/index.js:800](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L800)

***

### transactions

> **transactions**: [`Transaction`](Transaction.md)[]

Defined in: [modules/bitview-client/index.js:801](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L801)

Full transaction bodies in `getblocktemplate` order.
