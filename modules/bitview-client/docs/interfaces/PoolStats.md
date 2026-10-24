[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / PoolStats

# Interface: PoolStats

Defined in: [modules/bitview-client/index.js:1069](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1069)

## Properties

### blockCount

> **blockCount**: `number`

Defined in: [modules/bitview-client/index.js:1073](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1073)

***

### emptyBlocks

> **emptyBlocks**: `number`

Defined in: [modules/bitview-client/index.js:1075](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1075)

Number of empty (coinbase-only) blocks mined in the time period

***

### link

> **link**: `string`

Defined in: [modules/bitview-client/index.js:1072](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1072)

Pool website URL

***

### name

> **name**: `string`

Defined in: [modules/bitview-client/index.js:1071](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1071)

***

### poolId

> **poolId**: `number`

Defined in: [modules/bitview-client/index.js:1070](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1070)

One-based pool ID, matching mempool.space's `id`

***

### poolUniqueId

> **poolUniqueId**: `number`

Defined in: [modules/bitview-client/index.js:1078](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1078)

Zero-based pool ID, matching mempool.space's `unique_id`

***

### rank

> **rank**: `number`

Defined in: [modules/bitview-client/index.js:1074](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1074)

Pool ranking by block count (1 = most blocks)

***

### share

> **share**: `number`

Defined in: [modules/bitview-client/index.js:1077](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1077)

Pool's share of total blocks (0.0 - 1.0)

***

### slug

> **slug**: [`PoolSlug`](../type-aliases/PoolSlug.md)

Defined in: [modules/bitview-client/index.js:1076](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1076)

URL-friendly pool identifier
