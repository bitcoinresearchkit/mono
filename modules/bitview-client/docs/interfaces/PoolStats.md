[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / PoolStats

# Interface: PoolStats

Defined in: [modules/bitview-client/index.js:1161](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1161)

## Properties

### avgFeeDelta?

> `optional` **avgFeeDelta?**: `string` \| `null`

Defined in: [modules/bitview-client/index.js:1171](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1171)

Mean relative difference of the pool's fees from the expected template's, a decimal
string at mempool.space, from its own audits (always null: it needs the mempool as it was)

***

### avgMatchRate?

> `optional` **avgMatchRate?**: `number` \| `null`

Defined in: [modules/bitview-client/index.js:1169](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1169)

Mean share of the expected block template the pool's blocks matched, from mempool.space's
own audits (always null: it needs the mempool as it was)

***

### blockCount

> **blockCount**: `number`

Defined in: [modules/bitview-client/index.js:1165](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1165)

***

### emptyBlocks

> **emptyBlocks**: `number`

Defined in: [modules/bitview-client/index.js:1167](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1167)

Number of empty (coinbase-only) blocks mined in the time period

***

### link

> **link**: `string`

Defined in: [modules/bitview-client/index.js:1164](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1164)

Pool website URL

***

### name

> **name**: `string`

Defined in: [modules/bitview-client/index.js:1163](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1163)

***

### poolId

> **poolId**: `number`

Defined in: [modules/bitview-client/index.js:1162](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1162)

One-based pool ID, matching mempool.space's `id`

***

### poolUniqueId

> **poolUniqueId**: `number`

Defined in: [modules/bitview-client/index.js:1173](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1173)

Zero-based pool ID, matching mempool.space's `unique_id`

***

### rank

> **rank**: `number`

Defined in: [modules/bitview-client/index.js:1166](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1166)

Pool ranking by block count (1 = most blocks)

***

### share

> **share**: `number`

Defined in: [modules/bitview-client/index.js:1174](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1174)

Pool's share of total blocks (0.0 - 1.0)

***

### slug

> **slug**: [`PoolSlug`](../type-aliases/PoolSlug.md)

Defined in: [modules/bitview-client/index.js:1168](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1168)

URL-friendly pool identifier
