[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / BitviewClient

# Class: BitviewClient

Defined in: [Developer/mono/modules/bitview-client/index.js:15665](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L15665)

Main Bitview client with series tree and API methods

## Extends

- `BitviewClientBase`

## Constructors

### Constructor

> **new BitviewClient**(`options`): `BitviewClient`

Defined in: [Developer/mono/modules/bitview-client/index.js:16890](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L16890)

#### Parameters

##### options

`string` \| [`BitviewClientOptions`](../interfaces/BitviewClientOptions.md)

#### Returns

`BitviewClient`

#### Overrides

`BitviewClientBase.constructor`

## Properties

### \_browserCache

> **\_browserCache**: `Cache` \| `null`

Defined in: [Developer/mono/modules/bitview-client/index.js:2043](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L2043)

#### Inherited from

`BitviewClientBase._browserCache`

***

### \_browserCachePromise

> **\_browserCachePromise**: `Promise`\<`Cache` \| `null`\>

Defined in: [Developer/mono/modules/bitview-client/index.js:2041](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L2041)

#### Inherited from

`BitviewClientBase._browserCachePromise`

***

### \_memCache

> **\_memCache**: `Map`\<`string`, [`_MemEntry`](../type-aliases/MemEntry.md)\<`unknown`\>\>

Defined in: [Developer/mono/modules/bitview-client/index.js:2050](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L2050)

#### Inherited from

`BitviewClientBase._memCache`

## Accessors

### series

#### Get Signature

> **get** **series**(): [`SeriesTree`](../interfaces/SeriesTree.md)

Defined in: [Developer/mono/modules/bitview-client/index.js:16895](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L16895)

##### Returns

[`SeriesTree`](../interfaces/SeriesTree.md)

## Methods

### \_fetchSeriesData()

> **\_fetchSeriesData**\<`T`\>(`path`, `arg?`, `options?`): `Promise`\<[`DateSeriesData`](../type-aliases/DateSeriesData.md)\<`T`\>\>

Defined in: [Developer/mono/modules/bitview-client/index.js:2291](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L2291)

Fetch series data and wrap with helper methods (internal)

#### Type Parameters

##### T

`T`

#### Parameters

##### path

`string`

##### arg?

[`DateSeriesFetchArg`](../type-aliases/DateSeriesFetchArg.md)\<`T`\>

##### options?

[`ClientFetchOptions`](../interfaces/ClientFetchOptions.md)\<[`DateSeriesData`](../type-aliases/DateSeriesData.md)\<`T`\>\>

#### Returns

`Promise`\<[`DateSeriesData`](../type-aliases/DateSeriesData.md)\<`T`\>\>

#### Inherited from

`BitviewClientBase._fetchSeriesData`

***

### \_getCached()

> **\_getCached**\<`T`\>(`path`, `parse`, `options?`): `Promise`\<`T`\>

Defined in: [Developer/mono/modules/bitview-client/index.js:2117](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L2117)

Make a GET request with layered caching.

Contract:
- The returned Promise resolves with the **freshest** value (post-revalidation).
- `onValue` fires once with the freshest value, or twice if a stale snapshot
  could be shown first (stale-while-revalidate). On a 304 there is no second fire.

Layers:
- L1 (memCache): in-memory parsed values keyed by URL+ETag. Lets 304s skip the parse entirely.
- L2 (browserCache): Cache API, survives reload and feeds onValue fast on cold start.

#### Type Parameters

##### T

`T`

#### Parameters

##### path

`string`

##### parse

(`res`) => `Promise`\<`T`\>

Response body reader

##### options?

[`ClientFetchOptions`](../interfaces/ClientFetchOptions.md)\<`T`\> = `{}`

#### Returns

`Promise`\<`T`\>

#### Inherited from

`BitviewClientBase._getCached`

***

### \_memGet()

> **\_memGet**\<`T`\>(`key`): [`_MemEntry`](../type-aliases/MemEntry.md)\<`T`\> \| `undefined`

Defined in: [Developer/mono/modules/bitview-client/index.js:2058](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L2058)

#### Type Parameters

##### T

`T`

#### Parameters

##### key

`string`

#### Returns

[`_MemEntry`](../type-aliases/MemEntry.md)\<`T`\> \| `undefined`

#### Inherited from

`BitviewClientBase._memGet`

***

### \_memSet()

> **\_memSet**(`key`, `etag`, `value`): `void`

Defined in: [Developer/mono/modules/bitview-client/index.js:2072](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L2072)

#### Parameters

##### key

`string`

##### etag

`string` \| `null`

##### value

`unknown`

#### Returns

`void`

#### Inherited from

`BitviewClientBase._memSet`

***

### dateToIndex()

> **dateToIndex**(`index`, `d`): `number`

Defined in: [Developer/mono/modules/bitview-client/index.js:16882](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L16882)

Convert a Date to an index value for date-based indexes.

#### Parameters

##### index

[`Index`](../type-aliases/Index.md)

The index type

##### d

`Date`

The date to convert

#### Returns

`number`

***

### get()

> **get**(`path`, `options?`): `Promise`\<`Response`\>

Defined in: [Developer/mono/modules/bitview-client/index.js:2087](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L2087)

#### Parameters

##### path

`string`

##### options?

###### cache?

`boolean` = `true`

###### signal?

`AbortSignal`

#### Returns

`Promise`\<`Response`\>

#### Inherited from

`BitviewClientBase.get`

***

### getAddress()

> **getAddress**(`address`, `options?`): `Promise`\<[`AddrStats`](../interfaces/AddrStats.md)\>

Defined in: [Developer/mono/modules/bitview-client/index.js:23713](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L23713)

Address information

Retrieve address information including current balance and transaction counts. Supports all standard Bitcoin address types (P2PKH, P2SH, P2WPKH, P2WSH, P2TR).

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-address)*

Endpoint: `GET /api/address/{address}`

#### Parameters

##### address

`string`

##### options?

###### cache?

`boolean`

###### memCache?

`boolean`

###### onValue?

(`value`) => `void`

###### signal?

`AbortSignal`

#### Returns

`Promise`\<[`AddrStats`](../interfaces/AddrStats.md)\>

***

### getAddressConfirmedTxs()

> **getAddressConfirmedTxs**(`address`, `options?`): `Promise`\<[`Transaction`](../interfaces/Transaction.md)[]\>

Defined in: [Developer/mono/modules/bitview-client/index.js:23749](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L23749)

Address confirmed transactions

Get the first 25 confirmed transactions for an address. For pagination, request `GET /api/address/{address}/txs/chain/{after_txid}` with the last returned txid.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-address-transactions-chain)*

Endpoint: `GET /api/address/{address}/txs/chain`

#### Parameters

##### address

`string`

##### options?

###### cache?

`boolean`

###### memCache?

`boolean`

###### onValue?

(`value`) => `void`

###### signal?

`AbortSignal`

#### Returns

`Promise`\<[`Transaction`](../interfaces/Transaction.md)[]\>

***

### getAddressConfirmedTxsAfter()

> **getAddressConfirmedTxsAfter**(`address`, `after_txid`, `options?`): `Promise`\<[`Transaction`](../interfaces/Transaction.md)[]\>

Defined in: [Developer/mono/modules/bitview-client/index.js:23768](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L23768)

Address confirmed transactions (paginated)

Get the next 25 confirmed transactions strictly older than `after_txid` (Esplora-canonical pagination form, matches mempool.space).

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-address-transactions-chain)*

Endpoint: `GET /api/address/{address}/txs/chain/{after_txid}`

#### Parameters

##### address

`string`

##### after\_txid

`string`

Last txid from the previous page (return transactions strictly older than this)

##### options?

###### cache?

`boolean`

###### memCache?

`boolean`

###### onValue?

(`value`) => `void`

###### signal?

`AbortSignal`

#### Returns

`Promise`\<[`Transaction`](../interfaces/Transaction.md)[]\>

***

### getAddressHashPrefixMatches()

> **getAddressHashPrefixMatches**(`addr_type`, `prefix`, `options?`): `Promise`\<[`AddrHashPrefixMatches`](../interfaces/AddrHashPrefixMatches.md)\>

Defined in: [Developer/mono/modules/bitview-client/index.js:23695](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L23695)

Address hash-prefix matches

Find addresses by address type and by the first 1-16 hex nibbles of RapidHash v3 over the raw address payload bytes. Intended for privacy-preserving client-side wallet discovery without sending raw addresses or xpubs. Fetch metadata with `GET /api/address/{address}`.

Endpoint: `GET /api/address/hash-prefix/{addr_type}/{prefix}`

#### Parameters

##### addr\_type

[`OutputType`](../type-aliases/OutputType.md)

##### prefix

`string`

First 1–16 hexadecimal nibbles of the RapidHash v3 hash over the raw
address payload bytes.

##### options?

###### cache?

`boolean`

###### memCache?

`boolean`

###### onValue?

(`value`) => `void`

###### signal?

`AbortSignal`

#### Returns

`Promise`\<[`AddrHashPrefixMatches`](../interfaces/AddrHashPrefixMatches.md)\>

***

### getAddressMempoolTxs()

> **getAddressMempoolTxs**(`address`, `options?`): `Promise`\<[`Transaction`](../interfaces/Transaction.md)[]\>

Defined in: [Developer/mono/modules/bitview-client/index.js:23786](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L23786)

Address mempool transactions

Get unconfirmed transactions for an address from the mempool, newest first (up to 50).

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-address-transactions-mempool)*

Endpoint: `GET /api/address/{address}/txs/mempool`

#### Parameters

##### address

`string`

##### options?

###### cache?

`boolean`

###### memCache?

`boolean`

###### onValue?

(`value`) => `void`

###### signal?

`AbortSignal`

#### Returns

`Promise`\<[`Transaction`](../interfaces/Transaction.md)[]\>

***

### getAddressPayloadHashPrefixMatches()

> **getAddressPayloadHashPrefixMatches**(`addrType`, `payload`, `nibbles`, `options?`): `Promise`\<[`AddrHashPrefixMatches`](../interfaces/AddrHashPrefixMatches.md)\>

Defined in: [Developer/mono/modules/bitview-client/index.js:16917](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L16917)

Fetch address hash-prefix matches from raw address payload bytes.

#### Parameters

##### addrType

[`OutputType`](../type-aliases/OutputType.md)

##### payload

`number`[] \| `ArrayBuffer` \| `Uint8Array`\<`ArrayBufferLike`\> \| `ArrayBufferView`\<`ArrayBufferLike`\>

Raw payload bytes matching addrType length

##### nibbles

`number`

##### options?

###### cache?

`boolean`

###### memCache?

`boolean`

###### onValue?

(`value`) => `void`

###### signal?

`AbortSignal`

#### Returns

`Promise`\<[`AddrHashPrefixMatches`](../interfaces/AddrHashPrefixMatches.md)\>

***

### getAddressTxs()

> **getAddressTxs**(`address`, `options?`): `Promise`\<[`Transaction`](../interfaces/Transaction.md)[]\>

