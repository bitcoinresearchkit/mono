[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / MempoolBlockContent

# Interface: MempoolBlockContent

Defined in: [modules/bitview-client/index.js:755](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L755)

## Properties

### hash

> **hash**: `string`

Defined in: [modules/bitview-client/index.js:756](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L756)

Pass as `since` to `GET /api/mempool-blocks/{index}/diff/{since}` to follow changes.

***

### stats

> **stats**: [`MempoolBlock`](MempoolBlock.md)

Defined in: [modules/bitview-client/index.js:757](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L757)

***

### transactions

> **transactions**: [`Transaction`](Transaction.md)[]

Defined in: [modules/bitview-client/index.js:758](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L758)

Full transaction bodies in `getblocktemplate` order.
