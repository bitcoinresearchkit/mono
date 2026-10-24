[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / BlockTemplate

# Interface: BlockTemplate

Defined in: [modules/bitview-client/index.js:307](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L307)

## Properties

### hash

> **hash**: `string`

Defined in: [modules/bitview-client/index.js:308](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L308)

Pass to `GET /api/v1/mempool/block-template/diff/{hash}` to fetch deltas.

***

### stats

> **stats**: [`MempoolBlock`](MempoolBlock.md)

Defined in: [modules/bitview-client/index.js:309](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L309)

***

### transactions

> **transactions**: [`Transaction`](Transaction.md)[]

Defined in: [modules/bitview-client/index.js:310](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L310)

Full transaction bodies in `getblocktemplate` order.
