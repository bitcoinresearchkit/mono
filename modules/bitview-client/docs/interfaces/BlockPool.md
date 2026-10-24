[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / BlockPool

# Interface: BlockPool

Defined in: [modules/bitview-client/index.js:252](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L252)

## Properties

### blockNumber

> **blockNumber**: `number`

Defined in: [modules/bitview-client/index.js:256](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L256)

This block's ordinal among blocks attributed to this pool

***

### id

> **id**: `number`

Defined in: [modules/bitview-client/index.js:253](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L253)

Zero-based pool ID, matching mempool.space's `unique_id`

***

### minerNames?

> `optional` **minerNames?**: `string`[] \| `null`

Defined in: [modules/bitview-client/index.js:257](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L257)

Miner name tags found in coinbase scriptsig

***

### name

> **name**: `string`

Defined in: [modules/bitview-client/index.js:254](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L254)

***

### slug

> **slug**: [`PoolSlug`](../type-aliases/PoolSlug.md)

Defined in: [modules/bitview-client/index.js:255](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L255)

URL-friendly pool identifier
