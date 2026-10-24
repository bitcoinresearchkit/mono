[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / PoolDetail

# Interface: PoolDetail

Defined in: [modules/bitview-client/index.js:1060](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1060)

## Properties

### avgBlockHealth?

> `optional` **avgBlockHealth?**: `number` \| `null`

Defined in: [modules/bitview-client/index.js:1066](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1066)

Mean health of the pool's blocks, from mempool.space's own audits (always null: it needs
the mempool as it was)

***

### blockCount

> **blockCount**: [`PoolBlockCounts`](../type-aliases/PoolBlockCounts.md)

Defined in: [modules/bitview-client/index.js:1062](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1062)

***

### blockShare

> **blockShare**: [`PoolBlockShares`](../type-aliases/PoolBlockShares.md)

Defined in: [modules/bitview-client/index.js:1063](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1063)

***

### estimatedHashrate

> **estimatedHashrate**: `number`

Defined in: [modules/bitview-client/index.js:1064](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1064)

Estimated hashrate based on blocks mined (H/s)

***

### pool

> **pool**: [`PoolDetailInfo`](PoolDetailInfo.md)

Defined in: [modules/bitview-client/index.js:1061](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1061)

***

### reportedHashrate?

> `optional` **reportedHashrate?**: `number` \| `null`

Defined in: [modules/bitview-client/index.js:1065](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1065)

Self-reported hashrate (if available, H/s)

***

### totalReward

> **totalReward**: `string`

Defined in: [modules/bitview-client/index.js:1068](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1068)

Total reward earned by this pool: coinbase output value of every block it mined (sats),
as a decimal string
