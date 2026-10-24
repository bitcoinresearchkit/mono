[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / PoolDetail

# Interface: PoolDetail

Defined in: [modules/bitview-client/index.js:1036](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1036)

## Properties

### avgBlockHealth?

> `optional` **avgBlockHealth?**: `number` \| `null`

Defined in: [modules/bitview-client/index.js:1042](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1042)

Mean health of the pool's blocks, from mempool.space's own audits (always null: it needs
the mempool as it was)

***

### blockCount

> **blockCount**: [`PoolBlockCounts`](../type-aliases/PoolBlockCounts.md)

Defined in: [modules/bitview-client/index.js:1038](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1038)

***

### blockShare

> **blockShare**: [`PoolBlockShares`](../type-aliases/PoolBlockShares.md)

Defined in: [modules/bitview-client/index.js:1039](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1039)

***

### estimatedHashrate

> **estimatedHashrate**: `number`

Defined in: [modules/bitview-client/index.js:1040](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1040)

Estimated hashrate based on blocks mined (H/s)

***

### pool

> **pool**: [`PoolDetailInfo`](PoolDetailInfo.md)

Defined in: [modules/bitview-client/index.js:1037](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1037)

***

### reportedHashrate?

> `optional` **reportedHashrate?**: `number` \| `null`

Defined in: [modules/bitview-client/index.js:1041](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1041)

Self-reported hashrate (if available, H/s)

***

### totalReward

> **totalReward**: `string`

Defined in: [modules/bitview-client/index.js:1044](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1044)

Total reward earned by this pool: coinbase output value of every block it mined (sats),
as a decimal string
