[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / PoolDetail

# Interface: PoolDetail

Defined in: [modules/bitview-client/index.js:1031](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1031)

## Properties

### avgBlockHealth?

> `optional` **avgBlockHealth?**: `number` \| `null`

Defined in: [modules/bitview-client/index.js:1037](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1037)

Mean health of the pool's blocks, from mempool.space's own audits (always null: it needs
the mempool as it was)

***

### blockCount

> **blockCount**: [`PoolBlockCounts`](../type-aliases/PoolBlockCounts.md)

Defined in: [modules/bitview-client/index.js:1033](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1033)

***

### blockShare

> **blockShare**: [`PoolBlockShares`](../type-aliases/PoolBlockShares.md)

Defined in: [modules/bitview-client/index.js:1034](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1034)

***

### estimatedHashrate

> **estimatedHashrate**: `number`

Defined in: [modules/bitview-client/index.js:1035](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1035)

Estimated hashrate based on blocks mined (H/s)

***

### pool

> **pool**: [`PoolDetailInfo`](PoolDetailInfo.md)

Defined in: [modules/bitview-client/index.js:1032](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1032)

***

### reportedHashrate?

> `optional` **reportedHashrate?**: `number` \| `null`

Defined in: [modules/bitview-client/index.js:1036](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1036)

Self-reported hashrate (if available, H/s)

***

### totalReward

> **totalReward**: `string`

Defined in: [modules/bitview-client/index.js:1039](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1039)

Total reward earned by this pool: coinbase output value of every block it mined (sats),
as a decimal string
