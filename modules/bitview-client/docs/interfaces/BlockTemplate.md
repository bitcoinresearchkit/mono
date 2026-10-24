[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / BlockTemplate

# Interface: BlockTemplate

Defined in: [modules/bitview-client/index.js:292](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L292)

## Properties

### hash

> **hash**: `string`

Defined in: [modules/bitview-client/index.js:293](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L293)

Pass to `GET /api/v1/mempool/block-template/diff/{hash}` to fetch deltas.

***

### stats

> **stats**: [`MempoolBlock`](MempoolBlock.md)

Defined in: [modules/bitview-client/index.js:294](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L294)

***

### transactions

> **transactions**: [`Transaction`](Transaction.md)[]

Defined in: [modules/bitview-client/index.js:295](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L295)

Full transaction bodies in `getblocktemplate` order.
