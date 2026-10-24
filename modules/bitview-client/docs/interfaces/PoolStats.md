[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / PoolStats

# Interface: PoolStats

Defined in: [modules/bitview-client/index.js:1082](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1082)

## Properties

### avgFeeDelta?

> `optional` **avgFeeDelta?**: `string` \| `null`

Defined in: [modules/bitview-client/index.js:1092](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1092)

Mean relative difference of the pool's fees from the expected template's, a decimal
string at mempool.space, from its own audits (always null: it needs the mempool as it was)

***

### avgMatchRate?

> `optional` **avgMatchRate?**: `number` \| `null`

Defined in: [modules/bitview-client/index.js:1090](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1090)

Mean share of the expected block template the pool's blocks matched, from mempool.space's
own audits (always null: it needs the mempool as it was)

***

### blockCount

> **blockCount**: `number`

Defined in: [modules/bitview-client/index.js:1086](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1086)

***

### emptyBlocks

> **emptyBlocks**: `number`

Defined in: [modules/bitview-client/index.js:1088](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1088)

Number of empty (coinbase-only) blocks mined in the time period

***

### link

> **link**: `string`

Defined in: [modules/bitview-client/index.js:1085](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1085)

Pool website URL

***

### name

> **name**: `string`

Defined in: [modules/bitview-client/index.js:1084](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1084)

***

### poolId

> **poolId**: `number`

Defined in: [modules/bitview-client/index.js:1083](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1083)

One-based pool ID, matching mempool.space's `id`

***

### poolUniqueId

> **poolUniqueId**: `number`

Defined in: [modules/bitview-client/index.js:1094](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1094)

Zero-based pool ID, matching mempool.space's `unique_id`

***

### rank

> **rank**: `number`

Defined in: [modules/bitview-client/index.js:1087](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1087)

Pool ranking by block count (1 = most blocks)

***

### share

> **share**: `number`

Defined in: [modules/bitview-client/index.js:1095](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1095)

Pool's share of total blocks (0.0 - 1.0)

***

### slug

> **slug**: [`PoolSlug`](../type-aliases/PoolSlug.md)

Defined in: [modules/bitview-client/index.js:1089](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1089)

URL-friendly pool identifier
