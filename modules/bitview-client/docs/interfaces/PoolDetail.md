[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / PoolDetail

# Interface: PoolDetail

Defined in: [Developer/brk/modules/bitview-client/index.js:933](https://github.com/bitcoinresearchkit/brk/blob/70a3e90b82b397fa1d5ca46c4ed613066ed3d195/modules/bitview-client/index.js#L933)

## Properties

### blockCount

> **blockCount**: [`PoolBlockCounts`](PoolBlockCounts.md)

Defined in: [Developer/brk/modules/bitview-client/index.js:935](https://github.com/bitcoinresearchkit/brk/blob/70a3e90b82b397fa1d5ca46c4ed613066ed3d195/modules/bitview-client/index.js#L935)

Block counts for different time periods

***

### blockShare

> **blockShare**: [`PoolBlockShares`](PoolBlockShares.md)

Defined in: [Developer/brk/modules/bitview-client/index.js:936](https://github.com/bitcoinresearchkit/brk/blob/70a3e90b82b397fa1d5ca46c4ed613066ed3d195/modules/bitview-client/index.js#L936)

Pool's share of total blocks for different time periods

***

### estimatedHashrate

> **estimatedHashrate**: `number`

Defined in: [Developer/brk/modules/bitview-client/index.js:937](https://github.com/bitcoinresearchkit/brk/blob/70a3e90b82b397fa1d5ca46c4ed613066ed3d195/modules/bitview-client/index.js#L937)

Estimated hashrate based on blocks mined (H/s)

***

### pool

> **pool**: [`PoolDetailInfo`](PoolDetailInfo.md)

Defined in: [Developer/brk/modules/bitview-client/index.js:934](https://github.com/bitcoinresearchkit/brk/blob/70a3e90b82b397fa1d5ca46c4ed613066ed3d195/modules/bitview-client/index.js#L934)

Pool information

***

### reportedHashrate?

> `optional` **reportedHashrate?**: `number` \| `null`

Defined in: [Developer/brk/modules/bitview-client/index.js:938](https://github.com/bitcoinresearchkit/brk/blob/70a3e90b82b397fa1d5ca46c4ed613066ed3d195/modules/bitview-client/index.js#L938)

Self-reported hashrate (if available, H/s)

***

### totalReward?

> `optional` **totalReward?**: `number` \| `null`

Defined in: [Developer/brk/modules/bitview-client/index.js:939](https://github.com/bitcoinresearchkit/brk/blob/70a3e90b82b397fa1d5ca46c4ed613066ed3d195/modules/bitview-client/index.js#L939)

Total reward earned by this pool (sats, all time; None for minor pools)
