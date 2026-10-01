[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / BlockTemplate

# Interface: BlockTemplate

Defined in: [Developer/mono/modules/bitview-client/index.js:287](https://github.com/bitcoinresearchkit/mono/blob/ea62dc574077b34b3299e636a16f81ea96849ead/modules/bitview-client/index.js#L287)

## Properties

### hash

> **hash**: `number`

Defined in: [Developer/mono/modules/bitview-client/index.js:288](https://github.com/bitcoinresearchkit/mono/blob/ea62dc574077b34b3299e636a16f81ea96849ead/modules/bitview-client/index.js#L288)

Pass to `GET /api/v1/mempool/block-template/diff/{hash}` to fetch deltas.

***

### stats

> **stats**: [`MempoolBlock`](MempoolBlock.md)

Defined in: [Developer/mono/modules/bitview-client/index.js:289](https://github.com/bitcoinresearchkit/mono/blob/ea62dc574077b34b3299e636a16f81ea96849ead/modules/bitview-client/index.js#L289)

Aggregate stats for this block (size, vsize, fee range, ...).

***

### transactions

> **transactions**: [`Transaction`](Transaction.md)[]

Defined in: [Developer/mono/modules/bitview-client/index.js:290](https://github.com/bitcoinresearchkit/mono/blob/ea62dc574077b34b3299e636a16f81ea96849ead/modules/bitview-client/index.js#L290)

Full transaction bodies in `getblocktemplate` order.
