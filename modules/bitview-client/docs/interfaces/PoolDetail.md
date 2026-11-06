[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / PoolDetail

# Interface: PoolDetail

Defined in: [modules/bitview-client/index.js:1041](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1041)

## Properties

### avgBlockHealth?

> `optional` **avgBlockHealth?**: `number` \| `null`

Defined in: [modules/bitview-client/index.js:1047](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1047)

Mean health of the pool's blocks, from mempool.space's own audits (always null: it needs
  the mempool as it was)

***

### blockCount

> **blockCount**: [`PoolBlockCounts`](../type-aliases/PoolBlockCounts.md)

Defined in: [modules/bitview-client/index.js:1043](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1043)

***

### blockShare

> **blockShare**: [`PoolBlockShares`](../type-aliases/PoolBlockShares.md)

Defined in: [modules/bitview-client/index.js:1044](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1044)

***

### estimatedHashrate

> **estimatedHashrate**: `number`

Defined in: [modules/bitview-client/index.js:1045](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1045)

Estimated hashrate based on blocks mined (H/s)

***

### pool

> **pool**: [`PoolDetailInfo`](PoolDetailInfo.md)

Defined in: [modules/bitview-client/index.js:1042](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1042)

***

### reportedHashrate?

> `optional` **reportedHashrate?**: `number` \| `null`

Defined in: [modules/bitview-client/index.js:1046](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1046)

Self-reported hashrate (if available, H/s)

***

### totalReward

> **totalReward**: `string`

Defined in: [modules/bitview-client/index.js:1049](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1049)

Total reward earned by this pool: coinbase output value of every block it mined (sats),
  as a decimal string