Defined in: [Developer/mono/modules/bitview-client/index.js:23731](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L23731)

Address transactions

Get transaction history for an address, newest first. Returns up to 50 mempool transactions plus a confirmed page sized to fill the response to 50 total (chain floor of 25, so 25-50 confirmed depending on mempool weight). To paginate further confirmed history, request `GET /api/address/{address}/txs/chain/{after_txid}` with the last returned txid.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-address-transactions)*

Endpoint: `GET /api/address/{address}/txs`

#### Parameters

##### address

`string`

##### options?

###### cache?

`boolean`

###### memCache?

`boolean`

###### onValue?

(`value`) => `void`

###### signal?

`AbortSignal`

#### Returns

`Promise`\<[`Transaction`](../interfaces/Transaction.md)[]\>

***

### getAddressUtxos()

> **getAddressUtxos**(`address`, `options?`): `Promise`\<[`Utxo`](../interfaces/Utxo.md)[]\>

Defined in: [Developer/mono/modules/bitview-client/index.js:23804](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L23804)

Address UTXOs

Get unspent transaction outputs (UTXOs) for an address. Returns txid, vout, value, and confirmation status for each UTXO.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-address-utxo)*

Endpoint: `GET /api/address/{address}/utxo`

#### Parameters

##### address

`string`

##### options?

###### cache?

`boolean`

###### memCache?

`boolean`

###### onValue?

(`value`) => `void`

###### signal?

`AbortSignal`

#### Returns

`Promise`\<[`Utxo`](../interfaces/Utxo.md)[]\>

***

### getApi()

> **getApi**(`options?`): `Promise`\<`any`\>

Defined in: [Developer/mono/modules/bitview-client/index.js:24947](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L24947)

Compact OpenAPI specification

Compact OpenAPI specification optimized for LLM consumption. Removes redundant fields while preserving essential API information. The full specification is available at `GET /openapi.json`.

Endpoint: `GET /api.json`

#### Parameters

##### options?

###### cache?

`boolean`

###### memCache?

`boolean`

###### onValue?

(`value`) => `void`

###### signal?

`AbortSignal`

#### Returns

`Promise`\<`any`\>

***

### getBlock()

> **getBlock**(`hash`, `options?`): `Promise`\<[`BlockInfo`](../interfaces/BlockInfo.md)\>

Defined in: [Developer/mono/modules/bitview-client/index.js:23840](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L23840)

Block information

Retrieve block information by block hash. Returns block metadata including height, timestamp, difficulty, size, weight, and transaction count.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-block)*

Endpoint: `GET /api/block/{hash}`

#### Parameters

##### hash

`string`

##### options?

###### cache?

`boolean`

###### memCache?

`boolean`

###### onValue?

(`value`) => `void`

###### signal?

`AbortSignal`

#### Returns

`Promise`\<[`BlockInfo`](../interfaces/BlockInfo.md)\>

***

### getBlockByHeight()

> **getBlockByHeight**(`height`, `options?`): `Promise`\<`string`\>

Defined in: [Developer/mono/modules/bitview-client/index.js:23894](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L23894)

Block hash by height

Retrieve the block hash at a given height. Returns the hash as plain text.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-height)*

Endpoint: `GET /api/block-height/{height}`

#### Parameters

##### height

`number`

##### options?

###### cache?

`boolean`

###### memCache?

`boolean`

###### onValue?

(`value`) => `void`

###### signal?

`AbortSignal`

#### Returns

`Promise`\<`string`\>

***

### getBlockByTimestamp()

> **getBlockByTimestamp**(`timestamp`, `options?`): `Promise`\<[`BlockTimestamp`](../interfaces/BlockTimestamp.md)\>

Defined in: [Developer/mono/modules/bitview-client/index.js:23912](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L23912)

Block by timestamp

Find the block closest to a given UNIX timestamp.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-timestamp)*

Endpoint: `GET /api/v1/mining/blocks/timestamp/{timestamp}`

#### Parameters

##### timestamp

`number`

##### options?

###### cache?

`boolean`

###### memCache?

`boolean`

###### onValue?

(`value`) => `void`

###### signal?

`AbortSignal`

#### Returns

`Promise`\<[`BlockTimestamp`](../interfaces/BlockTimestamp.md)\>

***

### getBlockFeeRates()

> **getBlockFeeRates**(`time_period`, `options?`): `Promise`\<[`BlockFeeRatesEntry`](../interfaces/BlockFeeRatesEntry.md)[]\>

Defined in: [Developer/mono/modules/bitview-client/index.js:24403](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L24403)

Block fee rates

Get block fee rate percentiles (min, 10th, 25th, median, 75th, 90th, max) for a time period. Valid periods: `24h`, `3d`, `1w`, `1m`, `3m`, `6m`, `1y`, `2y`, `3y`.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-feerates)*

Endpoint: `GET /api/v1/mining/blocks/fee-rates/{time_period}`

#### Parameters

##### time\_period

[`TimePeriod`](../type-aliases/TimePeriod.md)

##### options?

###### cache?

`boolean`

###### memCache?

`boolean`

###### onValue?

(`value`) => `void`

###### signal?

`AbortSignal`

#### Returns

`Promise`\<[`BlockFeeRatesEntry`](../interfaces/BlockFeeRatesEntry.md)[]\>

***

### getBlockFees()

> **getBlockFees**(`time_period`, `options?`): `Promise`\<[`BlockFeesEntry`](../interfaces/BlockFeesEntry.md)[]\>

Defined in: [Developer/mono/modules/bitview-client/index.js:24367](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L24367)

Block fees

Get average total fees per block for a time period. Valid periods: `24h`, `3d`, `1w`, `1m`, `3m`, `6m`, `1y`, `2y`, `3y`.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-fees)*

Endpoint: `GET /api/v1/mining/blocks/fees/{time_period}`

#### Parameters

##### time\_period

[`TimePeriod`](../type-aliases/TimePeriod.md)

##### options?

###### cache?

`boolean`

###### memCache?

`boolean`

###### onValue?

(`value`) => `void`

###### signal?

`AbortSignal`

#### Returns

`Promise`\<[`BlockFeesEntry`](../interfaces/BlockFeesEntry.md)[]\>

***

### getBlockHeader()

> **getBlockHeader**(`hash`, `options?`): `Promise`\<`string`\>

Defined in: [Developer/mono/modules/bitview-client/index.js:23876](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L23876)

Block header

Returns the hex-encoded 80-byte block header.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-header)*

Endpoint: `GET /api/block/{hash}/header`

#### Parameters

##### hash

`string`

##### options?

###### cache?

`boolean`

###### memCache?

`boolean`

###### onValue?

(`value`) => `void`

###### signal?

`AbortSignal`

#### Returns

`Promise`\<`string`\>

***

### getBlockRaw()

> **getBlockRaw**(`hash`, `options?`): `Promise`\<`Uint8Array`\<`ArrayBufferLike`\>\>

Defined in: [Developer/mono/modules/bitview-client/index.js:23930](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L23930)

Raw block

Returns the raw block data in binary format.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-raw)*

Endpoint: `GET /api/block/{hash}/raw`

#### Parameters

##### hash

`string`

##### options?

###### cache?

`boolean`

###### memCache?

`boolean`

###### onValue?

(`value`) => `void`

###### signal?

`AbortSignal`

#### Returns

`Promise`\<`Uint8Array`\<`ArrayBufferLike`\>\>

***

### getBlockRewards()

> **getBlockRewards**(`time_period`, `options?`): `Promise`\<[`BlockRewardsEntry`](../interfaces/BlockRewardsEntry.md)[]\>

Defined in: [Developer/mono/modules/bitview-client/index.js:24385](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L24385)

Block rewards

Get average coinbase reward (subsidy + fees) per block for a time period. Valid periods: `24h`, `3d`, `1w`, `1m`, `3m`, `6m`, `1y`, `2y`, `3y`.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-rewards)*

Endpoint: `GET /api/v1/mining/blocks/rewards/{time_period}`

#### Parameters

##### time\_period

[`TimePeriod`](../type-aliases/TimePeriod.md)

##### options?

###### cache?

`boolean`

###### memCache?

`boolean`

###### onValue?

(`value`) => `void`

###### signal?

`AbortSignal`

#### Returns

`Promise`\<[`BlockRewardsEntry`](../interfaces/BlockRewardsEntry.md)[]\>

***

### getBlocks()

> **getBlocks**(`options?`): `Promise`\<[`BlockInfo`](../interfaces/BlockInfo.md)[]\>

Defined in: [Developer/mono/modules/bitview-client/index.js:24070](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L24070)

Recent blocks

Retrieve the last 10 blocks. Returns block metadata for each block.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-blocks)*

Endpoint: `GET /api/blocks`

#### Parameters

##### options?

###### cache?

`boolean`

###### memCache?

`boolean`

###### onValue?

(`value`) => `void`

###### signal?

`AbortSignal`

#### Returns

`Promise`\<[`BlockInfo`](../interfaces/BlockInfo.md)[]\>

***

### getBlocksFromHeight()

> **getBlocksFromHeight**(`height`, `options?`): `Promise`\<[`BlockInfo`](../interfaces/BlockInfo.md)[]\>

Defined in: [Developer/mono/modules/bitview-client/index.js:24088](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L24088)

Blocks from height

Retrieve up to 10 blocks going backwards from the given height. For example, height=100 returns blocks 100, 99, 98, ..., 91. Height=0 returns only block 0.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-blocks)*

Endpoint: `GET /api/blocks/{height}`

#### Parameters

##### height

`number`

##### options?

###### cache?

`boolean`

###### memCache?

`boolean`

###### onValue?

(`value`) => `void`

###### signal?

`AbortSignal`

#### Returns

`Promise`\<[`BlockInfo`](../interfaces/BlockInfo.md)[]\>

***

### getBlockSizesWeights()

> **getBlockSizesWeights**(`time_period`, `options?`): `Promise`\<[`BlockSizesWeights`](../interfaces/BlockSizesWeights.md)\>

Defined in: [Developer/mono/modules/bitview-client/index.js:24421](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L24421)

Block sizes and weights

Get average block sizes and weights for a time period. Valid periods: `24h`, `3d`, `1w`, `1m`, `3m`, `6m`, `1y`, `2y`, `3y`.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-sizes-weights)*

Endpoint: `GET /api/v1/mining/blocks/sizes-weights/{time_period}`

#### Parameters

##### time\_period

[`TimePeriod`](../type-aliases/TimePeriod.md)

##### options?

###### cache?

`boolean`

###### memCache?

`boolean`

###### onValue?

(`value`) => `void`

###### signal?

`AbortSignal`

#### Returns

`Promise`\<[`BlockSizesWeights`](../interfaces/BlockSizesWeights.md)\>

***

### getBlockStatus()

> **getBlockStatus**(`hash`, `options?`): `Promise`\<[`BlockStatus`](../interfaces/BlockStatus.md)\>

