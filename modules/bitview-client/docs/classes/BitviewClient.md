[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / BitviewClient

# Class: BitviewClient

Defined in: [modules/bitview-client/index.js:7117](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L7117)

A method per API endpoint, and the typed series tree.

## Extends

- `BitviewClientBase`

## Constructors

### Constructor

> **new BitviewClient**(`options`): `BitviewClient`

Defined in: [modules/bitview-client/index.js:2077](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L2077)

#### Parameters

##### options

`string` \| [`BitviewClientOptions`](../interfaces/BitviewClientOptions.md)

#### Returns

`BitviewClient`

#### Inherited from

`BitviewClientBase.constructor`

## Properties

### \_browserCache

> **\_browserCache**: `Cache` \| `null`

Defined in: [modules/bitview-client/index.js:2088](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L2088)

#### Inherited from

`BitviewClientBase._browserCache`

***

### \_browserCachePromise

> **\_browserCachePromise**: `Promise`\<`Cache` \| `null`\>

Defined in: [modules/bitview-client/index.js:2086](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L2086)

#### Inherited from

`BitviewClientBase._browserCachePromise`

***

### \_memCache

> **\_memCache**: `Map`\<`string`, [`_MemEntry`](../type-aliases/MemEntry.md)\<`unknown`\>\>

Defined in: [modules/bitview-client/index.js:2095](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L2095)

#### Inherited from

`BitviewClientBase._memCache`

## Accessors

### series

#### Get Signature

> **get** **series**(): [`SeriesTree`](../type-aliases/SeriesTree.md)

Defined in: [modules/bitview-client/index.js:7905](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L7905)

##### Returns

[`SeriesTree`](../type-aliases/SeriesTree.md)

## Methods

### \_fetchSeriesData()

> **\_fetchSeriesData**\<`T`\>(`path`, `arg?`, `options?`): `Promise`\<[`DateSeriesData`](../type-aliases/DateSeriesData.md)\<`T`\>\>

Defined in: [modules/bitview-client/index.js:2348](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L2348)

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

> **\_getCached**\<`T`\>(`path`, `kind`, `parse`, `options?`): `Promise`\<`T`\>

Defined in: [modules/bitview-client/index.js:2177](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L2177)

Make a GET request with layered caching.

Contract:
- The returned Promise resolves with the **freshest** value (post-revalidation) and rejects
  when that fails, even if a stale value was already delivered.
- `onValue` fires once with the freshest value, or twice if a stale snapshot
  could be shown first (stale-while-revalidate). On a 304 there is no second fire.
- Responses marked `Cache-Control: no-store` are never cached.

Layers:
- L1 (memCache): in-memory parsed values keyed by body kind + URL, with their ETag. Lets 304s
  skip the parse entirely.
- L2 (browserCache): Cache API, survives reload and feeds onValue fast on cold start.

#### Type Parameters

##### T

`T`

#### Parameters

##### path

`string`

##### kind

`"json"` \| `"text"` \| `"bytes"`

The body reader, part of the memCache key

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

Defined in: [modules/bitview-client/index.js:2103](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L2103)

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

Defined in: [modules/bitview-client/index.js:2117](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L2117)

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

### \_signal()

> **\_signal**(`signal?`): `AbortSignal`

Defined in: [modules/bitview-client/index.js:2132](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L2132)

The request's timeout, joined with the caller's signal when one is given.

#### Parameters

##### signal?

`AbortSignal`

#### Returns

`AbortSignal`

#### Inherited from

`BitviewClientBase._signal`

***

### dateToIndex()

> **dateToIndex**(`index`, `d`): `number`

Defined in: [modules/bitview-client/index.js:7899](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L7899)

The floor index (latest index whose date is <= the given date); throws for dates before the
index's first date.

#### Parameters

##### index

[`Index`](../type-aliases/Index.md)

##### d

`Date`

#### Returns

`number`

***

### get()

> **get**(`path`, `options?`): `Promise`\<`Response`\>

Defined in: [modules/bitview-client/index.js:2142](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L2142)

#### Parameters

##### path

`string`

##### options?

###### cache?

`boolean` = `true`

###### etag?

`string` \| `null`

###### signal?

`AbortSignal`

#### Returns

`Promise`\<`Response`\>

#### Inherited from

`BitviewClientBase.get`

***

### getAddress()

> **getAddress**(`address`, `options?`): `Promise`\<[`AddressStats`](../interfaces/AddressStats.md)\>

Defined in: [modules/bitview-client/index.js:8398](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L8398)

Address information

Balance and transaction counts. Supports P2PKH, P2SH, P2WPKH, P2WSH, P2TR and P2A addresses, and P2PK public keys in hex (33 or 65 bytes).

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

`Promise`\<[`AddressStats`](../interfaces/AddressStats.md)\>

***

### getAddressConfirmedTxs()

> **getAddressConfirmedTxs**(`address`, `max_txs?`, `options?`): `Promise`\<[`Transaction`](../interfaces/Transaction.md)[]\>

Defined in: [modules/bitview-client/index.js:8442](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L8442)

Address confirmed transactions

Get the newest confirmed transactions for an address, up to `max_txs` (25 by default and at most). For pagination, request `GET /api/address/{address}/txs/chain/{after_txid}` with the last returned txid.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-address-transactions-chain)*

Endpoint: `GET /api/address/{address}/txs/chain`

#### Parameters

##### address

`string`

##### max\_txs?

`number`

Transactions to return, at most 25 (the default); larger values return 25.

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

> **getAddressConfirmedTxsAfter**(`address`, `after_txid`, `max_txs?`, `options?`): `Promise`\<[`Transaction`](../interfaces/Transaction.md)[]\>

Defined in: [modules/bitview-client/index.js:8465](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L8465)

Address confirmed transactions (paginated)

Get the next confirmed transactions strictly older than `after_txid`, up to `max_txs` (25 by default and at most); a txid outside the address's history answers `[]` (Esplora-canonical pagination form, matches mempool.space).

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-address-transactions-chain)*

Endpoint: `GET /api/address/{address}/txs/chain/{after_txid}`

#### Parameters

##### address

`string`

##### after\_txid

`string`

