[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / PoolStats

# Interface: PoolStats

Defined in: [modules/bitview-client/index.js:1093](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1093)

## Properties

### avgFeeDelta?

> `optional` **avgFeeDelta?**: `string` \| `null`

Defined in: [modules/bitview-client/index.js:1103](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1103)

Mean relative difference of the pool's fees from the expected template's, a decimal
string at mempool.space, from its own audits (always null: it needs the mempool as it was)

***

### avgMatchRate?

> `optional` **avgMatchRate?**: `number` \| `null`

Defined in: [modules/bitview-client/index.js:1101](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1101)

Mean share of the expected block template the pool's blocks matched, from mempool.space's
own audits (always null: it needs the mempool as it was)

***

### blockCount

> **blockCount**: `number`

Defined in: [modules/bitview-client/index.js:1097](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1097)

***

### emptyBlocks

> **emptyBlocks**: `number`

Defined in: [modules/bitview-client/index.js:1099](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1099)

Number of empty (coinbase-only) blocks mined in the time period

***

### link

> **link**: `string`

Defined in: [modules/bitview-client/index.js:1096](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1096)

Pool website URL

***

### name

> **name**: `string`

Defined in: [modules/bitview-client/index.js:1095](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1095)

***

### poolId

> **poolId**: `number`

Defined in: [modules/bitview-client/index.js:1094](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1094)

One-based pool ID, matching mempool.space's `id`

***

### poolUniqueId

> **poolUniqueId**: `number`

Defined in: [modules/bitview-client/index.js:1105](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1105)

Zero-based pool ID, matching mempool.space's `unique_id`

***

### rank

> **rank**: `number`

Defined in: [modules/bitview-client/index.js:1098](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1098)

Pool ranking by block count (1 = most blocks)

***

### share

> **share**: `number`

Defined in: [modules/bitview-client/index.js:1106](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1106)

Pool's share of total blocks (0.0 - 1.0)

***

### slug

> **slug**: [`PoolSlug`](../type-aliases/PoolSlug.md)

Defined in: [modules/bitview-client/index.js:1100](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1100)

URL-friendly pool identifier