Defined in: [Developer/mono/modules/bitview-client/index.js:23948](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L23948)

Block status

Retrieve the status of a block. Returns whether the block is in the best chain and, if so, its height and the hash of the next block.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-status)*

Endpoint: `GET /api/block/{hash}/status`

#### Parameters

##### hash

`string`

##### options?

###### cache?

`boolean`

###### memCache?

`boolean`

###### onValue?

(`value`) => `void`

###### signal?

`AbortSignal`

#### Returns

`Promise`\<[`BlockStatus`](../interfaces/BlockStatus.md)\>

***

### getBlocksV1()

> **getBlocksV1**(`options?`): `Promise`\<[`BlockInfoV1`](../interfaces/BlockInfoV1.md)[]\>

Defined in: [Developer/mono/modules/bitview-client/index.js:24104](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L24104)

Recent blocks with extras

Retrieve the last 15 blocks with extended data including pool identification and fee statistics.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-blocks-v1)*

Endpoint: `GET /api/v1/blocks`

#### Parameters

##### options?

###### cache?

`boolean`

###### memCache?

`boolean`

###### onValue?

(`value`) => `void`

###### signal?

`AbortSignal`

#### Returns

`Promise`\<[`BlockInfoV1`](../interfaces/BlockInfoV1.md)[]\>

***

### getBlocksV1FromHeight()

> **getBlocksV1FromHeight**(`height`, `options?`): `Promise`\<[`BlockInfoV1`](../interfaces/BlockInfoV1.md)[]\>

Defined in: [Developer/mono/modules/bitview-client/index.js:24122](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L24122)

Blocks from height with extras

Retrieve up to 15 blocks with extended data going backwards from the given height.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-blocks-v1)*

Endpoint: `GET /api/v1/blocks/{height}`

#### Parameters

##### height

`number`

##### options?

###### cache?

`boolean`

###### memCache?

`boolean`

###### onValue?

(`value`) => `void`

###### signal?

`AbortSignal`

#### Returns

`Promise`\<[`BlockInfoV1`](../interfaces/BlockInfoV1.md)[]\>

***

### getBlockTemplate()

> **getBlockTemplate**(`options?`): `Promise`\<[`BlockTemplate`](../interfaces/BlockTemplate.md)\>

Defined in: [Developer/mono/modules/bitview-client/index.js:24577](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L24577)

Projected next block template

Bitcoin Core's `getblocktemplate` selection: full transaction bodies in GBT order with aggregate stats. The returned `hash` is an opaque content token; pass it to `GET /api/v1/mempool/block-template/diff/{hash}` to fetch deltas instead of refetching the whole template.

Endpoint: `GET /api/v1/mempool/block-template`

#### Parameters

##### options?

###### cache?

`boolean`

###### memCache?

`boolean`

###### onValue?

(`value`) => `void`

###### signal?

`AbortSignal`

#### Returns

`Promise`\<[`BlockTemplate`](../interfaces/BlockTemplate.md)\>

***

### getBlockTemplateDiff()

> **getBlockTemplateDiff**(`hash`, `options?`): `Promise`\<[`BlockTemplateDiff`](../interfaces/BlockTemplateDiff.md)\>

Defined in: [Developer/mono/modules/bitview-client/index.js:24593](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L24593)

Block template diff since hash

Delta of the projected next block since `<hash>`. `order` is the full new template in order: each entry is either a number (index into the prior template the client cached at `<hash>`) or a transaction object (new body to insert at this position). Walk `order` once to rebuild; `removed` is a convenience list of txids that left so clients can evict cached bodies. After applying, use the response `hash` as `<hash>` on the next call to keep iterating. Returns `404` when `<hash>` has aged out of server history; clients should fall back to `GET /api/v1/mempool/block-template`.

Endpoint: `GET /api/v1/mempool/block-template/diff/{hash}`

#### Parameters

##### hash

`number`

##### options?

###### cache?

`boolean`

###### memCache?

`boolean`

###### onValue?

(`value`) => `void`

###### signal?

`AbortSignal`

#### Returns

`Promise`\<[`BlockTemplateDiff`](../interfaces/BlockTemplateDiff.md)\>

***

### getBlockTipHash()

> **getBlockTipHash**(`options?`): `Promise`\<`string`\>

Defined in: [Developer/mono/modules/bitview-client/index.js:23980](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L23980)

Block tip hash

Returns the hash of the last block.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-tip-hash)*

Endpoint: `GET /api/blocks/tip/hash`

#### Parameters

##### options?

###### cache?

`boolean`

###### memCache?

`boolean`

###### onValue?

(`value`) => `void`

###### signal?

`AbortSignal`

#### Returns

`Promise`\<`string`\>

***

### getBlockTipHeight()

> **getBlockTipHeight**(`options?`): `Promise`\<`number`\>

Defined in: [Developer/mono/modules/bitview-client/index.js:23964](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L23964)

Block tip height

Returns the height of the last block.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-tip-height)*

Endpoint: `GET /api/blocks/tip/height`

#### Parameters

##### options?

###### cache?

`boolean`

###### memCache?

`boolean`

###### onValue?

(`value`) => `void`

###### signal?

`AbortSignal`

#### Returns

`Promise`\<`number`\>

***

### getBlockTxid()

> **getBlockTxid**(`hash`, `index`, `options?`): `Promise`\<`string`\>

Defined in: [Developer/mono/modules/bitview-client/index.js:23999](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L23999)

Transaction ID at index

Retrieve a single transaction ID at a specific index within a block. Returns plain text txid.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-transaction-id)*

Endpoint: `GET /api/block/{hash}/txid/{index}`

#### Parameters

##### hash

`string`

Bitcoin block hash

##### index

`number`

Transaction index within the block (0-based)

##### options?

###### cache?

`boolean`

###### memCache?

`boolean`

###### onValue?

(`value`) => `void`

###### signal?

`AbortSignal`

#### Returns

`Promise`\<`string`\>

***

### getBlockTxids()

> **getBlockTxids**(`hash`, `options?`): `Promise`\<`string`[]\>

Defined in: [Developer/mono/modules/bitview-client/index.js:24017](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L24017)

Block transaction IDs

Retrieve all transaction IDs in a block. Returns an array of txids in block order.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-transaction-ids)*

Endpoint: `GET /api/block/{hash}/txids`

#### Parameters

##### hash

`string`

##### options?

###### cache?

`boolean`

###### memCache?

`boolean`

###### onValue?

(`value`) => `void`

###### signal?

`AbortSignal`

#### Returns

`Promise`\<`string`[]\>

***

### getBlockTxs()

> **getBlockTxs**(`hash`, `options?`): `Promise`\<[`Transaction`](../interfaces/Transaction.md)[]\>

Defined in: [Developer/mono/modules/bitview-client/index.js:24035](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L24035)

Block transactions

Retrieve transactions in a block by block hash. Returns up to 25 transactions starting from index 0.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-transactions)*

Endpoint: `GET /api/block/{hash}/txs`

#### Parameters

##### hash

`string`

##### options?

###### cache?

`boolean`

###### memCache?

`boolean`

###### onValue?

(`value`) => `void`

###### signal?

`AbortSignal`

#### Returns

`Promise`\<[`Transaction`](../interfaces/Transaction.md)[]\>

***

### getBlockTxsFromIndex()

> **getBlockTxsFromIndex**(`hash`, `start_index`, `options?`): `Promise`\<[`Transaction`](../interfaces/Transaction.md)[]\>

Defined in: [Developer/mono/modules/bitview-client/index.js:24054](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L24054)

Block transactions (paginated)

Retrieve transactions in a block by block hash, starting from the specified index. Returns up to 25 transactions at a time.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-transactions)*

Endpoint: `GET /api/block/{hash}/txs/{start_index}`

#### Parameters

##### hash

`string`

Bitcoin block hash

##### start\_index

`number`

Starting transaction index within the block (0-based)

##### options?

###### cache?

`boolean`

###### memCache?

`boolean`

###### onValue?

(`value`) => `void`

###### signal?

`AbortSignal`

#### Returns

`Promise`\<[`Transaction`](../interfaces/Transaction.md)[]\>

***

### getBlockV1()

> **getBlockV1**(`hash`, `options?`): `Promise`\<[`BlockInfoV1`](../interfaces/BlockInfoV1.md)\>

Defined in: [Developer/mono/modules/bitview-client/index.js:23858](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L23858)

Block (v1)

Returns block details with extras by hash.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-v1)*

Endpoint: `GET /api/v1/block/{hash}`

#### Parameters

##### hash

`string`

##### options?

###### cache?

`boolean`

###### memCache?

`boolean`

###### onValue?

(`value`) => `void`

###### signal?

`AbortSignal`

#### Returns

`Promise`\<[`BlockInfoV1`](../interfaces/BlockInfoV1.md)\>

***

### getBytes()

> **getBytes**(`path`, `options?`): `Promise`\<`Uint8Array`\<`ArrayBufferLike`\>\>

Defined in: [Developer/mono/modules/bitview-client/index.js:2218](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L2218)

Make a GET request expecting binary data (application/octet-stream).
Cached and supports `onValue`, same as `getJson`.

#### Parameters

##### path

`string`

##### options?

[`ClientFetchOptions`](../interfaces/ClientFetchOptions.md)\<`Uint8Array`\<`ArrayBufferLike`\>\>

#### Returns

`Promise`\<`Uint8Array`\<`ArrayBufferLike`\>\>

#### Inherited from

`BitviewClientBase.getBytes`

***

### getCpfp()

> **getCpfp**(`txid`, `options?`): `Promise`\<[`CpfpInfo`](../interfaces/CpfpInfo.md)\>

Defined in: [Developer/mono/modules/bitview-client/index.js:24641](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L24641)

CPFP info

Returns ancestors and descendants for a CPFP (Child Pays For Parent) transaction, including the effective fee rate of the package.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-children-pay-for-parent)*

Endpoint: `GET /api/v1/cpfp/{txid}`

#### Parameters

##### txid

`string`

##### options?

###### cache?

`boolean`

###### memCache?

`boolean`

###### onValue?

(`value`) => `void`

###### signal?

`AbortSignal`

#### Returns

`Promise`\<[`CpfpInfo`](../interfaces/CpfpInfo.md)\>

***

### getDifficultyAdjustment()

> **getDifficultyAdjustment**(`options?`): `Promise`\<[`DifficultyAdjustment`](../interfaces/DifficultyAdjustment.md)\>

Defined in: [Developer/mono/modules/bitview-client/index.js:23640](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L23640)

Difficulty adjustment

Get current difficulty adjustment progress and estimates.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-difficulty-adjustment)*

Endpoint: `GET /api/v1/difficulty-adjustment`

#### Parameters

##### options?

###### cache?

`boolean`

###### memCache?

`boolean`

###### onValue?

(`value`) => `void`

###### signal?

`AbortSignal`

#### Returns

`Promise`\<[`DifficultyAdjustment`](../interfaces/DifficultyAdjustment.md)\>

***

### getDifficultyAdjustments()

