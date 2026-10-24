[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / PoolDetail

# Interface: PoolDetail

Defined in: [modules/bitview-client/index.js:1092](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1092)

## Properties

### avgBlockHealth?

> `optional` **avgBlockHealth?**: `number` \| `null`

Defined in: [modules/bitview-client/index.js:1098](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1098)

Mean health of the pool's blocks, from mempool.space's own audits (always null: it needs
the mempool as it was)

***

### blockCount

> **blockCount**: [`PoolBlockCounts`](../type-aliases/PoolBlockCounts.md)

Defined in: [modules/bitview-client/index.js:1094](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1094)

***

### blockShare

> **blockShare**: [`PoolBlockShares`](../type-aliases/PoolBlockShares.md)

Defined in: [modules/bitview-client/index.js:1095](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1095)

***

### estimatedHashrate

> **estimatedHashrate**: `number`

Defined in: [modules/bitview-client/index.js:1096](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1096)

Estimated hashrate based on blocks mined (H/s)

***

### pool

> **pool**: [`PoolDetailInfo`](PoolDetailInfo.md)

Defined in: [modules/bitview-client/index.js:1093](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1093)

***

### reportedHashrate?

> `optional` **reportedHashrate?**: `number` \| `null`

Defined in: [modules/bitview-client/index.js:1097](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1097)

Self-reported hashrate (if available, H/s)

***

### totalReward

> **totalReward**: `string`

Defined in: [modules/bitview-client/index.js:1100](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1100)

Total reward earned by this pool: coinbase output value of every block it mined (sats),
as a decimal string
