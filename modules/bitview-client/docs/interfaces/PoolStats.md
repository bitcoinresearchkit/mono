[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / PoolStats

# Interface: PoolStats

Defined in: [modules/bitview-client/index.js:1179](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1179)

## Properties

### avgFeeDelta?

> `optional` **avgFeeDelta?**: `string` \| `null`

Defined in: [modules/bitview-client/index.js:1189](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1189)

Mean relative difference of the pool's fees from the expected template's, a decimal
string at mempool.space, from its own audits (always null: it needs the mempool as it was)

***

### avgMatchRate?

> `optional` **avgMatchRate?**: `number` \| `null`

Defined in: [modules/bitview-client/index.js:1187](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1187)

Mean share of the expected block template the pool's blocks matched, from mempool.space's
own audits (always null: it needs the mempool as it was)

***

### blockCount

> **blockCount**: `number`

Defined in: [modules/bitview-client/index.js:1183](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1183)

***

### emptyBlocks

> **emptyBlocks**: `number`

Defined in: [modules/bitview-client/index.js:1185](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1185)

Number of empty (coinbase-only) blocks mined in the time period

***

### link

> **link**: `string`

Defined in: [modules/bitview-client/index.js:1182](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1182)

Pool website URL

***

### name

> **name**: `string`

Defined in: [modules/bitview-client/index.js:1181](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1181)

***

### poolId

> **poolId**: `number`

Defined in: [modules/bitview-client/index.js:1180](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1180)

One-based pool ID, matching mempool.space's `id`

***

### poolUniqueId

> **poolUniqueId**: `number`

Defined in: [modules/bitview-client/index.js:1191](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1191)

Zero-based pool ID, matching mempool.space's `unique_id`

***

### rank

> **rank**: `number`

Defined in: [modules/bitview-client/index.js:1184](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1184)

Pool ranking by block count (1 = most blocks)

***

### share

> **share**: `number`

Defined in: [modules/bitview-client/index.js:1192](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1192)

Pool's share of total blocks (0.0 - 1.0)

***

### slug

> **slug**: [`PoolSlug`](../type-aliases/PoolSlug.md)

Defined in: [modules/bitview-client/index.js:1186](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1186)

URL-friendly pool identifier
