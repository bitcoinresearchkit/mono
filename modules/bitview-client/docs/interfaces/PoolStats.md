[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / PoolStats

# Interface: PoolStats

Defined in: [modules/bitview-client/index.js:1149](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1149)

## Properties

### avgFeeDelta?

> `optional` **avgFeeDelta?**: `string` \| `null`

Defined in: [modules/bitview-client/index.js:1159](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1159)

Mean relative difference of the pool's fees from the expected template's, a decimal
  string at mempool.space, from its own audits (always null: it needs the mempool as it was)

***

### avgMatchRate?

> `optional` **avgMatchRate?**: `number` \| `null`

Defined in: [modules/bitview-client/index.js:1157](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1157)

Mean share of the expected block template the pool's blocks matched, from mempool.space's
  own audits (always null: it needs the mempool as it was)

***

### blockCount

> **blockCount**: `number`

Defined in: [modules/bitview-client/index.js:1153](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1153)

***

### emptyBlocks

> **emptyBlocks**: `number`

Defined in: [modules/bitview-client/index.js:1155](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1155)

Number of empty (coinbase-only) blocks mined in the time period

***

### link

> **link**: `string`

Defined in: [modules/bitview-client/index.js:1152](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1152)

Pool website URL

***

### name

> **name**: `string`

Defined in: [modules/bitview-client/index.js:1151](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1151)

***

### poolId

> **poolId**: `number`

Defined in: [modules/bitview-client/index.js:1150](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1150)

One-based pool ID, matching mempool.space's `id`

***

### poolUniqueId

> **poolUniqueId**: `number`

Defined in: [modules/bitview-client/index.js:1161](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1161)

Zero-based pool ID, matching mempool.space's `unique_id`

***

### rank

> **rank**: `number`

Defined in: [modules/bitview-client/index.js:1154](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1154)

Pool ranking by block count (1 = most blocks)

***

### share

> **share**: `number`

Defined in: [modules/bitview-client/index.js:1162](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1162)

Pool's share of total blocks (0.0 - 1.0)

***

### slug

> **slug**: [`PoolSlug`](../type-aliases/PoolSlug.md)

Defined in: [modules/bitview-client/index.js:1156](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1156)

URL-friendly pool identifier
