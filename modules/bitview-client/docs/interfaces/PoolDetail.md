[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / PoolDetail

# Interface: PoolDetail

Defined in: [modules/bitview-client/index.js:1128](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1128)

## Properties

### avgBlockHealth?

> `optional` **avgBlockHealth?**: `number` \| `null`

Defined in: [modules/bitview-client/index.js:1134](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1134)

Mean health of the pool's blocks, from mempool.space's own audits (always null: it needs
the mempool as it was)

***

### blockCount

> **blockCount**: [`PoolBlockCounts`](../type-aliases/PoolBlockCounts.md)

Defined in: [modules/bitview-client/index.js:1130](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1130)

***

### blockShare

> **blockShare**: [`PoolBlockShares`](../type-aliases/PoolBlockShares.md)

Defined in: [modules/bitview-client/index.js:1131](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1131)

***

### estimatedHashrate

> **estimatedHashrate**: `number`

Defined in: [modules/bitview-client/index.js:1132](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1132)

Estimated hashrate based on blocks mined (H/s)

***

### pool

> **pool**: [`PoolDetailInfo`](PoolDetailInfo.md)

Defined in: [modules/bitview-client/index.js:1129](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1129)

***

### reportedHashrate?

> `optional` **reportedHashrate?**: `number` \| `null`

Defined in: [modules/bitview-client/index.js:1133](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1133)

Self-reported hashrate (if available, H/s)

***

### totalReward

> **totalReward**: `string`

Defined in: [modules/bitview-client/index.js:1136](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1136)

Total reward earned by this pool: coinbase output value of every block it mined (sats),
as a decimal string