Last txid from the previous page (return transactions strictly older than this)

##### max\_txs?

`number`

Transactions to return, at most 25 (the default); larger values return 25.

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

> **getAddressHashPrefixMatches**(`address_type`, `prefix`, `options?`): `Promise`\<[`AddressHashPrefixMatches`](../interfaces/AddressHashPrefixMatches.md)\>

Defined in: [modules/bitview-client/index.js:8380](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L8380)

Address hash-prefix matches

Find addresses by address type and by the first 1-16 hex nibbles of RapidHash v3 over the raw address payload bytes. Intended for privacy-preserving client-side wallet discovery without sending raw addresses or xpubs. Fetch metadata with `GET /api/address/{address}`.

Endpoint: `GET /api/address/hash-prefix/{address_type}/{prefix}`

#### Parameters

##### address\_type

[`AddressOutputType`](../type-aliases/AddressOutputType.md)

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

`Promise`\<[`AddressHashPrefixMatches`](../interfaces/AddressHashPrefixMatches.md)\>

***

### getAddressMempoolTxs()

> **getAddressMempoolTxs**(`address`, `max_txs?`, `options?`): `Promise`\<[`Transaction`](../interfaces/Transaction.md)[]\>

Defined in: [modules/bitview-client/index.js:8487](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L8487)

Address mempool transactions

Get unconfirmed transactions for an address from the mempool, newest first, up to `max_txs` (50 by default and at most).

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-address-transactions-mempool)*

Endpoint: `GET /api/address/{address}/txs/mempool`

#### Parameters

##### address

`string`

##### max\_txs?

`number`

Transactions to return, at most 50 (the default); larger values return 50.

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

> **getAddressPayloadHashPrefixMatches**(`addressType`, `payload`, `nibbles`, `options?`): `Promise`\<[`AddressHashPrefixMatches`](../interfaces/AddressHashPrefixMatches.md)\>

Defined in: [modules/bitview-client/index.js:7926](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L7926)

#### Parameters

##### addressType

[`AddressOutputType`](../type-aliases/AddressOutputType.md)

##### payload

`number`[] \| `ArrayBuffer` \| `Uint8Array`\<`ArrayBufferLike`\> \| `ArrayBufferView`\<`ArrayBufferLike`\>

Raw payload bytes matching addressType length

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

`Promise`\<[`AddressHashPrefixMatches`](../interfaces/AddressHashPrefixMatches.md)\>

***

### getAddressTxs()

> **getAddressTxs**(`address`, `after_txid?`, `max_txs?`, `options?`): `Promise`\<[`Transaction`](../interfaces/Transaction.md)[]\>

Defined in: [modules/bitview-client/index.js:8419](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L8419)

Address transactions

Get transaction history for an address, newest first: up to `max_txs` (50 by default and at most), mempool transactions first and confirmed ones filling the rest. To continue, pass the last returned txid as `after_txid`, or request `GET /api/address/{address}/txs/chain/{after_txid}` for confirmed history.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-address-transactions)*

Endpoint: `GET /api/address/{address}/txs`

#### Parameters

##### address

`string`

##### after\_txid?

`string`

Continue after this transaction, a mempool or a confirmed one: the older ones (a confirmed
cursor skips the mempool). One that is neither answers 422.

##### max\_txs?

`number`

Transactions to return, at most 50 (the default); larger values return 50.

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

Defined in: [modules/bitview-client/index.js:8508](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L8508)

Address UTXOs

Get unspent transaction outputs (UTXOs) for an address, including unconfirmed ones; outputs that mempool transactions spend are left out.

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

Defined in: [modules/bitview-client/index.js:9673](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L9673)

Compact API summary

Compact custom summary of the OpenAPI specification for LLM reading: types instead of schemas, `returns` instead of responses, no examples; parameters keep their enum, default and bounds. Not valid OpenAPI: build tools from `GET /openapi.json`.

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

Defined in: [modules/bitview-client/index.js:8543](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L8543)

Block information

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

### getBlockByTimestamp()

> **getBlockByTimestamp**(`timestamp`, `options?`): `Promise`\<[`BlockTimestamp`](../interfaces/BlockTimestamp.md)\>

Defined in: [modules/bitview-client/index.js:8615](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L8615)

Block by timestamp

Find the block with the greatest header timestamp at or before the given UNIX timestamp, choosing the earliest height on ties.

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

Defined in: [modules/bitview-client/index.js:9098](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L9098)

Block fee rates

Get block fee rate percentiles (min, 10th, 25th, median, 75th, 90th, max) for a time period, each a whole sat/vB (`/precise` for 3 decimals).

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-feerates)*

Endpoint: `GET /api/v1/mining/blocks/fee-rates/{time_period}`

#### Parameters

##### time\_period

[`TimePeriod`](../type-aliases/TimePeriod.md)

Lookback period: `24h`, `3d`, `1w`, `1m`, `3m`, `6m`, `1y`, `2y`, `3y`, `4y` or `all`.

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

### getBlockFeeRatesPrecise()

> **getBlockFeeRatesPrecise**(`time_period`, `options?`): `Promise`\<[`BlockFeeRatesPreciseEntry`](../interfaces/BlockFeeRatesPreciseEntry.md)[]\>

Defined in: [modules/bitview-client/index.js:9116](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L9116)

Block fee rates (precise)

Get block fee rate percentiles (min, 10th, 25th, median, 75th, 90th, max) for a time period, each in sat/vB to 3 decimals.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-feerates)*

Endpoint: `GET /api/v1/mining/blocks/fee-rates/{time_period}/precise`

#### Parameters

##### time\_period

[`TimePeriod`](../type-aliases/TimePeriod.md)

Lookback period: `24h`, `3d`, `1w`, `1m`, `3m`, `6m`, `1y`, `2y`, `3y`, `4y` or `all`.

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

`Promise`\<[`BlockFeeRatesPreciseEntry`](../interfaces/BlockFeeRatesPreciseEntry.md)[]\>

***

### getBlockFees()

> **getBlockFees**(`time_period`, `options?`): `Promise`\<[`BlockFeesEntry`](../interfaces/BlockFeesEntry.md)[]\>

Defined in: [modules/bitview-client/index.js:9062](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L9062)

