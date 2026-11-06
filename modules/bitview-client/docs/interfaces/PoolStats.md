[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / PoolStats

# Interface: PoolStats

Defined in: [modules/bitview-client/index.js:1189](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1189)

## Properties

### avgFeeDelta?

> `optional` **avgFeeDelta?**: `string` \| `null`

Defined in: [modules/bitview-client/index.js:1199](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1199)

Mean relative difference of the pool's fees from the expected template's, a decimal
  string at mempool.space, from its own audits (always null: it needs the mempool as it was)

***

### avgMatchRate?

> `optional` **avgMatchRate?**: `number` \| `null`

Defined in: [modules/bitview-client/index.js:1197](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1197)

Mean share of the expected block template the pool's blocks matched, from mempool.space's
  own audits (always null: it needs the mempool as it was)

***

### blockCount

> **blockCount**: `number`

Defined in: [modules/bitview-client/index.js:1193](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1193)

***

### emptyBlocks

> **emptyBlocks**: `number`

Defined in: [modules/bitview-client/index.js:1195](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1195)

Number of empty (coinbase-only) blocks mined in the time period

***

### link

> **link**: `string`

Defined in: [modules/bitview-client/index.js:1192](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1192)

Pool website URL

***

### name

> **name**: `string`

Defined in: [modules/bitview-client/index.js:1191](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1191)

***

### poolId

> **poolId**: `number`

Defined in: [modules/bitview-client/index.js:1190](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1190)

One-based pool ID, matching mempool.space's `id`

***

### poolUniqueId

> **poolUniqueId**: `number`

Defined in: [modules/bitview-client/index.js:1201](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1201)

Zero-based pool ID, matching mempool.space's `unique_id`

***

### rank

> **rank**: `number`

Defined in: [modules/bitview-client/index.js:1194](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1194)

Pool ranking by block count (1 = most blocks)

***

### share

> **share**: `number`

Defined in: [modules/bitview-client/index.js:1202](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1202)

Pool's share of total blocks (0.0 - 1.0)

***

### slug

> **slug**: [`PoolSlug`](../type-aliases/PoolSlug.md)

Defined in: [modules/bitview-client/index.js:1196](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1196)

URL-friendly pool identifier
