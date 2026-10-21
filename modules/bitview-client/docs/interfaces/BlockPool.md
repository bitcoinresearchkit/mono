[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / BlockPool

# Interface: BlockPool

Defined in: [modules/bitview-client/index.js:209](https://github.com/bitcoinresearchkit/mono/blob/35af38a95a9d8eab77d0bc6900ec332bc53accaf/modules/bitview-client/index.js#L209)

## Properties

### blockNumber

> **blockNumber**: `number`

Defined in: [modules/bitview-client/index.js:213](https://github.com/bitcoinresearchkit/mono/blob/35af38a95a9d8eab77d0bc6900ec332bc53accaf/modules/bitview-client/index.js#L213)

This block's ordinal among blocks attributed to this pool

***

### id

> **id**: `number`

Defined in: [modules/bitview-client/index.js:210](https://github.com/bitcoinresearchkit/mono/blob/35af38a95a9d8eab77d0bc6900ec332bc53accaf/modules/bitview-client/index.js#L210)

Zero-based pool ID, matching mempool.space's `unique_id`

***

### minerNames?

> `optional` **minerNames?**: `string`[] \| `null`

Defined in: [modules/bitview-client/index.js:214](https://github.com/bitcoinresearchkit/mono/blob/35af38a95a9d8eab77d0bc6900ec332bc53accaf/modules/bitview-client/index.js#L214)

Miner name tags found in coinbase scriptsig

***

### name

> **name**: `string`

Defined in: [modules/bitview-client/index.js:211](https://github.com/bitcoinresearchkit/mono/blob/35af38a95a9d8eab77d0bc6900ec332bc53accaf/modules/bitview-client/index.js#L211)

***

### slug

> **slug**: [`PoolSlug`](../type-aliases/PoolSlug.md)

Defined in: [modules/bitview-client/index.js:212](https://github.com/bitcoinresearchkit/mono/blob/35af38a95a9d8eab77d0bc6900ec332bc53accaf/modules/bitview-client/index.js#L212)

URL-friendly pool identifier