> **getDifficultyAdjustments**(`options?`): `Promise`\<[`DifficultyAdjustmentEntry`](../type-aliases/DifficultyAdjustmentEntry.md)[]\>

Defined in: [Developer/mono/modules/bitview-client/index.js:24313](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L24313)

Difficulty adjustments (all time)

Get historical difficulty adjustments including timestamp, block height, difficulty value, and percentage change.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-difficulty-adjustments)*

Endpoint: `GET /api/v1/mining/difficulty-adjustments`

#### Parameters

##### options?

###### cache?

`boolean`

###### memCache?

`boolean`

###### onValue?

(`value`) => `void`

###### signal?

`AbortSignal`

#### Returns

`Promise`\<[`DifficultyAdjustmentEntry`](../type-aliases/DifficultyAdjustmentEntry.md)[]\>

***

### getDifficultyAdjustmentsByPeriod()

> **getDifficultyAdjustmentsByPeriod**(`time_period`, `options?`): `Promise`\<[`DifficultyAdjustmentEntry`](../type-aliases/DifficultyAdjustmentEntry.md)[]\>

Defined in: [Developer/mono/modules/bitview-client/index.js:24331](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L24331)

Difficulty adjustments

Get historical difficulty adjustments for a time period. Valid periods: `24h`, `3d`, `1w`, `1m`, `3m`, `6m`, `1y`, `2y`, `3y`.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-difficulty-adjustments)*

Endpoint: `GET /api/v1/mining/difficulty-adjustments/{time_period}`

#### Parameters

##### time\_period

[`TimePeriod`](../type-aliases/TimePeriod.md)

##### options?

###### cache?

`boolean`

###### memCache?

`boolean`

###### onValue?

(`value`) => `void`

###### signal?

`AbortSignal`

#### Returns

`Promise`\<[`DifficultyAdjustmentEntry`](../type-aliases/DifficultyAdjustmentEntry.md)[]\>

***

### getDiskUsage()

> **getDiskUsage**(`options?`): `Promise`\<[`DiskUsage`](../interfaces/DiskUsage.md)\>

Defined in: [Developer/mono/modules/bitview-client/index.js:23308](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L23308)

Disk usage

Returns the disk space used by BRK and Bitcoin data.

Endpoint: `GET /api/server/disk`

#### Parameters

##### options?

###### cache?

`boolean`

###### memCache?

`boolean`

###### onValue?

(`value`) => `void`

###### signal?

`AbortSignal`

#### Returns

`Promise`\<[`DiskUsage`](../interfaces/DiskUsage.md)\>

***

### getFullrbfReplacements()

> **getFullrbfReplacements**(`options?`): `Promise`\<[`ReplacementNode`](../interfaces/ReplacementNode.md)[]\>

Defined in: [Developer/mono/modules/bitview-client/index.js:24563](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L24563)

Recent full-RBF replacements

Same response shape as `GET /api/v1/replacements`, but limited to trees where at least one predecessor was non-signaling (full-RBF).

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-fullrbf-replacements)*

Endpoint: `GET /api/v1/fullrbf/replacements`

#### Parameters

##### options?

###### cache?

`boolean`

###### memCache?

`boolean`

###### onValue?

(`value`) => `void`

###### signal?

`AbortSignal`

#### Returns

`Promise`\<[`ReplacementNode`](../interfaces/ReplacementNode.md)[]\>

***

### getHashrate()

> **getHashrate**(`options?`): `Promise`\<[`HashrateSummary`](../interfaces/HashrateSummary.md)\>

Defined in: [Developer/mono/modules/bitview-client/index.js:24279](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L24279)

Network hashrate (all time)

Get network hashrate and difficulty data for all time.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-hashrate)*

Endpoint: `GET /api/v1/mining/hashrate`

#### Parameters

##### options?

###### cache?

`boolean`

###### memCache?

`boolean`

###### onValue?

(`value`) => `void`

###### signal?

`AbortSignal`

#### Returns

`Promise`\<[`HashrateSummary`](../interfaces/HashrateSummary.md)\>

***

### getHashrateByPeriod()

> **getHashrateByPeriod**(`time_period`, `options?`): `Promise`\<[`HashrateSummary`](../interfaces/HashrateSummary.md)\>

Defined in: [Developer/mono/modules/bitview-client/index.js:24297](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L24297)

Network hashrate

Get network hashrate and difficulty data for a time period. Valid periods: `24h`, `3d`, `1w`, `1m`, `3m`, `6m`, `1y`, `2y`, `3y`.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-hashrate)*

Endpoint: `GET /api/v1/mining/hashrate/{time_period}`

#### Parameters

##### time\_period

[`TimePeriod`](../type-aliases/TimePeriod.md)

##### options?

###### cache?

`boolean`

###### memCache?

`boolean`

###### onValue?

(`value`) => `void`

###### signal?

`AbortSignal`

#### Returns

`Promise`\<[`HashrateSummary`](../interfaces/HashrateSummary.md)\>

***

### getHealth()

> **getHealth**(`options?`): `Promise`\<[`Health`](../interfaces/Health.md)\>

Defined in: [Developer/mono/modules/bitview-client/index.js:23266](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L23266)

Health check

Liveness probe. Returns server identity, uptime, and indexed/computed heights from local state only (no bitcoind round-trip). For real chain-tip catch-up, request `GET /api/server/sync`.

Endpoint: `GET /health`

#### Parameters

##### options?

###### cache?

`boolean`

###### memCache?

`boolean`

###### onValue?

(`value`) => `void`

###### signal?

`AbortSignal`

#### Returns

`Promise`\<[`Health`](../interfaces/Health.md)\>

***

### getHistoricalPrice()

> **getHistoricalPrice**(`timestamp?`, `options?`): `Promise`\<[`HistoricalPrice`](../interfaces/HistoricalPrice.md)\>

Defined in: [Developer/mono/modules/bitview-client/index.js:23674](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L23674)

Historical price

Get historical BTC/USD price. Optionally specify a UNIX timestamp to get the price at that time.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-historical-price)*

Endpoint: `GET /api/v1/historical-price`

#### Parameters

##### timestamp?

`number`

##### options?

###### cache?

`boolean`

###### memCache?

`boolean`

###### onValue?

(`value`) => `void`

###### signal?

`AbortSignal`

#### Returns

`Promise`\<[`HistoricalPrice`](../interfaces/HistoricalPrice.md)\>

***

### getIndexes()

> **getIndexes**(`options?`): `Promise`\<[`IndexInfo`](../interfaces/IndexInfo.md)[]\>

Defined in: [Developer/mono/modules/bitview-client/index.js:23350](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L23350)

List available indexes

Returns all available indexes with their accepted query aliases. Use any alias when querying series.

Endpoint: `GET /api/series/indexes`

#### Parameters

##### options?

###### cache?

`boolean`

###### memCache?

`boolean`

###### onValue?

(`value`) => `void`

###### signal?

`AbortSignal`

#### Returns

`Promise`\<[`IndexInfo`](../interfaces/IndexInfo.md)[]\>

***

### getJson()

> **getJson**\<`T`\>(`path`, `options?`): `Promise`\<`T`\>

Defined in: [Developer/mono/modules/bitview-client/index.js:2196](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L2196)

Make a GET request expecting a JSON response. Cached and supports `onValue`.

#### Type Parameters

##### T

`T`

#### Parameters

##### path

`string`

##### options?

[`ClientFetchOptions`](../interfaces/ClientFetchOptions.md)\<`T`\>

#### Returns

`Promise`\<`T`\>

#### Inherited from

`BitviewClientBase.getJson`

***

### getLivePrice()

> **getLivePrice**(`options?`): `Promise`\<`number`\>

Defined in: [Developer/mono/modules/bitview-client/index.js:24607](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L24607)

Live BTC/USD price

Returns the current BTC/USD price in dollars, derived from on-chain round-dollar output patterns in the last 12 blocks plus mempool.

Endpoint: `GET /api/mempool/price`

#### Parameters

##### options?

###### cache?

`boolean`

###### memCache?

`boolean`

###### onValue?

(`value`) => `void`

###### signal?

`AbortSignal`

#### Returns

`Promise`\<`number`\>

***

### getMempool()

> **getMempool**(`options?`): `Promise`\<[`MempoolInfo`](../interfaces/MempoolInfo.md)\>

Defined in: [Developer/mono/modules/bitview-client/index.js:24485](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L24485)

Mempool statistics

Get current mempool statistics including transaction count, total vsize, total fees, and fee histogram.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-mempool)*

Endpoint: `GET /api/mempool`

#### Parameters

##### options?

###### cache?

`boolean`

###### memCache?

`boolean`

###### onValue?

(`value`) => `void`

###### signal?

`AbortSignal`

#### Returns

`Promise`\<[`MempoolInfo`](../interfaces/MempoolInfo.md)\>

***

### getMempoolBlocks()

> **getMempoolBlocks**(`options?`): `Promise`\<[`MempoolBlock`](../interfaces/MempoolBlock.md)[]\>

Defined in: [Developer/mono/modules/bitview-client/index.js:24437](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L24437)

Projected mempool blocks

Projected blocks for fee estimation. Block 0 reflects Bitcoin Core's actual next-block selection; blocks 1+ are a fee-tier approximation.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-mempool-blocks-fees)*

Endpoint: `GET /api/v1/fees/mempool-blocks`

#### Parameters

##### options?

###### cache?

`boolean`

###### memCache?

`boolean`

###### onValue?

(`value`) => `void`

###### signal?

`AbortSignal`

#### Returns

`Promise`\<[`MempoolBlock`](../interfaces/MempoolBlock.md)[]\>

***

### getMempoolHash()

> **getMempoolHash**(`options?`): `Promise`\<`number`\>

Defined in: [Developer/mono/modules/bitview-client/index.js:24499](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L24499)

Mempool content hash

Returns an opaque hash that changes whenever the projected next block changes. Same value as the mempool ETag. Useful as a freshness/liveness signal: if it stays constant for tens of seconds on a live network, the mempool sync loop has stalled.

Endpoint: `GET /api/mempool/hash`

#### Parameters

##### options?

###### cache?

`boolean`

###### memCache?

`boolean`

###### onValue?

(`value`) => `void`

###### signal?

`AbortSignal`

#### Returns

`Promise`\<`number`\>

***

### getMempoolRecent()

> **getMempoolRecent**(`options?`): `Promise`\<[`MempoolRecentTx`](../interfaces/MempoolRecentTx.md)[]\>

Defined in: [Developer/mono/modules/bitview-client/index.js:24531](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L24531)

Recent mempool transactions

Get the last 10 transactions to enter the mempool.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-mempool-recent)*

Endpoint: `GET /api/mempool/recent`

#### Parameters

##### options?

###### cache?

`boolean`

###### memCache?

`boolean`

###### onValue?

(`value`) => `void`

###### signal?

`AbortSignal`

#### Returns

`Promise`\<[`MempoolRecentTx`](../interfaces/MempoolRecentTx.md)[]\>

