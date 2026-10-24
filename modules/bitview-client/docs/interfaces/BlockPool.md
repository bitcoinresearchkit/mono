[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / BlockPool

# Interface: BlockPool

Defined in: [modules/bitview-client/index.js:267](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L267)

## Properties

### blockNumber

> **blockNumber**: `number`

Defined in: [modules/bitview-client/index.js:271](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L271)

This block's ordinal among blocks attributed to this pool

***

### id

> **id**: `number`

Defined in: [modules/bitview-client/index.js:268](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L268)

Zero-based pool ID, matching mempool.space's `unique_id`

***

### minerNames?

> `optional` **minerNames?**: `string`[] \| `null`

Defined in: [modules/bitview-client/index.js:272](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L272)

Miner name tags found in coinbase scriptsig

***

### name

> **name**: `string`

Defined in: [modules/bitview-client/index.js:269](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L269)

***

### slug

> **slug**: [`PoolSlug`](../type-aliases/PoolSlug.md)

Defined in: [modules/bitview-client/index.js:270](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L270)

URL-friendly pool identifier