Block fees

Get average total fees per block for a time period.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-fees)*

Endpoint: `GET /api/v1/mining/blocks/fees/{time_period}`

#### Parameters

##### time\_period

[`TimePeriod`](../type-aliases/TimePeriod.md)

Lookback period: `24h`, `3d`, `1w`, `1m`, `3m`, `6m`, `1y`, `2y`, `3y`, `4y` or `all`.

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

### getBlockHashByHeight()

> **getBlockHashByHeight**(`height`, `options?`): `Promise`\<`string`\>

Defined in: [modules/bitview-client/index.js:8597](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L8597)

Block hash by height

Block hash as plain text.

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

### getBlockHeader()

> **getBlockHeader**(`hash`, `options?`): `Promise`\<`string`\>

Defined in: [modules/bitview-client/index.js:8579](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L8579)

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

Defined in: [modules/bitview-client/index.js:8633](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L8633)

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

Defined in: [modules/bitview-client/index.js:9080](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L9080)

Block rewards

Get average coinbase reward (subsidy + fees) per block for a time period.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-rewards)*

Endpoint: `GET /api/v1/mining/blocks/rewards/{time_period}`

#### Parameters

##### time\_period

[`TimePeriod`](../type-aliases/TimePeriod.md)

Lookback period: `24h`, `3d`, `1w`, `1m`, `3m`, `6m`, `1y`, `2y`, `3y`, `4y` or `all`.

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

Defined in: [modules/bitview-client/index.js:8769](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L8769)

Recent blocks

The last 10 blocks.

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

Defined in: [modules/bitview-client/index.js:8787](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L8787)

Blocks from height

Retrieve up to 10 blocks going backwards from the given height. For example, height=100 returns blocks 100, 99, 98, ..., 91. Height=0 returns only block 0. A height past the tip is not found.

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

Defined in: [modules/bitview-client/index.js:9134](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L9134)

Block sizes and weights

Get average block sizes and weights for a time period.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-sizes-weights)*

Endpoint: `GET /api/v1/mining/blocks/sizes-weights/{time_period}`

#### Parameters

##### time\_period

[`TimePeriod`](../type-aliases/TimePeriod.md)

Lookback period: `24h`, `3d`, `1w`, `1m`, `3m`, `6m`, `1y`, `2y`, `3y`, `4y` or `all`.

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

Defined in: [modules/bitview-client/index.js:8651](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L8651)

Block status

Whether the block is in the best chain and, if so, its height and the hash of the next block. A block outside the best chain (orphaned or unknown) reports `in_best_chain: false`.

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

Defined in: [modules/bitview-client/index.js:8803](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L8803)

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

Defined in: [modules/bitview-client/index.js:8821](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L8821)

Blocks from height with extras

Retrieve up to 15 blocks with extended data going backwards from the given height: the published blocks among the 15 heights ending at it, so fewer (or none) past the tip.

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

### getBlockTipHash()

> **getBlockTipHash**(`options?`): `Promise`\<`string`\>

Defined in: [modules/bitview-client/index.js:8679](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L8679)

Block tip hash

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

Defined in: [modules/bitview-client/index.js:8665](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L8665)

Block tip height

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

Defined in: [modules/bitview-client/index.js:8698](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L8698)

Transaction ID at index

Retrieve a single transaction ID at a specific index within a block. Returns plain text txid.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-transaction-id)*

Endpoint: `GET /api/block/{hash}/txid/{index}`

#### Parameters

##### hash

`string`

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

Defined in: [modules/bitview-client/index.js:8716](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L8716)

Block transaction IDs

Transaction IDs in block order.

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

Defined in: [modules/bitview-client/index.js:8734](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L8734)

Block transactions

Up to 25 transactions starting at index 0.

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

Defined in: [modules/bitview-client/index.js:8753](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L8753)

Block transactions (paginated)

Up to 25 transactions starting at `start_index`, a multiple of 25.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-transactions)*

Endpoint: `GET /api/block/{hash}/txs/{start_index}`

#### Parameters

##### hash

`string`

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

Defined in: [modules/bitview-client/index.js:8561](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L8561)

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

Defined in: [modules/bitview-client/index.js:2286](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L2286)

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

> **getCpfp**(`txid`, `options?`): `Promise`\<[`Cpfp`](../type-aliases/Cpfp.md)\>

Defined in: [modules/bitview-client/index.js:9327](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L9327)

CPFP info

Returns ancestors and descendants for a CPFP (Child Pays For Parent) transaction, including the effective fee rate of the package. A mempool transaction answers its live package; a confirmed one its package among its block's transactions; an unknown transaction, or a confirmed one without relatives in its block, `{"ancestors":[]}`.

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

`Promise`\<[`Cpfp`](../type-aliases/Cpfp.md)\>

***

### getDifficultyAdjustment()

> **getDifficultyAdjustment**(`options?`): `Promise`\<[`DifficultyAdjustment`](../interfaces/DifficultyAdjustment.md)\>

Defined in: [modules/bitview-client/index.js:8323](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L8323)

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

Defined in: [modules/bitview-client/index.js:9008](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L9008)

Difficulty adjustments (all time)

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

Defined in: [modules/bitview-client/index.js:9026](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L9026)

Difficulty adjustments

Get historical difficulty adjustments for a time period.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-difficulty-adjustments)*

Endpoint: `GET /api/v1/mining/difficulty-adjustments/{time_period}`

#### Parameters

##### time\_period

[`TimePeriod`](../type-aliases/TimePeriod.md)

Lookback period: `24h`, `3d`, `1w`, `1m`, `3m`, `6m`, `1y`, `2y`, `3y`, `4y` or `all`.

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

Defined in: [modules/bitview-client/index.js:8013](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L8013)

Disk usage

Returns allocated file bytes (file lengths on non-Unix platforms) for BRK and Bitcoin data. Each request scans both trees; these are independent observations, not an atomic filesystem snapshot. Conditional requests validate the newly observed totals. Directory-link cycles and excessive nesting fail without returning partial totals.

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

Defined in: [modules/bitview-client/index.js:9260](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L9260)

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

Defined in: [modules/bitview-client/index.js:8976](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L8976)

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