***

### getMempoolTxids()

> **getMempoolTxids**(`options?`): `Promise`\<`string`[]\>

Defined in: [Developer/mono/modules/bitview-client/index.js:24515](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L24515)

Mempool transaction IDs

Get all transaction IDs currently in the mempool.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-mempool-transaction-ids)*

Endpoint: `GET /api/mempool/txids`

#### Parameters

##### options?

###### cache?

`boolean`

###### memCache?

`boolean`

###### onValue?

(`value`) => `void`

###### signal?

`AbortSignal`

#### Returns

`Promise`\<`string`[]\>

***

### getOpenapi()

> **getOpenapi**(`options?`): `Promise`\<`any`\>

Defined in: [Developer/mono/modules/bitview-client/index.js:24933](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L24933)

OpenAPI specification

Full OpenAPI 3.1 specification for this API.

Endpoint: `GET /openapi.json`

#### Parameters

##### options?

###### cache?

`boolean`

###### memCache?

`boolean`

###### onValue?

(`value`) => `void`

###### signal?

`AbortSignal`

#### Returns

`Promise`\<`any`\>

***

### getOracleHistogramOutputs()

> **getOracleHistogramOutputs**(`point`, `options?`): `Promise`\<`number`[]\>

Defined in: [Developer/mono/modules/bitview-client/index.js:24919](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L24919)

Output value histogram at height or day

Unfiltered output value histogram for a confirmed point. A block height (`840000`) gives every output in that block, coinbase included, binned by value on the oracle log scale; a calendar date (`YYYY-MM-DD`) sums every block that day. A flat array of log-scale bins.

Endpoint: `GET /api/oracle/histogram/outputs/{point}`

#### Parameters

##### point

`string`

Confirmed block height as decimal digits (`840000`) or calendar date in
`YYYY-MM-DD` format.

##### options?

###### cache?

`boolean`

###### memCache?

`boolean`

###### onValue?

(`value`) => `void`

###### signal?

`AbortSignal`

#### Returns

`Promise`\<`number`[]\>

***

### getOracleHistogramOutputsLive()

> **getOracleHistogramOutputsLive**(`options?`): `Promise`\<`number`[]\>

Defined in: [Developer/mono/modules/bitview-client/index.js:24902](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L24902)

Live output value histogram

Live unfiltered output value histogram for the forming mempool block. Every live output is binned by value on the oracle log scale; no oracle payment filters are applied. A flat array of log-scale bins, all zero when no mempool is configured.

Endpoint: `GET /api/oracle/histogram/outputs/live`

#### Parameters

##### options?

###### cache?

`boolean`

###### memCache?

`boolean`

###### onValue?

(`value`) => `void`

###### signal?

`AbortSignal`

#### Returns

`Promise`\<`number`[]\>

***

### getOracleHistogramPayments()

> **getOracleHistogramPayments**(`point`, `options?`): `Promise`\<`number`[]\>

Defined in: [Developer/mono/modules/bitview-client/index.js:24888](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L24888)

Payment output histogram at height or day

Smoothed histogram of oracle-eligible payment outputs for a confirmed point. A block height (`840000`) gives that block's oracle payment histogram; a calendar date (`YYYY-MM-DD`) gives the average of that day's per-block payment histograms. A flat array of log-scale bins.

Endpoint: `GET /api/oracle/histogram/payments/{point}`

#### Parameters

##### point

`string`

Confirmed block height as decimal digits (`840000`) or calendar date in
`YYYY-MM-DD` format.

##### options?

###### cache?

`boolean`

###### memCache?

`boolean`

###### onValue?

(`value`) => `void`

###### signal?

`AbortSignal`

#### Returns

`Promise`\<`number`[]\>

***

### getOracleHistogramPaymentsLive()

> **getOracleHistogramPaymentsLive**(`options?`): `Promise`\<`number`[]\>

Defined in: [Developer/mono/modules/bitview-client/index.js:24871](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L24871)

Live payment output histogram

Live smoothed histogram of oracle-eligible payment outputs, binned by output value on the oracle log scale. It combines the committed oracle window with the forming mempool block. A flat array of log-scale bins.

Endpoint: `GET /api/oracle/histogram/payments/live`

#### Parameters

##### options?

###### cache?

`boolean`

###### memCache?

`boolean`

###### onValue?

(`value`) => `void`

###### signal?

`AbortSignal`

#### Returns

`Promise`\<`number`[]\>

***

### getOraclePrice()

> **getOraclePrice**(`options?`): `Promise`\<`number`\>

Defined in: [Developer/mono/modules/bitview-client/index.js:24857](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L24857)

Live BTC/USD price

Current BTC/USD price in dollars. Same value as `GET /api/mempool/price`. Confirmed per-height history is available at `GET /api/series/price/height`.

Endpoint: `GET /api/oracle/price`

#### Parameters

##### options?

###### cache?

`boolean`

###### memCache?

`boolean`

###### onValue?

(`value`) => `void`

###### signal?

`AbortSignal`

#### Returns

`Promise`\<`number`\>

***

### getPool()

> **getPool**(`slug`, `options?`): `Promise`\<[`PoolDetail`](../interfaces/PoolDetail.md)\>

Defined in: [Developer/mono/modules/bitview-client/index.js:24174](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L24174)

Mining pool details

Get detailed information about a specific mining pool including block counts and shares for different time periods.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-mining-pool)*

Endpoint: `GET /api/v1/mining/pool/{slug}`

#### Parameters

##### slug

[`PoolSlug`](../type-aliases/PoolSlug.md)

##### options?

###### cache?

`boolean`

###### memCache?

`boolean`

###### onValue?

(`value`) => `void`

###### signal?

`AbortSignal`

#### Returns

`Promise`\<[`PoolDetail`](../interfaces/PoolDetail.md)\>

***

### getPoolBlocks()

> **getPoolBlocks**(`slug`, `options?`): `Promise`\<[`BlockInfoV1`](../interfaces/BlockInfoV1.md)[]\>

Defined in: [Developer/mono/modules/bitview-client/index.js:24244](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L24244)

Mining pool blocks

Get the 10 most recent blocks mined by a specific pool.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-mining-pool-blocks)*

Endpoint: `GET /api/v1/mining/pool/{slug}/blocks`

#### Parameters

##### slug

[`PoolSlug`](../type-aliases/PoolSlug.md)

##### options?

###### cache?

`boolean`

###### memCache?

`boolean`

###### onValue?

(`value`) => `void`

###### signal?

`AbortSignal`

#### Returns

`Promise`\<[`BlockInfoV1`](../interfaces/BlockInfoV1.md)[]\>

***

### getPoolBlocksFrom()

> **getPoolBlocksFrom**(`slug`, `height`, `options?`): `Promise`\<[`BlockInfoV1`](../interfaces/BlockInfoV1.md)[]\>

Defined in: [Developer/mono/modules/bitview-client/index.js:24263](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L24263)

Mining pool blocks from height

Get 10 blocks mined by a specific pool before (and including) the given height.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-mining-pool-blocks)*

Endpoint: `GET /api/v1/mining/pool/{slug}/blocks/{height}`

#### Parameters

##### slug

[`PoolSlug`](../type-aliases/PoolSlug.md)

##### height

`number`

##### options?

###### cache?

`boolean`

###### memCache?

`boolean`

###### onValue?

(`value`) => `void`

###### signal?

`AbortSignal`

#### Returns

`Promise`\<[`BlockInfoV1`](../interfaces/BlockInfoV1.md)[]\>

***

### getPoolHashrate()

> **getPoolHashrate**(`slug`, `options?`): `Promise`\<[`PoolHashrateEntry`](../interfaces/PoolHashrateEntry.md)[]\>

Defined in: [Developer/mono/modules/bitview-client/index.js:24226](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L24226)

Mining pool hashrate

Get hashrate history for a specific mining pool.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-mining-pool-hashrate)*

Endpoint: `GET /api/v1/mining/pool/{slug}/hashrate`

#### Parameters

##### slug

[`PoolSlug`](../type-aliases/PoolSlug.md)

##### options?

###### cache?

`boolean`

###### memCache?

`boolean`

###### onValue?

(`value`) => `void`

###### signal?

`AbortSignal`

#### Returns

`Promise`\<[`PoolHashrateEntry`](../interfaces/PoolHashrateEntry.md)[]\>

***

### getPools()

> **getPools**(`options?`): `Promise`\<[`PoolInfo`](../interfaces/PoolInfo.md)[]\>

Defined in: [Developer/mono/modules/bitview-client/index.js:24138](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L24138)

List all mining pools

Get list of all known mining pools with their identifiers.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-mining-pools)*

Endpoint: `GET /api/v1/mining/pools`

#### Parameters

##### options?

###### cache?

`boolean`

###### memCache?

`boolean`

###### onValue?

(`value`) => `void`

###### signal?

`AbortSignal`

#### Returns

`Promise`\<[`PoolInfo`](../interfaces/PoolInfo.md)[]\>

***

### getPoolsHashrate()

> **getPoolsHashrate**(`options?`): `Promise`\<[`PoolHashrateEntry`](../interfaces/PoolHashrateEntry.md)[]\>

Defined in: [Developer/mono/modules/bitview-client/index.js:24190](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L24190)

All pools hashrate (all time)

Get hashrate data for all mining pools.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-mining-pool-hashrates)*

Endpoint: `GET /api/v1/mining/hashrate/pools`

#### Parameters

##### options?

###### cache?

`boolean`

###### memCache?

`boolean`

###### onValue?

(`value`) => `void`

###### signal?

`AbortSignal`

#### Returns

`Promise`\<[`PoolHashrateEntry`](../interfaces/PoolHashrateEntry.md)[]\>

***

### getPoolsHashrateByPeriod()

> **getPoolsHashrateByPeriod**(`time_period`, `options?`): `Promise`\<[`PoolHashrateEntry`](../interfaces/PoolHashrateEntry.md)[]\>

Defined in: [Developer/mono/modules/bitview-client/index.js:24208](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L24208)

All pools hashrate

Get hashrate data for all mining pools for a time period. Valid periods: `1m`, `3m`, `6m`, `1y`, `2y`, `3y`.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-mining-pool-hashrates)*

Endpoint: `GET /api/v1/mining/hashrate/pools/{time_period}`

#### Parameters

##### time\_period

[`TimePeriod`](../type-aliases/TimePeriod.md)

##### options?

###### cache?

`boolean`

###### memCache?

`boolean`

###### onValue?

(`value`) => `void`

###### signal?

`AbortSignal`

#### Returns

`Promise`\<[`PoolHashrateEntry`](../interfaces/PoolHashrateEntry.md)[]\>

***

### getPoolStats()

> **getPoolStats**(`time_period`, `options?`): `Promise`\<[`PoolsSummary`](../interfaces/PoolsSummary.md)\>

Defined in: [Developer/mono/modules/bitview-client/index.js:24156](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L24156)

Mining pool statistics

