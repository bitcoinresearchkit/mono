[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / PoolStats

# Interface: PoolStats

Defined in: [modules/bitview-client/index.js:1143](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1143)

## Properties

### avgFeeDelta?

> `optional` **avgFeeDelta?**: `string` \| `null`

Defined in: [modules/bitview-client/index.js:1153](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1153)

Mean relative difference of the pool's fees from the expected template's, a decimal
string at mempool.space, from its own audits (always null: it needs the mempool as it was)

***

### avgMatchRate?

> `optional` **avgMatchRate?**: `number` \| `null`

Defined in: [modules/bitview-client/index.js:1151](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1151)

Mean share of the expected block template the pool's blocks matched, from mempool.space's
own audits (always null: it needs the mempool as it was)

***

### blockCount

> **blockCount**: `number`

Defined in: [modules/bitview-client/index.js:1147](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1147)

***

### emptyBlocks

> **emptyBlocks**: `number`

Defined in: [modules/bitview-client/index.js:1149](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1149)

Number of empty (coinbase-only) blocks mined in the time period

***

### link

> **link**: `string`

Defined in: [modules/bitview-client/index.js:1146](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1146)

Pool website URL

***

### name

> **name**: `string`

Defined in: [modules/bitview-client/index.js:1145](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1145)

***

### poolId

> **poolId**: `number`

Defined in: [modules/bitview-client/index.js:1144](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1144)

One-based pool ID, matching mempool.space's `id`

***

### poolUniqueId

> **poolUniqueId**: `number`

Defined in: [modules/bitview-client/index.js:1155](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1155)

Zero-based pool ID, matching mempool.space's `unique_id`

***

### rank

> **rank**: `number`

Defined in: [modules/bitview-client/index.js:1148](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1148)

Pool ranking by block count (1 = most blocks)

***

### share

> **share**: `number`

Defined in: [modules/bitview-client/index.js:1156](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1156)

Pool's share of total blocks (0.0 - 1.0)

***

### slug

> **slug**: [`PoolSlug`](../type-aliases/PoolSlug.md)

Defined in: [modules/bitview-client/index.js:1150](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1150)

URL-friendly pool identifier
