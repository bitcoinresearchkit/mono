[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / BlockTemplate

# Interface: BlockTemplate

Defined in: [modules/bitview-client/index.js:249](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L249)

## Properties

### hash

> **hash**: `string`

Defined in: [modules/bitview-client/index.js:250](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L250)

Pass to `GET /api/v1/mempool/block-template/diff/{hash}` to fetch deltas.

***

### stats

> **stats**: [`MempoolBlock`](MempoolBlock.md)

Defined in: [modules/bitview-client/index.js:251](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L251)

***

### transactions

> **transactions**: [`Transaction`](Transaction.md)[]

Defined in: [modules/bitview-client/index.js:252](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L252)

Full transaction bodies in `getblocktemplate` order.
