[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / PoolDetail

# Interface: PoolDetail

Defined in: [modules/bitview-client/index.js:954](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L954)

## Properties

### blockCount

> **blockCount**: [`PoolBlockCounts`](../type-aliases/PoolBlockCounts.md)

Defined in: [modules/bitview-client/index.js:956](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L956)

***

### blockShare

> **blockShare**: [`PoolBlockShares`](../type-aliases/PoolBlockShares.md)

Defined in: [modules/bitview-client/index.js:957](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L957)

***

### estimatedHashrate

> **estimatedHashrate**: `number`

Defined in: [modules/bitview-client/index.js:958](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L958)

Estimated hashrate based on blocks mined (H/s)

***

### pool

> **pool**: [`PoolDetailInfo`](PoolDetailInfo.md)

Defined in: [modules/bitview-client/index.js:955](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L955)

***

### reportedHashrate?

> `optional` **reportedHashrate?**: `number` \| `null`

Defined in: [modules/bitview-client/index.js:959](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L959)

Self-reported hashrate (if available, H/s)

***

### totalReward

> **totalReward**: `number`

Defined in: [modules/bitview-client/index.js:960](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L960)

Total reward earned by this pool: coinbase output value of every block it mined (sats)
