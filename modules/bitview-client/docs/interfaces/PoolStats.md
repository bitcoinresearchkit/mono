[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / PoolStats

# Interface: PoolStats

Defined in: [modules/bitview-client/index.js:1002](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1002)

## Properties

### blockCount

> **blockCount**: `number`

Defined in: [modules/bitview-client/index.js:1006](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1006)

***

### emptyBlocks

> **emptyBlocks**: `number`

Defined in: [modules/bitview-client/index.js:1008](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1008)

Number of empty (coinbase-only) blocks mined in the time period

***

### link

> **link**: `string`

Defined in: [modules/bitview-client/index.js:1005](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1005)

Pool website URL

***

### name

> **name**: `string`

Defined in: [modules/bitview-client/index.js:1004](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1004)

***

### poolId

> **poolId**: `number`

Defined in: [modules/bitview-client/index.js:1003](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1003)

One-based pool ID, matching mempool.space's `id`

***

### poolUniqueId

> **poolUniqueId**: `number`

Defined in: [modules/bitview-client/index.js:1011](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1011)

Zero-based pool ID, matching mempool.space's `unique_id`

***

### rank

> **rank**: `number`

Defined in: [modules/bitview-client/index.js:1007](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1007)

Pool ranking by block count (1 = most blocks)

***

### share

> **share**: `number`

Defined in: [modules/bitview-client/index.js:1010](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1010)

Pool's share of total blocks (0.0 - 1.0)

***

### slug

> **slug**: [`PoolSlug`](../type-aliases/PoolSlug.md)

Defined in: [modules/bitview-client/index.js:1009](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1009)

URL-friendly pool identifier
