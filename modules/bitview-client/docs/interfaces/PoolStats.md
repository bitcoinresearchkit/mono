[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / PoolStats

# Interface: PoolStats

Defined in: [modules/bitview-client/index.js:1111](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1111)

## Properties

### avgFeeDelta?

> `optional` **avgFeeDelta?**: `string` \| `null`

Defined in: [modules/bitview-client/index.js:1121](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1121)

Mean relative difference of the pool's fees from the expected template's, a decimal
string at mempool.space, from its own audits (always null: it needs the mempool as it was)

***

### avgMatchRate?

> `optional` **avgMatchRate?**: `number` \| `null`

Defined in: [modules/bitview-client/index.js:1119](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1119)

Mean share of the expected block template the pool's blocks matched, from mempool.space's
own audits (always null: it needs the mempool as it was)

***

### blockCount

> **blockCount**: `number`

Defined in: [modules/bitview-client/index.js:1115](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1115)

***

### emptyBlocks

> **emptyBlocks**: `number`

Defined in: [modules/bitview-client/index.js:1117](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1117)

Number of empty (coinbase-only) blocks mined in the time period

***

### link

> **link**: `string`

Defined in: [modules/bitview-client/index.js:1114](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1114)

Pool website URL

***

### name

> **name**: `string`

Defined in: [modules/bitview-client/index.js:1113](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1113)

***

### poolId

> **poolId**: `number`

Defined in: [modules/bitview-client/index.js:1112](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1112)

One-based pool ID, matching mempool.space's `id`

***

### poolUniqueId

> **poolUniqueId**: `number`

Defined in: [modules/bitview-client/index.js:1123](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1123)

Zero-based pool ID, matching mempool.space's `unique_id`

***

### rank

> **rank**: `number`

Defined in: [modules/bitview-client/index.js:1116](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1116)

Pool ranking by block count (1 = most blocks)

***

### share

> **share**: `number`

Defined in: [modules/bitview-client/index.js:1124](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1124)

Pool's share of total blocks (0.0 - 1.0)

***

### slug

> **slug**: [`PoolSlug`](../type-aliases/PoolSlug.md)

Defined in: [modules/bitview-client/index.js:1118](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1118)

URL-friendly pool identifier