Get mining pool statistics for a time period. Valid periods: `24h`, `3d`, `1w`, `1m`, `3m`, `6m`, `1y`, `2y`, `3y`.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-mining-pools)*

Endpoint: `GET /api/v1/mining/pools/{time_period}`

#### Parameters

##### time\_period

[`TimePeriod`](../type-aliases/TimePeriod.md)

##### options?

###### cache?

`boolean`

###### memCache?

`boolean`

###### onValue?

(`value`) => `void`

###### signal?

`AbortSignal`

#### Returns

`Promise`\<[`PoolsSummary`](../interfaces/PoolsSummary.md)\>

***

### getPreciseFees()

> **getPreciseFees**(`options?`): `Promise`\<[`RecommendedFees`](../interfaces/RecommendedFees.md)\>

Defined in: [Developer/mono/modules/bitview-client/index.js:24469](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L24469)

Recommended fee rates (precise)

Recommended fee rates by confirmation target, with up to three decimal places and support for sub-sat/vB rates.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-recommended-fees-precise)*

Endpoint: `GET /api/v1/fees/precise`

#### Parameters

##### options?

###### cache?

`boolean`

###### memCache?

`boolean`

###### onValue?

(`value`) => `void`

###### signal?

`AbortSignal`

#### Returns

`Promise`\<[`RecommendedFees`](../interfaces/RecommendedFees.md)\>

***

### getPrices()

> **getPrices**(`options?`): `Promise`\<[`Prices`](../interfaces/Prices.md)\>

Defined in: [Developer/mono/modules/bitview-client/index.js:23656](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L23656)

Current BTC price

Returns bitcoin latest price (on-chain derived, USD only).

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-price)*

Endpoint: `GET /api/v1/prices`

#### Parameters

##### options?

###### cache?

`boolean`

###### memCache?

`boolean`

###### onValue?

(`value`) => `void`

###### signal?

`AbortSignal`

#### Returns

`Promise`\<[`Prices`](../interfaces/Prices.md)\>

***

### getRecommendedFees()

> **getRecommendedFees**(`options?`): `Promise`\<[`RecommendedFees`](../interfaces/RecommendedFees.md)\>

Defined in: [Developer/mono/modules/bitview-client/index.js:24453](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L24453)

Recommended fees

Recommended fee rates by confirmation target.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-recommended-fees)*

Endpoint: `GET /api/v1/fees/recommended`

#### Parameters

##### options?

###### cache?

`boolean`

###### memCache?

`boolean`

###### onValue?

(`value`) => `void`

###### signal?

`AbortSignal`

#### Returns

`Promise`\<[`RecommendedFees`](../interfaces/RecommendedFees.md)\>

***

### getReplacements()

> **getReplacements**(`options?`): `Promise`\<[`ReplacementNode`](../interfaces/ReplacementNode.md)[]\>

Defined in: [Developer/mono/modules/bitview-client/index.js:24547](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L24547)

Recent RBF replacements

Returns up to 25 most-recent RBF replacement trees across the whole mempool. Each entry has the same shape as `tx_rbf().replacements`.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-replacements)*

Endpoint: `GET /api/v1/replacements`

#### Parameters

##### options?

###### cache?

`boolean`

###### memCache?

`boolean`

###### onValue?

(`value`) => `void`

###### signal?

`AbortSignal`

#### Returns

`Promise`\<[`ReplacementNode`](../interfaces/ReplacementNode.md)[]\>

***

### getRewardStats()

> **getRewardStats**(`block_count`, `options?`): `Promise`\<[`RewardStats`](../interfaces/RewardStats.md)\>

Defined in: [Developer/mono/modules/bitview-client/index.js:24349](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L24349)

Mining reward statistics

Get mining reward statistics for the last N blocks including total rewards, fees, and transaction count.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-reward-stats)*

Endpoint: `GET /api/v1/mining/reward-stats/{block_count}`

#### Parameters

##### block\_count

`number`

Number of recent blocks to include

##### options?

###### cache?

`boolean`

###### memCache?

`boolean`

###### onValue?

(`value`) => `void`

###### signal?

`AbortSignal`

#### Returns

`Promise`\<[`RewardStats`](../interfaces/RewardStats.md)\>

***

### getSeries()

> **getSeries**(`series`, `index`, `start?`, `end?`, `limit?`, `format?`, `options?`): `Promise`\<`string` \| [`AnySeriesData`](../type-aliases/AnySeriesData.md)\>

Defined in: [Developer/mono/modules/bitview-client/index.js:23429](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L23429)

Get series data

Fetch data for a specific series at the given index. Use query parameters to filter by date range and format (json/csv).

Endpoint: `GET /api/series/{series}/{index}`

#### Parameters

##### series

`string`

Series name

##### index

[`Index`](../type-aliases/Index.md)

Aggregation index

##### start?

[`RangeIndex`](../type-aliases/RangeIndex.md)

Inclusive start: integer index, date (YYYY-MM-DD), or timestamp (ISO 8601). Negative integers count from end. Aliases: `from`, `f`, `s`

##### end?

[`RangeIndex`](../type-aliases/RangeIndex.md)

Exclusive end: integer index, date (YYYY-MM-DD), or timestamp (ISO 8601). Negative integers count from end. Aliases: `to`, `t`, `e`

##### limit?

`number`

Maximum number of values to return (ignored if `end` is set). Aliases: `count`, `c`, `l`

##### format?

[`Format`](../type-aliases/Format.md)

Format of the output

##### options?

###### cache?

`boolean`

###### memCache?

`boolean`

###### onValue?

(`value`) => `void`

###### signal?

`AbortSignal`

#### Returns

`Promise`\<`string` \| [`AnySeriesData`](../type-aliases/AnySeriesData.md)\>

***

### getSeriesBulk()

> **getSeriesBulk**(`series`, `index`, `start?`, `end?`, `limit?`, `format?`, `options?`): `Promise`\<`string` \| [`AnySeriesData`](../type-aliases/AnySeriesData.md)[]\>

Defined in: [Developer/mono/modules/bitview-client/index.js:23536](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L23536)

Bulk series data

Fetch multiple series in a single request. Supports filtering by index and date range. Returns an array of SeriesData objects. For a single series, use `get_series` instead.

Endpoint: `GET /api/series/bulk`

#### Parameters

##### series

`string`

Requested series

##### index

[`Index`](../type-aliases/Index.md)

Index to query

##### start?

[`RangeIndex`](../type-aliases/RangeIndex.md)

Inclusive start: integer index, date (YYYY-MM-DD), or timestamp (ISO 8601). Negative integers count from end. Aliases: `from`, `f`, `s`

##### end?

[`RangeIndex`](../type-aliases/RangeIndex.md)

Exclusive end: integer index, date (YYYY-MM-DD), or timestamp (ISO 8601). Negative integers count from end. Aliases: `to`, `t`, `e`

##### limit?

`number`

Maximum number of values to return (ignored if `end` is set). Aliases: `count`, `c`, `l`

##### format?

[`Format`](../type-aliases/Format.md)

Format of the output

##### options?

###### cache?

`boolean`

###### memCache?

`boolean`

###### onValue?

(`value`) => `void`

###### signal?

`AbortSignal`

#### Returns

`Promise`\<`string` \| [`AnySeriesData`](../type-aliases/AnySeriesData.md)[]\>

***

### getSeriesCount()

> **getSeriesCount**(`options?`): `Promise`\<[`DetailedSeriesCount`](../interfaces/DetailedSeriesCount.md)\>

Defined in: [Developer/mono/modules/bitview-client/index.js:23336](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L23336)

Series count

Returns the number of series available per index type.

Endpoint: `GET /api/series/count`

#### Parameters

##### options?

###### cache?

`boolean`

###### memCache?

`boolean`

###### onValue?

(`value`) => `void`

###### signal?

`AbortSignal`

#### Returns

`Promise`\<[`DetailedSeriesCount`](../interfaces/DetailedSeriesCount.md)\>

***

### getSeriesData()

> **getSeriesData**(`series`, `index`, `start?`, `end?`, `limit?`, `format?`, `options?`): `Promise`\<`string` \| `boolean`[]\>

Defined in: [Developer/mono/modules/bitview-client/index.js:23457](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L23457)

Get raw series data

Returns just the data array without the SeriesData wrapper. Supports the same range and format parameters as `GET /api/series/{series}/{index}`.

Endpoint: `GET /api/series/{series}/{index}/data`

#### Parameters

##### series

`string`

Series name

##### index

[`Index`](../type-aliases/Index.md)

Aggregation index

##### start?

[`RangeIndex`](../type-aliases/RangeIndex.md)

Inclusive start: integer index, date (YYYY-MM-DD), or timestamp (ISO 8601). Negative integers count from end. Aliases: `from`, `f`, `s`

##### end?

[`RangeIndex`](../type-aliases/RangeIndex.md)

Exclusive end: integer index, date (YYYY-MM-DD), or timestamp (ISO 8601). Negative integers count from end. Aliases: `to`, `t`, `e`

##### limit?

`number`

Maximum number of values to return (ignored if `end` is set). Aliases: `count`, `c`, `l`

##### format?

[`Format`](../type-aliases/Format.md)

Format of the output

##### options?

###### cache?

`boolean`

###### memCache?

`boolean`

###### onValue?

(`value`) => `void`

###### signal?

`AbortSignal`

#### Returns

`Promise`\<`string` \| `boolean`[]\>

***

### getSeriesInfo()

> **getSeriesInfo**(`series`, `options?`): `Promise`\<[`SeriesInfo`](../interfaces/SeriesInfo.md)\>

Defined in: [Developer/mono/modules/bitview-client/index.js:23408](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L23408)

Get series info

Returns the optional description, supported indexes, and value type for the specified series.

Endpoint: `GET /api/series/{series}`

#### Parameters

##### series

`string`

##### options?

###### cache?

`boolean`

###### memCache?

`boolean`

###### onValue?

(`value`) => `void`

###### signal?

`AbortSignal`

#### Returns

`Promise`\<[`SeriesInfo`](../interfaces/SeriesInfo.md)\>

***

### getSeriesLatest()

> **getSeriesLatest**(`series`, `index`, `options?`): `Promise`\<`any`\>

Defined in: [Developer/mono/modules/bitview-client/index.js:23481](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L23481)

Get latest series value

Returns the single most recent value for a series, unwrapped (not inside a SeriesData object).

Endpoint: `GET /api/series/{series}/{index}/latest`

#### Parameters

##### series

`string`

Series name

##### index

[`Index`](../type-aliases/Index.md)

Aggregation index

##### options?

###### cache?

`boolean`

###### memCache?

`boolean`

###### onValue?

(`value`) => `void`

###### signal?

`AbortSignal`

#### Returns

`Promise`\<`any`\>

***

### getSeriesLen()

> **getSeriesLen**(`series`, `index`, `options?`): `Promise`\<`number`\>

Defined in: [Developer/mono/modules/bitview-client/index.js:23498](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L23498)

Get series data length

Returns the total number of data points for a series at the given index.