Defined in: [modules/bitview-client/index.js:8994](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L8994)

Network hashrate

Get network hashrate and difficulty data for a time period.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-hashrate)*

Endpoint: `GET /api/v1/mining/hashrate/{time_period}`

#### Parameters

##### time\_period

[`TimePeriod`](../type-aliases/TimePeriod.md)

Lookback period: `24h`, `3d`, `1w`, `1m`, `3m`, `6m`, `1y`, `2y`, `3y`, `4y` or `all`.

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

Defined in: [modules/bitview-client/index.js:7952](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L7952)

Health check

Local health and query-readiness check. Returns server identity, uptime, and a coherent local sync snapshot without a bitcoind round-trip. Reads the published prefix during processing; an empty index returns 503 `state_updating` at once. Responses are not cached. For chain-tip catch-up, request `GET /api/server/sync`.

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

> **getHistoricalPrice**(`timestamp?`, `currency?`, `options?`): `Promise`\<[`HistoricalPrice`](../interfaces/HistoricalPrice.md)\>

Defined in: [modules/bitview-client/index.js:8358](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L8358)

Historical price

Completed four-hour BTC/USD closes, newest first as mempool.space lists them, labeled by interval end, from the first priced interval (2010-08-16). With a UNIX timestamp, returns the latest nonempty completed close at or before it; before the first close returns an empty list. The current partial interval is excluded. USD only (`currency` accepts `USD`); exchangeRates is empty.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-historical-price)*

Endpoint: `GET /api/v1/historical-price`

#### Parameters

##### timestamp?

`number`

UNIX timestamp: the latest close at or before it

##### currency?

`"USD"`

Quote currency: USD, the only one served

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

### getJson()

> **getJson**\<`T`\>(`path`, `options?`, `keep?`): `Promise`\<`T`\>

Defined in: [modules/bitview-client/index.js:2266](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L2266)

Cached and supports `onValue`. Field names come camelCased, map keys as sent (`keep`).

#### Type Parameters

##### T

`T`

#### Parameters

##### path

`string`

##### options?

[`ClientFetchOptions`](../interfaces/ClientFetchOptions.md)\<`T`\>

##### keep?

`any`

Where the response has maps, as `_camelKeys` takes it

#### Returns

`Promise`\<`T`\>

#### Inherited from

`BitviewClientBase.getJson`

***

### getLivePrice()

> **getLivePrice**(`options?`): `Promise`\<`number`\>

Defined in: [modules/bitview-client/index.js:9585](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L9585)

Live BTC/USD price

Current BTC/USD price in dollars, derived from on-chain round-dollar output patterns in the oracle's recent-block window, with the mempool's outputs blended in when available. Confirmed per-height history is available at `GET /api/series/price/height`.

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

### getLogs()

> **getLogs**(`started_unix_ms?`, `after?`, `options?`): `Promise`\<[`Logs`](../interfaces/Logs.md)\>

Defined in: [modules/bitview-client/index.js:7995](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L7995)

Logs

The server's last log lines, at most 1,000, oldest first: requests (status, path and duration; never who made them) and everything else it logs. To read on, pass the answer's `started_unix_ms` and its last line's `seq` as `after`: the lines since then, each once. A jump in `seq` shows lines that left memory before the next request; another `started_unix_ms` shows the server restarted. Answers are cached for a second.

Endpoint: `GET /api/server/logs`

#### Parameters

##### started\_unix\_ms?

`number`

The `started_unix_ms` of the last answer: `after` counts in that process's log.

##### after?

`number`

Only the lines after this `seq`, when `started_unix_ms` is this process's; otherwise every kept line.

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

`Promise`\<[`Logs`](../interfaces/Logs.md)\>

***

### getMempool()

> **getMempool**(`options?`): `Promise`\<[`MempoolInfo`](../interfaces/MempoolInfo.md)\>

Defined in: [modules/bitview-client/index.js:9196](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L9196)

Mempool statistics

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

### getMempoolBlock()

> **getMempoolBlock**(`index`, `options?`): `Promise`\<[`MempoolBlockContent`](../interfaces/MempoolBlockContent.md)\>

Defined in: [modules/bitview-client/index.js:9276](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L9276)

Projected mempool block

A projected mempool block with its transactions in order and its stats, by position as `GET /api/v1/fees/mempool-blocks` lists them: 0 is the next block, Bitcoin Core's `getblocktemplate` selection, and the only one served for now (others answer 404). The returned `hash` is an opaque content token: poll `GET /api/mempool-blocks/{index}/diff/{hash}` to follow changes instead of refetching the block.

Endpoint: `GET /api/mempool-blocks/{index}`

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

`Promise`\<[`MempoolBlockContent`](../interfaces/MempoolBlockContent.md)\>

***

### getMempoolBlockDiff()

> **getMempoolBlockDiff**(`index`, `since`, `options?`): `Promise`\<[`MempoolBlockDiff`](../interfaces/MempoolBlockDiff.md)\>

Defined in: [modules/bitview-client/index.js:9293](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L9293)

Projected mempool block changes

The projected mempool block's changes since the client's copy at `since`. `order` is the new block in order: each entry is either `[start, count]`, the client's transactions `start..start + count` kept in order, or a transaction object, a body to place there. Walk `order` once to rebuild, and replace the cached `stats` with the response's; `removed` lists the txids that left, so clients can evict cached bodies. Use the response `hash` as the next `since`; with `If-None-Match` an unchanged block answers 304. Returns `404` when `since` has aged out of server history: fetch `GET /api/mempool-blocks/{index}` again.

Endpoint: `GET /api/mempool-blocks/{index}/diff/{since}`

#### Parameters

##### index

`number`

##### since

`string`

The `hash` of the client's copy, from the block or a previous diff

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

`Promise`\<[`MempoolBlockDiff`](../interfaces/MempoolBlockDiff.md)\>

***

### getMempoolBlocks()

> **getMempoolBlocks**(`options?`): `Promise`\<[`MempoolBlock`](../interfaces/MempoolBlock.md)[]\>

Defined in: [modules/bitview-client/index.js:9150](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L9150)

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

### getMempoolRecent()

