[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / BlockPool

# Interface: BlockPool

Defined in: [modules/bitview-client/index.js:225](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L225)

## Properties

### blockNumber

> **blockNumber**: `number`

Defined in: [modules/bitview-client/index.js:229](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L229)

This block's ordinal among blocks attributed to this pool

***

### id

> **id**: `number`

Defined in: [modules/bitview-client/index.js:226](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L226)

Zero-based pool ID, matching mempool.space's `unique_id`

***

### minerNames?

> `optional` **minerNames?**: `string`[] \| `null`

Defined in: [modules/bitview-client/index.js:230](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L230)

Miner name tags found in coinbase scriptsig

***

### name

> **name**: `string`

Defined in: [modules/bitview-client/index.js:227](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L227)

***

### slug

> **slug**: [`PoolSlug`](../type-aliases/PoolSlug.md)

Defined in: [modules/bitview-client/index.js:228](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L228)

URL-friendly pool identifier