Endpoint: `GET /api/series/{series}/{index}/len`

#### Parameters

##### series

`string`

Series name

##### index

[`Index`](../type-aliases/Index.md)

Aggregation index

##### options?

###### cache?

`boolean`

###### memCache?

`boolean`

###### onValue?

(`value`) => `void`

###### signal?

`AbortSignal`

#### Returns

`Promise`\<`number`\>

***

### getSeriesTree()

> **getSeriesTree**(`options?`): `Promise`\<[`TreeNode`](../type-aliases/TreeNode.md)\>

Defined in: [Developer/mono/modules/bitview-client/index.js:23322](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L23322)

Series catalog

Returns the complete hierarchical catalog of available series organized as a tree structure. Series are grouped by categories and subcategories.

Endpoint: `GET /api/series`

#### Parameters

##### options?

###### cache?

`boolean`

###### memCache?

`boolean`

###### onValue?

(`value`) => `void`

###### signal?

`AbortSignal`

#### Returns

`Promise`\<[`TreeNode`](../type-aliases/TreeNode.md)\>

***

### getSeriesVersion()

> **getSeriesVersion**(`series`, `index`, `options?`): `Promise`\<`number`\>

Defined in: [Developer/mono/modules/bitview-client/index.js:23515](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L23515)

Get series version

Returns the current version of a series. Changes when the series data is updated.

Endpoint: `GET /api/series/{series}/{index}/version`

#### Parameters

##### series

`string`

Series name

##### index

[`Index`](../type-aliases/Index.md)

Aggregation index

##### options?

###### cache?

`boolean`

###### memCache?

`boolean`

###### onValue?

(`value`) => `void`

###### signal?

`AbortSignal`

#### Returns

`Promise`\<`number`\>

***

### getSyncStatus()

> **getSyncStatus**(`options?`): `Promise`\<[`SyncStatus`](../interfaces/SyncStatus.md)\>

Defined in: [Developer/mono/modules/bitview-client/index.js:23294](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L23294)

Sync status

Returns the sync status of the indexer, including indexed height, tip height, blocks behind, and last indexed timestamp.

Endpoint: `GET /api/server/sync`

#### Parameters

##### options?

###### cache?

`boolean`

###### memCache?

`boolean`

###### onValue?

(`value`) => `void`

###### signal?

`AbortSignal`

#### Returns

`Promise`\<[`SyncStatus`](../interfaces/SyncStatus.md)\>

***

### getText()

> **getText**(`path`, `options?`): `Promise`\<`string`\>

Defined in: [Developer/mono/modules/bitview-client/index.js:2207](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L2207)

Make a GET request expecting a text response (text/plain, text/csv, ...).
Cached and supports `onValue`, same as `getJson`.

#### Parameters

##### path

`string`

##### options?

[`ClientFetchOptions`](../interfaces/ClientFetchOptions.md)\<`string`\>

#### Returns

`Promise`\<`string`\>

#### Inherited from

`BitviewClientBase.getText`

***

### getTransactionTimes()

> **getTransactionTimes**(`txId`, `options?`): `Promise`\<`number`[]\>

Defined in: [Developer/mono/modules/bitview-client/index.js:24822](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L24822)

Transaction first-seen times

Returns timestamps when transactions were first seen in the mempool. Returns 0 for mined or unknown transactions.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-transaction-times)*

Endpoint: `GET /api/v1/transaction-times`

#### Parameters

##### txId

`string`[]

Transaction IDs to look up (max 250 per request).

##### options?

###### cache?

`boolean`

###### memCache?

`boolean`

###### onValue?

(`value`) => `void`

###### signal?

`AbortSignal`

#### Returns

`Promise`\<`number`[]\>

***

### getTx()

> **getTx**(`txid`, `options?`): `Promise`\<[`Transaction`](../interfaces/Transaction.md)\>

Defined in: [Developer/mono/modules/bitview-client/index.js:24677](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L24677)

Transaction information

Retrieve complete transaction data by transaction ID (txid). Returns inputs, outputs, fee, size, and confirmation status.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-transaction)*

Endpoint: `GET /api/tx/{txid}`

#### Parameters

##### txid

`string`

##### options?

###### cache?

`boolean`

###### memCache?

`boolean`

###### onValue?

(`value`) => `void`

###### signal?

`AbortSignal`

#### Returns

`Promise`\<[`Transaction`](../interfaces/Transaction.md)\>

***

### getTxByIndex()

> **getTxByIndex**(`index`, `options?`): `Promise`\<`string`\>

Defined in: [Developer/mono/modules/bitview-client/index.js:24623](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L24623)

Txid by index

Retrieve the transaction ID (txid) at a given global transaction index. Returns the txid as plain text.

Endpoint: `GET /api/tx-index/{index}`

#### Parameters

##### index

`number`

##### options?

###### cache?

`boolean`

###### memCache?

`boolean`

###### onValue?

(`value`) => `void`

###### signal?

`AbortSignal`

#### Returns

`Promise`\<`string`\>

***

### getTxHex()

> **getTxHex**(`txid`, `options?`): `Promise`\<`string`\>

Defined in: [Developer/mono/modules/bitview-client/index.js:24695](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L24695)

Transaction hex

Retrieve the raw transaction as a hex-encoded string. Returns the serialized transaction in hexadecimal format.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-transaction-hex)*

Endpoint: `GET /api/tx/{txid}/hex`

#### Parameters

##### txid

`string`

##### options?

###### cache?

`boolean`

###### memCache?

`boolean`

###### onValue?

(`value`) => `void`

###### signal?

`AbortSignal`

#### Returns

`Promise`\<`string`\>

***

### getTxMerkleblockProof()

> **getTxMerkleblockProof**(`txid`, `options?`): `Promise`\<`string`\>

Defined in: [Developer/mono/modules/bitview-client/index.js:24713](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L24713)

Transaction merkleblock proof

Get the merkleblock proof for a transaction (BIP37 format, hex encoded).

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-transaction-merkleblock-proof)*

Endpoint: `GET /api/tx/{txid}/merkleblock-proof`

#### Parameters

##### txid

`string`

##### options?

###### cache?

`boolean`

###### memCache?

`boolean`

###### onValue?

(`value`) => `void`

###### signal?

`AbortSignal`

#### Returns

`Promise`\<`string`\>

***

### getTxMerkleProof()

> **getTxMerkleProof**(`txid`, `options?`): `Promise`\<[`MerkleProof`](../interfaces/MerkleProof.md)\>

Defined in: [Developer/mono/modules/bitview-client/index.js:24731](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L24731)

Transaction merkle proof

Get the merkle inclusion proof for a transaction.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-transaction-merkle-proof)*

Endpoint: `GET /api/tx/{txid}/merkle-proof`

#### Parameters

##### txid

`string`

##### options?

###### cache?

`boolean`

###### memCache?

`boolean`

###### onValue?

(`value`) => `void`

###### signal?

`AbortSignal`

#### Returns

`Promise`\<[`MerkleProof`](../interfaces/MerkleProof.md)\>

***

### getTxOutspend()

> **getTxOutspend**(`txid`, `vout`, `options?`): `Promise`\<[`TxOutspend`](../interfaces/TxOutspend.md)\>

Defined in: [Developer/mono/modules/bitview-client/index.js:24750](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L24750)

Output spend status

Get the spending status of a transaction output. Returns whether the output has been spent and, if so, the spending transaction details.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-transaction-outspend)*

Endpoint: `GET /api/tx/{txid}/outspend/{vout}`

#### Parameters

##### txid

`string`

Transaction ID

##### vout

`number`

Output index

##### options?

###### cache?

`boolean`

###### memCache?

`boolean`

###### onValue?

(`value`) => `void`

###### signal?

`AbortSignal`

#### Returns

`Promise`\<[`TxOutspend`](../interfaces/TxOutspend.md)\>

***

### getTxOutspends()

> **getTxOutspends**(`txid`, `options?`): `Promise`\<[`TxOutspend`](../interfaces/TxOutspend.md)[]\>

Defined in: [Developer/mono/modules/bitview-client/index.js:24768](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L24768)

All output spend statuses

Get the spending status of all outputs in a transaction. Returns an array with the spend status for each output.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-transaction-outspends)*

Endpoint: `GET /api/tx/{txid}/outspends`

#### Parameters

##### txid

`string`

##### options?

###### cache?

`boolean`

###### memCache?

`boolean`

###### onValue?

(`value`) => `void`

###### signal?

`AbortSignal`

#### Returns

`Promise`\<[`TxOutspend`](../interfaces/TxOutspend.md)[]\>

***

### getTxRaw()

> **getTxRaw**(`txid`, `options?`): `Promise`\<`Uint8Array`\<`ArrayBufferLike`\>\>

Defined in: [Developer/mono/modules/bitview-client/index.js:24786](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L24786)

Transaction raw

Returns a transaction as binary data.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-transaction-raw)*

Endpoint: `GET /api/tx/{txid}/raw`

#### Parameters

##### txid

`string`

##### options?

###### cache?

`boolean`

###### memCache?

`boolean`

###### onValue?

(`value`) => `void`

###### signal?

`AbortSignal`

#### Returns

`Promise`\<`Uint8Array`\<`ArrayBufferLike`\>\>

***

### getTxRbf()

> **getTxRbf**(`txid`, `options?`): `Promise`\<[`RbfResponse`](../interfaces/RbfResponse.md)\>

Defined in: [Developer/mono/modules/bitview-client/index.js:24659](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L24659)

RBF replacement history

Returns the RBF replacement tree for a transaction, if any. Both `replacements` and `replaces` are null when the tx has no known RBF history within the mempool monitor's retention window.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-transaction-rbf-history)*

Endpoint: `GET /api/v1/tx/{txid}/rbf`

#### Parameters

##### txid

`string`

##### options?

###### cache?

`boolean`

###### memCache?

`boolean`

###### onValue?

(`value`) => `void`

###### signal?

`AbortSignal`

#### Returns

`Promise`\<[`RbfResponse`](../interfaces/RbfResponse.md)\>

***

### getTxStatus()

> **getTxStatus**(`txid`, `options?`): `Promise`\<[`TxStatus`](../interfaces/TxStatus.md)\>

Defined in: [Developer/mono/modules/bitview-client/index.js:24804](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L24804)

Transaction status

Retrieve the confirmation status of a transaction. Returns whether the transaction is confirmed and, if so, the block height, hash, and timestamp.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-transaction-status)*

Endpoint: `GET /api/tx/{txid}/status`

#### Parameters

##### txid

`string`

##### options?

###### cache?

`boolean`

###### memCache?

`boolean`

###### onValue?

(`value`) => `void`

###### signal?

`AbortSignal`

#### Returns

`Promise`\<[`TxStatus`](../interfaces/TxStatus.md)\>

***

### getUrpd()

> **getUrpd**(`cohort`, `agg?`, `weight?`, `options?`): `Promise`\<[`Urpd`](../interfaces/Urpd.md)\>