> **getMempoolRecent**(`options?`): `Promise`\<[`MempoolRecentTx`](../interfaces/MempoolRecentTx.md)[]\>

Defined in: [modules/bitview-client/index.js:9228](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L9228)

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

Defined in: [modules/bitview-client/index.js:9212](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L9212)

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

Defined in: [modules/bitview-client/index.js:9659](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L9659)

OpenAPI specification

Full OpenAPI 3.1 specification for this API. Build tools from this document.

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

Defined in: [modules/bitview-client/index.js:9645](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L9645)

Output value histogram at height or day

Unfiltered output value histogram for a confirmed point. A block height (`840000`) counts that block's nonzero outputs within the histogram's bin range, coinbase included, by value on the oracle log scale; a calendar date (`YYYY-MM-DD`) sums every block that day. A flat array of log-scale bins.

Endpoint: `GET /api/oracle/histogram/outputs/{point}`

#### Parameters

##### point

[`HeightOrDate`](../type-aliases/HeightOrDate.md)

Confirmed block height (`840000`) or UTC calendar day (`YYYY-MM-DD`)

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

Defined in: [modules/bitview-client/index.js:9629](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L9629)

Live output value histogram

Live unfiltered output value histogram for the complete published mempool. Nonzero outputs within the histogram's bin range are counted by value on the oracle log scale; no oracle payment filters are applied. A flat array of log-scale bins, all zero when no mempool is configured.

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

Defined in: [modules/bitview-client/index.js:9615](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L9615)

Payment output histogram at height or day

Smoothed histogram of oracle-eligible payment outputs for a confirmed point. A block height (`840000`) gives that block's oracle payment histogram; a calendar date (`YYYY-MM-DD`) gives the average of that day's per-block payment histograms. A flat array of log-scale bins.

Endpoint: `GET /api/oracle/histogram/payments/{point}`

#### Parameters

##### point

[`HeightOrDate`](../type-aliases/HeightOrDate.md)

Confirmed block height (`840000`) or UTC calendar day (`YYYY-MM-DD`)

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

Defined in: [modules/bitview-client/index.js:9599](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L9599)

Live payment output histogram

Live smoothed histogram of oracle-eligible payment outputs, binned by output value on the oracle log scale. It combines the committed oracle window with the complete mempool's eligible outputs from a matching chain publication. A flat array of log-scale bins.

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

### getPool()

> **getPool**(`slug`, `options?`): `Promise`\<[`PoolDetail`](../interfaces/PoolDetail.md)\>

Defined in: [modules/bitview-client/index.js:8871](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L8871)

Mining pool details

Block counts and mining shares by time period for one pool.

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

Defined in: [modules/bitview-client/index.js:8941](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L8941)

Mining pool blocks

Get up to 100 recent blocks mined by a specific pool.

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

### getPoolBlocksFromHeight()

> **getPoolBlocksFromHeight**(`slug`, `height`, `options?`): `Promise`\<[`BlockInfoV1`](../interfaces/BlockInfoV1.md)[]\>

Defined in: [modules/bitview-client/index.js:8960](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L8960)

Mining pool blocks before height

Get up to 100 blocks mined by a specific pool strictly below the given height (pass the last height of a page to get the next one).

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

Defined in: [modules/bitview-client/index.js:8923](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L8923)

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

### getPoolRanking()

> **getPoolRanking**(`time_period`, `options?`): `Promise`\<[`PoolsSummary`](../interfaces/PoolsSummary.md)\>

Defined in: [modules/bitview-client/index.js:8853](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L8853)

Mining pool statistics

Get mining pool statistics for a time period.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-mining-pools)*

Endpoint: `GET /api/v1/mining/pools/{time_period}`

#### Parameters

##### time\_period

[`TimePeriod`](../type-aliases/TimePeriod.md)

Lookback period: `24h`, `3d`, `1w`, `1m`, `3m`, `6m`, `1y`, `2y`, `3y`, `4y` or `all`.

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

### getPools()

> **getPools**(`options?`): `Promise`\<[`PoolInfo`](../interfaces/PoolInfo.md)[]\>

Defined in: [modules/bitview-client/index.js:8835](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L8835)

List all mining pools

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

Defined in: [modules/bitview-client/index.js:8887](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L8887)

All pools hashrate (all time)

Get weekly hashrate data, over the whole chain, of the mining pools that mined in the last week, by time and then their weekly rank.

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

Defined in: [modules/bitview-client/index.js:8905](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L8905)

All pools hashrate

Get weekly hashrate data, for a time period, of the mining pools that mined in the last week, by time and then their weekly rank.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-mining-pool-hashrates)*

Endpoint: `GET /api/v1/mining/hashrate/pools/{time_period}`

#### Parameters

##### time\_period

[`TimePeriod`](../type-aliases/TimePeriod.md)

Lookback period: `24h`, `3d`, `1w`, `1m`, `3m`, `6m`, `1y`, `2y`, `3y`, `4y` or `all`.

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

### getPreciseFees()

> **getPreciseFees**(`options?`): `Promise`\<[`PreciseRecommendedFees`](../interfaces/PreciseRecommendedFees.md)\>

Defined in: [modules/bitview-client/index.js:9182](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L9182)

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

`Promise`\<[`PreciseRecommendedFees`](../interfaces/PreciseRecommendedFees.md)\>

***

### getPrices()

> **getPrices**(`options?`): `Promise`\<[`PricePoint`](../interfaces/PricePoint.md)\>

Defined in: [modules/bitview-client/index.js:8339](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L8339)

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

`Promise`\<[`PricePoint`](../interfaces/PricePoint.md)\>

***

### getRecommendedFees()

> **getRecommendedFees**(`options?`): `Promise`\<[`RecommendedFees`](../interfaces/RecommendedFees.md)\>

Defined in: [modules/bitview-client/index.js:9166](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L9166)

Recommended fees

Recommended fee rates by confirmation target, in whole sat/vB.

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

Defined in: [modules/bitview-client/index.js:9244](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L9244)

Recent RBF replacements

Returns up to 25 most-recent RBF replacement trees across the whole mempool. Each entry has the shape of `replacements` in `GET /api/v1/tx/{txid}/rbf`.

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

