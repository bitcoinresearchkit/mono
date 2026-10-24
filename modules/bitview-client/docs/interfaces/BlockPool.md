[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / BlockPool

# Interface: BlockPool

Defined in: [modules/bitview-client/index.js:282](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L282)

## Properties

### blockNumber

> **blockNumber**: `number`

Defined in: [modules/bitview-client/index.js:286](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L286)

This block's ordinal among blocks attributed to this pool

***

### id

> **id**: `number`

Defined in: [modules/bitview-client/index.js:283](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L283)

Zero-based pool ID, matching mempool.space's `unique_id`

***

### minerNames?

> `optional` **minerNames?**: `string`[] \| `null`

Defined in: [modules/bitview-client/index.js:287](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L287)

Miner name tags found in coinbase scriptsig

***

### name

> **name**: `string`

Defined in: [modules/bitview-client/index.js:284](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L284)

***

### slug

> **slug**: [`PoolSlug`](../type-aliases/PoolSlug.md)

Defined in: [modules/bitview-client/index.js:285](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L285)

URL-friendly pool identifier
