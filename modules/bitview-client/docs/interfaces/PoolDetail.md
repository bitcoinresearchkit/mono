[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / PoolDetail

# Interface: PoolDetail

Defined in: [modules/bitview-client/index.js:1138](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1138)

## Properties

### avgBlockHealth?

> `optional` **avgBlockHealth?**: `number` \| `null`

Defined in: [modules/bitview-client/index.js:1144](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1144)

Mean health of the pool's blocks, from mempool.space's own audits (always null: it needs
  the mempool as it was)

***

### blockCount

> **blockCount**: [`PoolBlockCounts`](../type-aliases/PoolBlockCounts.md)

Defined in: [modules/bitview-client/index.js:1140](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1140)

***

### blockShare

> **blockShare**: [`PoolBlockShares`](../type-aliases/PoolBlockShares.md)

Defined in: [modules/bitview-client/index.js:1141](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1141)

***

### estimatedHashrate

> **estimatedHashrate**: `number`

Defined in: [modules/bitview-client/index.js:1142](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1142)

Estimated hashrate based on blocks mined (H/s)

***

### pool

> **pool**: [`PoolDetailInfo`](PoolDetailInfo.md)

Defined in: [modules/bitview-client/index.js:1139](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1139)

***

### reportedHashrate?

> `optional` **reportedHashrate?**: `number` \| `null`

Defined in: [modules/bitview-client/index.js:1143](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1143)

Self-reported hashrate (if available, H/s)

***

### totalReward

> **totalReward**: `string`

Defined in: [modules/bitview-client/index.js:1146](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1146)

Total reward earned by this pool: coinbase output value of every block it mined (sats),
  as a decimal string