Defined in: [modules/bitview-client/index.js:9044](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L9044)

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

Defined in: [modules/bitview-client/index.js:8134](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L8134)

Get series data

Rows selected by a range of positions, dates or timestamps, as JSON or CSV.

Endpoint: `GET /api/series/{series}/{index}`

#### Parameters

##### series

`string`

##### index

[`Index`](../type-aliases/Index.md)

##### start?

[`RangeIndex`](../type-aliases/RangeIndex.md)

Inclusive start: integer index, date (YYYY-MM-DD), or timestamp (ISO 8601). Negative integers count from end.

##### end?

[`RangeIndex`](../type-aliases/RangeIndex.md)

Exclusive end: integer index, date (YYYY-MM-DD), or timestamp (ISO 8601). Negative integers count from end.

##### limit?

`number`

Maximum number of values to return; with `end` too, the tighter of the two applies. Omitted, the rest of the range.

##### format?

[`Format`](../type-aliases/Format.md)

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

Defined in: [modules/bitview-client/index.js:8239](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L8239)

Bulk series data

Fetch multiple series in a single request. Supports filtering by index and date range. Returns an array of SeriesData objects. For a single series, use `get_series` instead.

Endpoint: `GET /api/series/bulk`

#### Parameters

##### series

`string`

##### index

[`Index`](../type-aliases/Index.md)

##### start?

[`RangeIndex`](../type-aliases/RangeIndex.md)

Inclusive start: integer index, date (YYYY-MM-DD), or timestamp (ISO 8601). Negative integers count from end.

##### end?

[`RangeIndex`](../type-aliases/RangeIndex.md)

Exclusive end: integer index, date (YYYY-MM-DD), or timestamp (ISO 8601). Negative integers count from end.

##### limit?

`number`

Maximum number of values to return; with `end` too, the tighter of the two applies. Omitted, the rest of the range.

##### format?

[`Format`](../type-aliases/Format.md)

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

Defined in: [modules/bitview-client/index.js:8041](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L8041)

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

> **getSeriesData**(`series`, `index`, `start?`, `end?`, `limit?`, `format?`, `options?`): `Promise`\<`string` \| `any`[]\>

Defined in: [modules/bitview-client/index.js:8162](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L8162)

Get raw series data

The data array without the `SeriesData` wrapper. Supports the same range and format parameters as `GET /api/series/{series}/{index}`.

Endpoint: `GET /api/series/{series}/{index}/data`

#### Parameters

##### series

`string`

##### index

[`Index`](../type-aliases/Index.md)

##### start?

[`RangeIndex`](../type-aliases/RangeIndex.md)

Inclusive start: integer index, date (YYYY-MM-DD), or timestamp (ISO 8601). Negative integers count from end.

##### end?

[`RangeIndex`](../type-aliases/RangeIndex.md)

Exclusive end: integer index, date (YYYY-MM-DD), or timestamp (ISO 8601). Negative integers count from end.

##### limit?

`number`

Maximum number of values to return; with `end` too, the tighter of the two applies. Omitted, the rest of the range.

##### format?

[`Format`](../type-aliases/Format.md)

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

`Promise`\<`string` \| `any`[]\>

***

### getSeriesInfo()

> **getSeriesInfo**(`series`, `options?`): `Promise`\<[`SeriesInfo`](../interfaces/SeriesInfo.md)\>

Defined in: [modules/bitview-client/index.js:8113](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L8113)

Get series info

Returns the optional description, supported indexes, the indexes whose values can be null, the value type, and the optional unit (what the value type measures) for the specified series. The decoded series name is limited to 1024 UTF-8 bytes.

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

Defined in: [modules/bitview-client/index.js:8186](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L8186)

Get latest series value

Returns the single most recent value for a series, unwrapped (not inside a SeriesData object).

Endpoint: `GET /api/series/{series}/{index}/latest`

#### Parameters

##### series

`string`

##### index

[`Index`](../type-aliases/Index.md)

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

Defined in: [modules/bitview-client/index.js:8201](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L8201)

Get series data length

Endpoint: `GET /api/series/{series}/{index}/len`

#### Parameters

##### series

`string`

##### index

[`Index`](../type-aliases/Index.md)

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

Defined in: [modules/bitview-client/index.js:8027](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L8027)

Series catalog

The available series as a tree of categories and subcategories.

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

Defined in: [modules/bitview-client/index.js:8218](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L8218)

Get series version

Returns the vector's schema/computation version, not its length or latest update. Appends and reorgs do not by themselves change this version.

Endpoint: `GET /api/series/{series}/{index}/version`

#### Parameters

##### series

`string`

##### index

[`Index`](../type-aliases/Index.md)

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

Defined in: [modules/bitview-client/index.js:7978](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L7978)

Sync status

Returns a coherent local index snapshot and a separately observed Bitcoin Core tip height. The two heights can differ during indexing or a reorg. Conditional requests refresh these observations before validation.

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

Defined in: [modules/bitview-client/index.js:2276](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L2276)

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

> **getTransactionTimes**(`txids`, `options?`): `Promise`\<`number`[]\>

Defined in: [modules/bitview-client/index.js:9504](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L9504)

Transaction first-seen times

First-seen UNIX timestamps in seconds, in request order, of transactions in the mempool or that recently left it other than by replacement (mined ones included); 0 otherwise.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-transaction-times)*

Endpoint: `GET /api/v1/transaction-times`

#### Parameters

##### txids

`string`[]

Transaction IDs to look up (max 100 per request).

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

Defined in: [modules/bitview-client/index.js:9361](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L9361)

Transaction information

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

### getTxHex()

> **getTxHex**(`txid`, `options?`): `Promise`\<`string`\>

Defined in: [modules/bitview-client/index.js:9379](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L9379)

Transaction hex

Hex-encoded serialized transaction.

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

### getTxidByIndex()

> **getTxidByIndex**(`index`, `options?`): `Promise`\<`string`\>

Defined in: [modules/bitview-client/index.js:9309](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L9309)

Txid by index

Retrieve the transaction ID (txid) at a given global transaction index. Returns the txid as plain text.

Endpoint: `GET /api/tx-index/{index}`

#### Parameters

