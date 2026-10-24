[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / PoolDetail

# Interface: PoolDetail

Defined in: [modules/bitview-client/index.js:1021](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1021)

## Properties

### blockCount

> **blockCount**: [`PoolBlockCounts`](../type-aliases/PoolBlockCounts.md)

Defined in: [modules/bitview-client/index.js:1023](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1023)

***

### blockShare

> **blockShare**: [`PoolBlockShares`](../type-aliases/PoolBlockShares.md)

Defined in: [modules/bitview-client/index.js:1024](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1024)

***

### estimatedHashrate

> **estimatedHashrate**: `number`

Defined in: [modules/bitview-client/index.js:1025](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1025)

Estimated hashrate based on blocks mined (H/s)

***

### pool

> **pool**: [`PoolDetailInfo`](PoolDetailInfo.md)

Defined in: [modules/bitview-client/index.js:1022](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1022)

***

### reportedHashrate?

> `optional` **reportedHashrate?**: `number` \| `null`

Defined in: [modules/bitview-client/index.js:1026](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1026)

Self-reported hashrate (if available, H/s)

***

### totalReward

> **totalReward**: `number`

Defined in: [modules/bitview-client/index.js:1027](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1027)

Total reward earned by this pool: coinbase output value of every block it mined (sats)
