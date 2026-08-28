[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / BlockTemplate

# Interface: BlockTemplate

Defined in: [Developer/brk/modules/bitview-client/index.js:279](https://github.com/bitcoinresearchkit/brk/blob/a6ece0db382228669ac4d1a9c673190fe7a4897b/modules/bitview-client/index.js#L279)

## Properties

### hash

> **hash**: `number`

Defined in: [Developer/brk/modules/bitview-client/index.js:280](https://github.com/bitcoinresearchkit/brk/blob/a6ece0db382228669ac4d1a9c673190fe7a4897b/modules/bitview-client/index.js#L280)

Pass to `GET /api/v1/mempool/block-template/diff/{hash}` to fetch deltas.

***

### stats

> **stats**: [`MempoolBlock`](MempoolBlock.md)

Defined in: [Developer/brk/modules/bitview-client/index.js:281](https://github.com/bitcoinresearchkit/brk/blob/a6ece0db382228669ac4d1a9c673190fe7a4897b/modules/bitview-client/index.js#L281)

Aggregate stats for this block (size, vsize, fee range, ...).

***

### transactions

> **transactions**: [`Transaction`](Transaction.md)[]

Defined in: [Developer/brk/modules/bitview-client/index.js:282](https://github.com/bitcoinresearchkit/brk/blob/a6ece0db382228669ac4d1a9c673190fe7a4897b/modules/bitview-client/index.js#L282)

Full transaction bodies in `getblocktemplate` order.