##### index

`number`

Global transaction index across the chain

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

Defined in: [modules/bitview-client/index.js:9397](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L9397)

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

Defined in: [modules/bitview-client/index.js:9413](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L9413)

Transaction merkle proof

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

Defined in: [modules/bitview-client/index.js:9432](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L9432)

Output spend status

Whether the output has been spent and, if so, the spending transaction details. An output no known transaction holds is unspent.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-transaction-outspend)*

Endpoint: `GET /api/tx/{txid}/outspend/{vout}`

#### Parameters

##### txid

`string`

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

Defined in: [modules/bitview-client/index.js:9450](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L9450)

All output spend statuses

Spend statuses in output-index order.

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

Defined in: [modules/bitview-client/index.js:9468](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L9468)

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

Defined in: [modules/bitview-client/index.js:9345](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L9345)

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

Defined in: [modules/bitview-client/index.js:9486](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L9486)

Transaction status

Retrieve the confirmation status of a transaction. Returns whether the transaction is confirmed and, if so, the block height, hash, and timestamp. A transaction not in the chain, pending or unknown, is `{"confirmed":false}`.

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

> **getUrpd**(`cohort`, `point`, `agg?`, `weight?`, `options?`): `Promise`\<[`Urpd`](../interfaces/Urpd.md)\>

Defined in: [modules/bitview-client/index.js:8303](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L8303)

URPD at block height or date

URPD for a cohort at a block height or the last block of a UTC day. Returns `{ height, hash, date, cohort, weight, aggregation, close, supply, buckets }` where `buckets` is columns, `{ price, supply, realized_cap, unrealized_pnl }`, one entry per bucket in ascending price: the same block header and columnar layout as `GET /api/utxo-set/{point}`. `close`, `price`, `realized_cap`, and `unrealized_pnl` are USD; `supply` is BTC. `unrealized_pnl` can be negative. Bucket `realized_cap` and `unrealized_pnl` value each coin at its creation price on the price grid (four significant digits of the price in cents, within 0.05% of the exact price), rounded to the cent per bucket, so the buckets need not sum to the realized cap series. With a `wakefulness` or `mobility` `weight`, `supply` is the weighted supply (whole satoshis) and the bucket values follow it.

Endpoint: `GET /api/urpd/{cohort}/{point}`

#### Parameters

##### cohort

[`UrpdCohort`](../type-aliases/UrpdCohort.md)

##### point

[`HeightOrDate`](../type-aliases/HeightOrDate.md)

Confirmed block height (`840000`) or UTC calendar day (`YYYY-MM-DD`)

##### agg?

[`UrpdAggregation`](../type-aliases/UrpdAggregation.md)

Aggregation strategy. Default: raw (no aggregation).

##### weight?

[`UrpdWeight`](../type-aliases/UrpdWeight.md)

Supply weighting. Default: none (unweighted).

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

### getUrpdLatest()

> **getUrpdLatest**(`cohort`, `agg?`, `weight?`, `options?`): `Promise`\<[`Urpd`](../interfaces/Urpd.md)\>

Defined in: [modules/bitview-client/index.js:8280](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L8280)

Latest URPD

URPD for the latest published block; `date` echoes the date served. Fields, units, price-grid rounding and weighting as in `GET /api/urpd/{cohort}/{point}`.

Endpoint: `GET /api/urpd/{cohort}`

#### Parameters

##### cohort

[`UrpdCohort`](../type-aliases/UrpdCohort.md)

##### agg?

[`UrpdAggregation`](../type-aliases/UrpdAggregation.md)

Aggregation strategy. Default: raw (no aggregation).

##### weight?

[`UrpdWeight`](../type-aliases/UrpdWeight.md)

Supply weighting. Default: none (unweighted).

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

### getUtxoSet()

> **getUtxoSet**(`point`, `options?`): `Promise`\<[`UtxoSet`](../interfaces/UtxoSet.md)\>

Defined in: [modules/bitview-client/index.js:9555](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L9555)

UTXO set at block height or date

The UTXO set after a block (`840000`) or after the last published block of a UTC day (`YYYY-MM-DD`), by creation height. Returns `{ height, hash, date, count, supply, origins }`; entry `i` of `origins.count` and `origins.supply` (BTC) is what remains unspent of block `i`'s outputs.

Endpoint: `GET /api/utxo-set/{point}`

#### Parameters

##### point

[`HeightOrDate`](../type-aliases/HeightOrDate.md)

Confirmed block height (`840000`) or UTC calendar day (`YYYY-MM-DD`)

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

`Promise`\<[`UtxoSet`](../interfaces/UtxoSet.md)\>

***

### getUtxoSetDiff()

> **getUtxoSetDiff**(`point`, `options?`): `Promise`\<[`UtxoSetDiff`](../interfaces/UtxoSetDiff.md)\>

Defined in: [modules/bitview-client/index.js:9571](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L9571)

UTXO set changes of a block or date

What a block (`840000`) or a UTC day's published blocks (`YYYY-MM-DD`) changed in the UTXO set. Returns `{ first, last, hash, date, created, spent }`: `created` has one row per block, `spent` one row per creation height, each as columnar `height`, `count` and `supply` (BTC). The set after block `first - 1` plus `created` minus `spent` is the set after block `last`.

Endpoint: `GET /api/utxo-set/{point}/diff`

#### Parameters

##### point

[`HeightOrDate`](../type-aliases/HeightOrDate.md)

Confirmed block height (`840000`) or UTC calendar day (`YYYY-MM-DD`)

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

`Promise`\<[`UtxoSetDiff`](../interfaces/UtxoSetDiff.md)\>

***

### getUtxoSetLatest()

> **getUtxoSetLatest**(`options?`): `Promise`\<[`UtxoSet`](../interfaces/UtxoSet.md)\>

Defined in: [modules/bitview-client/index.js:9539](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L9539)

Latest UTXO set

The UTXO set after the latest published block, in the layout of `GET /api/utxo-set/{point}`.

Endpoint: `GET /api/utxo-set`

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

`Promise`\<[`UtxoSet`](../interfaces/UtxoSet.md)\>

***

### getVersion()

> **getVersion**(`options?`): `Promise`\<`string`\>

