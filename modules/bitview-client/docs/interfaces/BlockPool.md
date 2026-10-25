[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / BlockPool

# Interface: BlockPool

Defined in: [modules/bitview-client/index.js:272](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L272)

## Properties

### blockNumber

> **blockNumber**: `number`

Defined in: [modules/bitview-client/index.js:276](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L276)

This block's ordinal among blocks attributed to this pool

***

### id

> **id**: `number`

Defined in: [modules/bitview-client/index.js:273](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L273)

Zero-based pool ID, matching mempool.space's `unique_id`

***

### minerNames?

> `optional` **minerNames?**: `string`[] \| `null`

Defined in: [modules/bitview-client/index.js:277](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L277)

Miner name tags found in coinbase scriptsig

***

### name

> **name**: `string`

Defined in: [modules/bitview-client/index.js:274](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L274)

***

### slug

> **slug**: [`PoolSlug`](../type-aliases/PoolSlug.md)

Defined in: [modules/bitview-client/index.js:275](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L275)

URL-friendly pool identifier
