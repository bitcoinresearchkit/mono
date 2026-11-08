[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / PoolDetail

# Interface: PoolDetail

Defined in: [modules/bitview-client/index.js:1107](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1107)

## Properties

### avgBlockHealth?

> `optional` **avgBlockHealth?**: `number` \| `null`

Defined in: [modules/bitview-client/index.js:1113](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1113)

Mean health of the pool's blocks, from mempool.space's own audits (always null: it needs
  the mempool as it was)

***

### blockCount

> **blockCount**: [`PoolBlockCounts`](../type-aliases/PoolBlockCounts.md)

Defined in: [modules/bitview-client/index.js:1109](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1109)

***

### blockShare

> **blockShare**: [`PoolBlockShares`](../type-aliases/PoolBlockShares.md)

Defined in: [modules/bitview-client/index.js:1110](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1110)

***

### estimatedHashrate

> **estimatedHashrate**: `number`

Defined in: [modules/bitview-client/index.js:1111](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1111)

Estimated hashrate based on blocks mined (H/s)

***

### pool

> **pool**: [`PoolDetailInfo`](PoolDetailInfo.md)

Defined in: [modules/bitview-client/index.js:1108](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1108)

***

### reportedHashrate?

> `optional` **reportedHashrate?**: `number` \| `null`

Defined in: [modules/bitview-client/index.js:1112](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1112)

Self-reported hashrate (if available, H/s)

***

### totalReward

> **totalReward**: `string`

Defined in: [modules/bitview-client/index.js:1115](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1115)

Total reward earned by this pool: coinbase output value of every block it mined (sats),
  as a decimal string
