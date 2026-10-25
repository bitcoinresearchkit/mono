[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / PoolDetail

# Interface: PoolDetail

Defined in: [modules/bitview-client/index.js:1110](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1110)

## Properties

### avgBlockHealth?

> `optional` **avgBlockHealth?**: `number` \| `null`

Defined in: [modules/bitview-client/index.js:1116](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1116)

Mean health of the pool's blocks, from mempool.space's own audits (always null: it needs
the mempool as it was)

***

### blockCount

> **blockCount**: [`PoolBlockCounts`](../type-aliases/PoolBlockCounts.md)

Defined in: [modules/bitview-client/index.js:1112](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1112)

***

### blockShare

> **blockShare**: [`PoolBlockShares`](../type-aliases/PoolBlockShares.md)

Defined in: [modules/bitview-client/index.js:1113](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1113)

***

### estimatedHashrate

> **estimatedHashrate**: `number`

Defined in: [modules/bitview-client/index.js:1114](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1114)

Estimated hashrate based on blocks mined (H/s)

***

### pool

> **pool**: [`PoolDetailInfo`](PoolDetailInfo.md)

Defined in: [modules/bitview-client/index.js:1111](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1111)

***

### reportedHashrate?

> `optional` **reportedHashrate?**: `number` \| `null`

Defined in: [modules/bitview-client/index.js:1115](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1115)

Self-reported hashrate (if available, H/s)

***

### totalReward

> **totalReward**: `string`

Defined in: [modules/bitview-client/index.js:1118](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1118)

Total reward earned by this pool: coinbase output value of every block it mined (sats),
as a decimal string
