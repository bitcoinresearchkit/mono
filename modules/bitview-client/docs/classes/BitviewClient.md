[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / BitviewClient

# Class: BitviewClient

Defined in: [modules/bitview-client/index.js:7189](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L7189)

A method per API endpoint, and the typed series tree.

## Extends

- `BitviewClientBase`

## Constructors

### Constructor

> **new BitviewClient**(`options`): `BitviewClient`

Defined in: [modules/bitview-client/index.js:2149](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L2149)

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

Defined in: [modules/bitview-client/index.js:2160](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L2160)

#### Inherited from

`BitviewClientBase._browserCache`

***

### \_browserCachePromise

> **\_browserCachePromise**: `Promise`\<`Cache` \| `null`\>

Defined in: [modules/bitview-client/index.js:2158](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L2158)

#### Inherited from

`BitviewClientBase._browserCachePromise`

***

### \_memCache

> **\_memCache**: `Map`\<`string`, [`_MemEntry`](../type-aliases/MemEntry.md)\<`unknown`\>\>

Defined in: [modules/bitview-client/index.js:2167](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L2167)

#### Inherited from

`BitviewClientBase._memCache`

## Accessors

### series

#### Get Signature

> **get** **series**(): [`SeriesTree`](../type-aliases/SeriesTree.md)

Defined in: [modules/bitview-client/index.js:7977](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L7977)

##### Returns

[`SeriesTree`](../type-aliases/SeriesTree.md)

## Methods

### \_fetchSeriesData()

> **\_fetchSeriesData**\<`T`\>(`path`, `arg?`, `options?`): `Promise`\<[`DateSeriesData`](../type-aliases/DateSeriesData.md)\<`T`\>\>

Defined in: [modules/bitview-client/index.js:2420](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L2420)

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

Defined in: [modules/bitview-client/index.js:2249](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L2249)

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

Defined in: [modules/bitview-client/index.js:2175](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L2175)

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

Defined in: [modules/bitview-client/index.js:2189](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L2189)

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

Defined in: [modules/bitview-client/index.js:2204](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L2204)

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

Defined in: [modules/bitview-client/index.js:7971](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L7971)

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

Defined in: [modules/bitview-client/index.js:2214](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L2214)

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

Defined in: [modules/bitview-client/index.js:8470](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L8470)

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

Defined in: [modules/bitview-client/index.js:8514](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L8514)

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

Defined in: [modules/bitview-client/index.js:8537](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L8537)

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

Defined in: [modules/bitview-client/index.js:8452](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L8452)

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

Defined in: [modules/bitview-client/index.js:8559](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L8559)

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

Defined in: [modules/bitview-client/index.js:7998](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L7998)

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

Defined in: [modules/bitview-client/index.js:8491](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L8491)

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

Defined in: [modules/bitview-client/index.js:8580](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L8580)

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

Defined in: [modules/bitview-client/index.js:9731](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L9731)

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

Defined in: [modules/bitview-client/index.js:8615](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L8615)

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

Defined in: [modules/bitview-client/index.js:8687](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L8687)

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

Defined in: [modules/bitview-client/index.js:9170](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L9170)

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

Defined in: [modules/bitview-client/index.js:9188](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L9188)

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

Defined in: [modules/bitview-client/index.js:9134](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L9134)

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

Defined in: [modules/bitview-client/index.js:8669](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L8669)

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

Defined in: [modules/bitview-client/index.js:8651](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L8651)

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

Defined in: [modules/bitview-client/index.js:8705](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L8705)

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

Defined in: [modules/bitview-client/index.js:9152](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L9152)

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

Defined in: [modules/bitview-client/index.js:8841](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L8841)

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

Defined in: [modules/bitview-client/index.js:8859](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L8859)

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

Defined in: [modules/bitview-client/index.js:9206](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L9206)

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

Defined in: [modules/bitview-client/index.js:8723](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L8723)

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

Defined in: [modules/bitview-client/index.js:8875](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L8875)

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

Defined in: [modules/bitview-client/index.js:8893](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L8893)

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

Defined in: [modules/bitview-client/index.js:8751](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L8751)

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

Defined in: [modules/bitview-client/index.js:8737](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L8737)

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

Defined in: [modules/bitview-client/index.js:8770](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L8770)

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

Defined in: [modules/bitview-client/index.js:8788](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L8788)

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

Defined in: [modules/bitview-client/index.js:8806](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L8806)

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

Defined in: [modules/bitview-client/index.js:8825](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L8825)

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

Defined in: [modules/bitview-client/index.js:8633](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L8633)

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

Defined in: [modules/bitview-client/index.js:2358](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L2358)

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

Defined in: [modules/bitview-client/index.js:9399](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L9399)

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

Defined in: [modules/bitview-client/index.js:8395](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L8395)

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

Defined in: [modules/bitview-client/index.js:9080](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L9080)

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

Defined in: [modules/bitview-client/index.js:9098](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L9098)

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

Defined in: [modules/bitview-client/index.js:8085](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L8085)

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

Defined in: [modules/bitview-client/index.js:9332](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L9332)

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

Defined in: [modules/bitview-client/index.js:9048](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L9048)

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

Defined in: [modules/bitview-client/index.js:9066](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L9066)

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

Defined in: [modules/bitview-client/index.js:8024](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L8024)

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

Defined in: [modules/bitview-client/index.js:8430](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L8430)

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

Defined in: [modules/bitview-client/index.js:2338](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L2338)

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

Defined in: [modules/bitview-client/index.js:9657](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L9657)

Live BTC/USD price

Current BTC/USD price in dollars, derived from on-chain round-dollar output patterns in the oracle's recent-block window, with the mempool's outputs blended in when available, to the cent: `price` of `GET /api/oracle/live`. Confirmed per-height history is available at `GET /api/series/price/height`. Not found before the oracle's first prices.

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

> **getLogs**(`after?`, `options?`): `Promise`\<[`Logs`](../interfaces/Logs.md)\>

Defined in: [modules/bitview-client/index.js:8068](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L8068)

Logs

The server's last log lines, at most 1,000, oldest first: requests (status, path and duration; never who made them) and everything else it logs. To read on, send the answer's `next` back as `after`: the lines since, each once. A jump in `seq` shows lines that left memory before the next request; another `started_unix_ms` shows the server restarted (and its answer has every kept line). Answers are cached for a second.

Endpoint: `GET /api/server/logs`

#### Parameters

##### after?

`string`

The last answer's `next`: only the lines since. From a process that's since restarted, or none, every kept line.

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

Defined in: [modules/bitview-client/index.js:9268](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L9268)

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

Defined in: [modules/bitview-client/index.js:9348](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L9348)

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

Defined in: [modules/bitview-client/index.js:9365](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L9365)

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

Defined in: [modules/bitview-client/index.js:9222](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L9222)

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

Defined in: [modules/bitview-client/index.js:9300](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L9300)

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

Defined in: [modules/bitview-client/index.js:9284](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L9284)

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

Defined in: [modules/bitview-client/index.js:9717](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L9717)

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

### getOracleBlock()

> **getOracleBlock**(`height`, `options?`): `Promise`\<[`OracleBlock`](../interfaces/OracleBlock.md)\>

Defined in: [modules/bitview-client/index.js:9687](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L9687)

The oracle at a block

How the oracle read a confirmed block, after height 508,000: what it counted (the smoothed histogram of the payments of the block and the 11 before it, by amount on its log scale; the block's outputs as its filter judged them, each skipped one under the first rule that skips it), where it looked (every candidate bin in its window, 12 below the block before's answer to 11 above, with its score) and what it found (the bin it picked and its price). Bins: 200 a decade, a bin `round(log10(sats) * 200)`; as where $100 lands, a bin is the price `10^(10 - bin / 200)` dollars. A settled block's never changes.

Endpoint: `GET /api/oracle/blocks/{height}`

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

`Promise`\<[`OracleBlock`](../interfaces/OracleBlock.md)\>

***

### getOracleLive()

> **getOracleLive**(`options?`): `Promise`\<[`OracleBlock`](../interfaces/OracleBlock.md)\>

Defined in: [modules/bitview-client/index.js:9671](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L9671)

The oracle now

The oracle's reading now, as `GET /api/oracle/blocks/{height}` reads a block: with the mempool's outputs blended in as the next block (`mempool` true, `height` the next block's, `outputs` the mempool's), its search from the tip's answer, `price` the live price; without a mempool at the tip, the tip block's own. One snapshot: its parts never from two moments, `hash` the confirmed tip it was read at. Not found before the oracle's first prices, nor before height 508,000.

Endpoint: `GET /api/oracle/live`

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

`Promise`\<[`OracleBlock`](../interfaces/OracleBlock.md)\>

***

### getOraclePayments()

> **getOraclePayments**(`start`, `options?`): `Promise`\<[`OraclePayments`](../interfaces/OraclePayments.md)\>

Defined in: [modules/bitview-client/index.js:9703](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L9703)

The oracle's payments, 144 blocks at a time

For each of the 144 blocks from `start` (a multiple of 144, else a bad request; from 508,032, earlier ones out of range): its payments as the oracle filters them, each read as each of its 19 round-dollar amounts ($1 to $10,000) at its exact amount in sats, counted by the price each reading implies, in buckets a tenth of a bin wide (0.115% of price) over the block's search window: finer than the oracle's bins, the evidence behind its prices. The chunk at the tip holds its blocks so far, `hash` its last block's; a whole, settled chunk never changes.

Endpoint: `GET /api/oracle/payments/{start}`

#### Parameters

##### start

`number`

The first block's height.

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

`Promise`\<[`OraclePayments`](../interfaces/OraclePayments.md)\>

***

### getPool()

> **getPool**(`slug`, `options?`): `Promise`\<[`PoolDetail`](../interfaces/PoolDetail.md)\>

Defined in: [modules/bitview-client/index.js:8943](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L8943)

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

Defined in: [modules/bitview-client/index.js:9013](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L9013)

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

Defined in: [modules/bitview-client/index.js:9032](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L9032)

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

Defined in: [modules/bitview-client/index.js:8995](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L8995)

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

Defined in: [modules/bitview-client/index.js:8925](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L8925)

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

Defined in: [modules/bitview-client/index.js:8907](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L8907)

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

Defined in: [modules/bitview-client/index.js:8959](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L8959)

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

Defined in: [modules/bitview-client/index.js:8977](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L8977)

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

Defined in: [modules/bitview-client/index.js:9254](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L9254)

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

Defined in: [modules/bitview-client/index.js:8411](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L8411)

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

Defined in: [modules/bitview-client/index.js:9238](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L9238)

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

Defined in: [modules/bitview-client/index.js:9316](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L9316)

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

Defined in: [modules/bitview-client/index.js:9116](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L9116)

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

Defined in: [modules/bitview-client/index.js:8206](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L8206)

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

Defined in: [modules/bitview-client/index.js:8311](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L8311)

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

Defined in: [modules/bitview-client/index.js:8113](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L8113)

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

Defined in: [modules/bitview-client/index.js:8234](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L8234)

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

Defined in: [modules/bitview-client/index.js:8185](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L8185)

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

Defined in: [modules/bitview-client/index.js:8258](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L8258)

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

Defined in: [modules/bitview-client/index.js:8273](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L8273)

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

Defined in: [modules/bitview-client/index.js:8099](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L8099)

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

Defined in: [modules/bitview-client/index.js:8290](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L8290)

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

Defined in: [modules/bitview-client/index.js:8050](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L8050)

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

Defined in: [modules/bitview-client/index.js:2348](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L2348)

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

Defined in: [modules/bitview-client/index.js:9576](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L9576)

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

Defined in: [modules/bitview-client/index.js:9433](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L9433)

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

Defined in: [modules/bitview-client/index.js:9451](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L9451)

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

Defined in: [modules/bitview-client/index.js:9381](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L9381)

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

Defined in: [modules/bitview-client/index.js:9469](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L9469)

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

Defined in: [modules/bitview-client/index.js:9485](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L9485)

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

Defined in: [modules/bitview-client/index.js:9504](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L9504)

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

Defined in: [modules/bitview-client/index.js:9522](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L9522)

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

Defined in: [modules/bitview-client/index.js:9540](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L9540)

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

Defined in: [modules/bitview-client/index.js:9417](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L9417)

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

Defined in: [modules/bitview-client/index.js:9558](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L9558)

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

Defined in: [modules/bitview-client/index.js:8375](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L8375)

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

Defined in: [modules/bitview-client/index.js:8352](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L8352)

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

Defined in: [modules/bitview-client/index.js:9627](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L9627)

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

Defined in: [modules/bitview-client/index.js:9643](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L9643)

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

Defined in: [modules/bitview-client/index.js:9611](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L9611)

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

Defined in: [modules/bitview-client/index.js:8036](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L8036)

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

Defined in: [modules/bitview-client/index.js:7960](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L7960)

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

Defined in: [modules/bitview-client/index.js:8127](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L8127)

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

Defined in: [modules/bitview-client/index.js:8144](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L8144)

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

Defined in: [modules/bitview-client/index.js:8334](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L8334)

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

Defined in: [modules/bitview-client/index.js:2371](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L2371)

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

Defined in: [modules/bitview-client/index.js:2408](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L2408)

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

Defined in: [modules/bitview-client/index.js:2386](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L2386)

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

Defined in: [modules/bitview-client/index.js:2397](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L2397)

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

Defined in: [modules/bitview-client/index.js:9597](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L9597)

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

Defined in: [modules/bitview-client/index.js:8165](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L8165)

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

Defined in: [modules/bitview-client/index.js:8011](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L8011)

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

Defined in: [modules/bitview-client/index.js:8599](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L8599)

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

Defined in: [modules/bitview-client/index.js:7987](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L7987)

Compute the RapidHash v3 hash-prefix for raw address payload bytes.

#### Parameters

##### payload

`number`[] \| `ArrayBuffer` \| `Uint8Array`\<`ArrayBufferLike`\> \| `ArrayBufferView`\<`ArrayBufferLike`\>

##### nibbles

`number`

#### Returns

`string`