Defined in: [Developer/mono/modules/bitview-client/index.js:23597](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L23597)

Latest URPD

URPD for the most recent available date in the cohort. The response's `date` field echoes which date was served. Returns `{ cohort, date, weight, aggregation, close, total_supply, buckets }`. `close` and each bucket's `price_floor`, `realized_cap`, and `unrealized_pnl` are USD; `total_supply` and bucket `supply` are BTC. `unrealized_pnl` can be negative.

Endpoint: `GET /api/urpd/{cohort}`

#### Parameters

##### cohort

[`Cohort`](../type-aliases/Cohort.md)

##### agg?

[`UrpdAggregation`](../type-aliases/UrpdAggregation.md)

Aggregation strategy. Default: raw (no aggregation). Accepts `bucket` as alias.

##### weight?

[`UrpdWeight`](../type-aliases/UrpdWeight.md)

Supply weighting. Default: raw (unweighted).

##### options?

###### cache?

`boolean`

###### memCache?

`boolean`

###### onValue?

(`value`) => `void`

###### signal?

`AbortSignal`

#### Returns

`Promise`\<[`Urpd`](../interfaces/Urpd.md)\>

***

### getUrpdAt()

> **getUrpdAt**(`cohort`, `date`, `agg?`, `weight?`, `options?`): `Promise`\<[`Urpd`](../interfaces/Urpd.md)\>

Defined in: [Developer/mono/modules/bitview-client/index.js:23620](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L23620)

URPD at date

URPD for a (cohort, date) pair. Returns `{ cohort, date, weight, aggregation, close, total_supply, buckets }` where each bucket is `{ price_floor, supply, realized_cap, unrealized_pnl }`. `close`, `price_floor`, `realized_cap`, and `unrealized_pnl` are USD; `total_supply` and `supply` are BTC. `unrealized_pnl` can be negative.

Endpoint: `GET /api/urpd/{cohort}/{date}`

#### Parameters

##### cohort

[`Cohort`](../type-aliases/Cohort.md)

##### date

`string`

Calendar date of the URPD snapshot in `YYYY-MM-DD` format.

##### agg?

[`UrpdAggregation`](../type-aliases/UrpdAggregation.md)

Aggregation strategy. Default: raw (no aggregation). Accepts `bucket` as alias.

##### weight?

[`UrpdWeight`](../type-aliases/UrpdWeight.md)

Supply weighting. Default: raw (unweighted).

##### options?

###### cache?

`boolean`

###### memCache?

`boolean`

###### onValue?

(`value`) => `void`

###### signal?

`AbortSignal`

#### Returns

`Promise`\<[`Urpd`](../interfaces/Urpd.md)\>

***

### getVersion()

> **getVersion**(`options?`): `Promise`\<`string`\>

Defined in: [Developer/mono/modules/bitview-client/index.js:23280](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L23280)

API version

Returns the current version of the API server

Endpoint: `GET /version`

#### Parameters

##### options?

###### cache?

`boolean`

###### memCache?

`boolean`

###### onValue?

(`value`) => `void`

###### signal?

`AbortSignal`

#### Returns

`Promise`\<`string`\>

***

### indexToDate()

> **indexToDate**(`index`, `i`): `Date`

Defined in: [Developer/mono/modules/bitview-client/index.js:16872](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L16872)

Convert an index value to a Date for date-based indexes.

#### Parameters

##### index

[`Index`](../type-aliases/Index.md)

The index type

##### i

`number`

The index value

#### Returns

`Date`

***

### listSeries()

> **listSeries**(`page?`, `per_page?`, `options?`): `Promise`\<[`PaginatedSeries`](../interfaces/PaginatedSeries.md)\>

Defined in: [Developer/mono/modules/bitview-client/index.js:23367](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L23367)

Series list

Paginated flat list of all available series names. Use `page` query param for pagination.

Endpoint: `GET /api/series/list`

#### Parameters

##### page?

`number`

Pagination index

##### per\_page?

`number`

Results per page (default: 1000, max: 1000)

##### options?

###### cache?

`boolean`

###### memCache?

`boolean`

###### onValue?

(`value`) => `void`

###### signal?

`AbortSignal`

#### Returns

`Promise`\<[`PaginatedSeries`](../interfaces/PaginatedSeries.md)\>

***

### listUrpdCohorts()

> **listUrpdCohorts**(`options?`): `Promise`\<[`Cohort`](../type-aliases/Cohort.md)[]\>

Defined in: [Developer/mono/modules/bitview-client/index.js:23559](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L23559)

Available URPD cohorts

Cohorts for which URPD data is available. Returns names like `all`, `sth`, `lth`, `utxos_under_1h_old`.

Endpoint: `GET /api/urpd`

#### Parameters

##### options?

###### cache?

`boolean`

###### memCache?

`boolean`

###### onValue?

(`value`) => `void`

###### signal?

`AbortSignal`

#### Returns

`Promise`\<[`Cohort`](../type-aliases/Cohort.md)[]\>

***

### listUrpdDates()

> **listUrpdDates**(`cohort`, `weight?`, `options?`): `Promise`\<`string`[]\>

Defined in: [Developer/mono/modules/bitview-client/index.js:23576](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L23576)

Available URPD dates

Dates for which a URPD snapshot is available for the cohort and selected `weight`. One entry per UTC day, sorted ascending.

Endpoint: `GET /api/urpd/{cohort}/dates`

#### Parameters

##### cohort

[`Cohort`](../type-aliases/Cohort.md)

##### weight?

[`UrpdWeight`](../type-aliases/UrpdWeight.md)

Supply weighting. Default: raw (unweighted).

##### options?

###### cache?

`boolean`

###### memCache?

`boolean`

###### onValue?

(`value`) => `void`

###### signal?

`AbortSignal`

#### Returns

`Promise`\<`string`[]\>

***

### post()

> **post**(`path`, `body`, `options?`): `Promise`\<`Response`\>

Defined in: [Developer/mono/modules/bitview-client/index.js:2233](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L2233)

Make a POST request with a string body.

POST responses are uncached and never invoke `onValue` — every call hits
the network with the same body and returns the upstream response.

#### Parameters

##### path

`string`

##### body

`string`

##### options?

###### signal?

`AbortSignal`

#### Returns

`Promise`\<`Response`\>

#### Inherited from

`BitviewClientBase.post`

***

### postBytes()

> **postBytes**(`path`, `body`, `options?`): `Promise`\<`Uint8Array`\<`ArrayBufferLike`\>\>

Defined in: [Developer/mono/modules/bitview-client/index.js:2278](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L2278)

Make a POST request expecting binary data (application/octet-stream).

#### Parameters

##### path

`string`

##### body

`string`

##### options?

###### signal?

`AbortSignal`

#### Returns

`Promise`\<`Uint8Array`\<`ArrayBufferLike`\>\>

#### Inherited from

`BitviewClientBase.postBytes`

***

### postJson()

> **postJson**\<`T`\>(`path`, `body`, `options?`): `Promise`\<`T`\>

Defined in: [Developer/mono/modules/bitview-client/index.js:2254](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L2254)

Make a POST request expecting a JSON response.

#### Type Parameters

##### T

`T`

#### Parameters

##### path

`string`

##### body

`string`

##### options?

###### signal?

`AbortSignal`

#### Returns

`Promise`\<`T`\>

#### Inherited from

`BitviewClientBase.postJson`

***

### postText()

> **postText**(`path`, `body`, `options?`): `Promise`\<`string`\>

Defined in: [Developer/mono/modules/bitview-client/index.js:2266](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L2266)

Make a POST request expecting a text response.

#### Parameters

##### path

`string`

##### body

`string`

##### options?

###### signal?

`AbortSignal`

#### Returns

`Promise`\<`string`\>

#### Inherited from

`BitviewClientBase.postText`

***

### postTx()

> **postTx**(`body`, `options?`): `Promise`\<`string`\>

Defined in: [Developer/mono/modules/bitview-client/index.js:24843](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L24843)

Broadcast transaction

Broadcast a raw transaction to the network. The transaction should be provided as hex in the request body. The txid will be returned on success.

*[Mempool.space docs](https://mempool.space/docs/api/rest#post-transaction)*

Endpoint: `POST /api/tx`

#### Parameters

##### body

`string`

Request body

##### options?

###### signal?

`AbortSignal`

#### Returns

`Promise`\<`string`\>

***

### searchSeries()

> **searchSeries**(`q`, `limit?`, `options?`): `Promise`\<`string`[]\>

Defined in: [Developer/mono/modules/bitview-client/index.js:23388](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L23388)

Search series

Search series by name or descriptive terms. Matches metric names, descriptions, formulas, cohort aliases, partial words, and common typos.

Endpoint: `GET /api/series/search`

#### Parameters

##### q

`string`

Search query string

##### limit?

`number`

Maximum number of results

##### options?

###### cache?

`boolean`

###### memCache?

`boolean`

###### onValue?

(`value`) => `void`

###### signal?

`AbortSignal`

#### Returns

`Promise`\<`string`[]\>

***

### seriesEndpoint()

> **seriesEndpoint**(`series`, `index`): [`SeriesEndpoint`](../interfaces/SeriesEndpoint.md)\<`unknown`\>

Defined in: [Developer/mono/modules/bitview-client/index.js:23253](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L23253)

Create a dynamic series endpoint builder for any series/index combination.

Use this for programmatic access when the series name is determined at runtime.
For type-safe access, use the `series` tree instead.

#### Parameters

##### series

`string`

The series name

##### index

[`Index`](../type-aliases/Index.md)

The index name

#### Returns

[`SeriesEndpoint`](../interfaces/SeriesEndpoint.md)\<`unknown`\>

***

### validateAddress()

> **validateAddress**(`address`, `options?`): `Promise`\<[`AddrValidation`](../interfaces/AddrValidation.md)\>

Defined in: [Developer/mono/modules/bitview-client/index.js:23822](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L23822)

Validate address

Validate a Bitcoin address and get information about its type and scriptPubKey. Returns `isvalid: false` with an error message for invalid addresses.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-address-validate)*

Endpoint: `GET /api/v1/validate-address/{address}`

#### Parameters

##### address

`string`

Bitcoin address to validate (can be any string)

##### options?

###### cache?

`boolean`

###### memCache?

`boolean`

###### onValue?

(`value`) => `void`

###### signal?

`AbortSignal`

#### Returns

`Promise`\<[`AddrValidation`](../interfaces/AddrValidation.md)\>

***

### addressPayloadHashPrefix()

> `static` **addressPayloadHashPrefix**(`payload`, `nibbles`): `string`

Defined in: [Developer/mono/modules/bitview-client/index.js:16905](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L16905)

Compute the RapidHash v3 hash-prefix for raw address payload bytes.

#### Parameters

##### payload

`number`[] \| `ArrayBuffer` \| `Uint8Array`\<`ArrayBufferLike`\> \| `ArrayBufferView`\<`ArrayBufferLike`\>

##### nibbles

`number`

#### Returns

`string`