Defined in: [modules/bitview-client/index.js:7964](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L7964)

API version

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

Defined in: [modules/bitview-client/index.js:7888](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L7888)

The UTC start of bucket `i`; throws for an index without a date calendar.

#### Parameters

##### index

[`Index`](../type-aliases/Index.md)

##### i

`number`

#### Returns

`Date`

***

### listIndexes()

> **listIndexes**(`options?`): `Promise`\<[`Index`](../type-aliases/Index.md)[]\>

Defined in: [modules/bitview-client/index.js:8055](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L8055)

List available indexes

Returns every index a series can be read at, by the name series routes take.

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

`Promise`\<[`Index`](../type-aliases/Index.md)[]\>

***

### listSeries()

> **listSeries**(`page?`, `per_page?`, `options?`): `Promise`\<[`PaginatedSeries`](../interfaces/PaginatedSeries.md)\>

Defined in: [modules/bitview-client/index.js:8072](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L8072)

Series list

Paginated flat list of all available series names.

Endpoint: `GET /api/series/list`

#### Parameters

##### page?

`number`

Zero-based page number (default: 0)

##### per\_page?

`number`

Results per page, 1 to 1,000 (default: 1000)

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

> **listUrpdCohorts**(`options?`): `Promise`\<[`UrpdCohort`](../type-aliases/UrpdCohort.md)[]\>

Defined in: [modules/bitview-client/index.js:8262](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L8262)

Available URPD cohorts

Every URPD cohort once a UTXO set is published (none before): the age aggregates (`all`, `sth`, `lth`, `age_under_4m`, ...), then the age bands from the youngest (`age_under_1h`, `age_1h_to_1d`, ...).

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

`Promise`\<[`UrpdCohort`](../type-aliases/UrpdCohort.md)[]\>

***

### post()

> **post**(`path`, `body`, `options?`): `Promise`\<`Response`\>

Defined in: [modules/bitview-client/index.js:2299](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L2299)

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

Defined in: [modules/bitview-client/index.js:2336](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L2336)

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

> **postJson**\<`T`\>(`path`, `body`, `options?`, `keep?`): `Promise`\<`T`\>

Defined in: [modules/bitview-client/index.js:2314](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L2314)

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

##### keep?

`any`

Where the response has maps, as `_camelKeys` takes it

#### Returns

`Promise`\<`T`\>

#### Inherited from

`BitviewClientBase.postJson`

***

### postText()

> **postText**(`path`, `body`, `options?`): `Promise`\<`string`\>

Defined in: [modules/bitview-client/index.js:2325](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L2325)

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

Defined in: [modules/bitview-client/index.js:9525](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L9525)

Broadcast transaction

Submit a raw transaction as hexadecimal text (at most 8,000,000 request bytes, including whitespace). Returns its txid as plain text. No responses are cached. Cancellation or a transport error after dispatch may leave the submission outcome unknown; do not automatically retry.

*[Mempool.space docs](https://mempool.space/docs/api/rest#post-transaction)*

Endpoint: `POST /api/tx`

#### Parameters

##### body

`string`

##### options?

###### signal?

`AbortSignal`

#### Returns

`Promise`\<`string`\>

***

### searchSeries()

> **searchSeries**(`q`, `limit?`, `options?`): `Promise`\<`string`[]\>

Defined in: [modules/bitview-client/index.js:8093](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L8093)

Search series

Search series by name or descriptive terms. Results prioritize whole query words in names, then descriptions, then fuzzy names, then fuzzy descriptions. Word order does not matter. Descriptions provide cohort terminology and formulas. The decoded q parameter is limited to 1024 UTF-8 bytes.

Endpoint: `GET /api/series/search`

#### Parameters

##### q

`string`

Words to find in series names and descriptions (`realized price`, `mvrv`), in any order

##### limit?

`number`

Maximum number of results, 1 to 1,000 (default: 100)

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

> **seriesEndpoint**\<`I`\>(`series`, `index`): `I` *extends* [`DateIndex`](../type-aliases/DateIndex.md) ? [`DateSeriesEndpoint`](../interfaces/DateSeriesEndpoint.md)\<`unknown`\> : [`SeriesEndpoint`](../interfaces/SeriesEndpoint.md)\<`unknown`\>

Defined in: [modules/bitview-client/index.js:7939](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L7939)

A series by a name known only at runtime; `series` has the typed paths.

#### Type Parameters

##### I

`I` *extends* [`Index`](../type-aliases/Index.md)

#### Parameters

##### series

`string`

##### index

`I`

The index name; date indexes also slice by Date

#### Returns

`I` *extends* [`DateIndex`](../type-aliases/DateIndex.md) ? [`DateSeriesEndpoint`](../interfaces/DateSeriesEndpoint.md)\<`unknown`\> : [`SeriesEndpoint`](../interfaces/SeriesEndpoint.md)\<`unknown`\>

***

### validateAddress()

> **validateAddress**(`address`, `options?`): `Promise`\<[`AddressValidation`](../interfaces/AddressValidation.md)\>

Defined in: [modules/bitview-client/index.js:8527](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L8527)

Validate address

Validate a Bitcoin address and get information about its type and scriptPubKey. Returns `isvalid: false` with an error message for invalid addresses; a string that isn't 2 to 130 letters and digits answers 400.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-address-validate)*

Endpoint: `GET /api/v1/validate-address/{address}`

#### Parameters

##### address

`string`

Bitcoin address to validate: 2 to 130 letters and digits (a P2PK public key in hex is 130);
anything else answers 400, as mempool.space does

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

`Promise`\<[`AddressValidation`](../interfaces/AddressValidation.md)\>

***

### addressPayloadHashPrefix()

> `static` **addressPayloadHashPrefix**(`payload`, `nibbles`): `string`

Defined in: [modules/bitview-client/index.js:7915](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L7915)

Compute the RapidHash v3 hash-prefix for raw address payload bytes.

#### Parameters

##### payload

`number`[] \| `ArrayBuffer` \| `Uint8Array`\<`ArrayBufferLike`\> \| `ArrayBufferView`\<`ArrayBufferLike`\>

##### nibbles

`number`

#### Returns

`string`
