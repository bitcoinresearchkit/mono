[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / BlockExtras

# Interface: BlockExtras

Defined in: [modules/bitview-client/index.js:112](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L112)

## Properties

### avgFee

> **avgFee**: `number`

Defined in: [modules/bitview-client/index.js:118](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L118)

Mean fee per non-coinbase transaction in satoshis, rounded down (0 without any)

***

### avgFeeRate

> **avgFeeRate**: `number`

Defined in: [modules/bitview-client/index.js:119](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L119)

Average fee rate in sat/vB

***

### avgTxSize

> **avgTxSize**: `number`

Defined in: [modules/bitview-client/index.js:125](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L125)

Non-coinbase transaction bytes over the transaction count (coinbase included), rounded to
2 decimals

***

### coinbaseAddress?

> `optional` **coinbaseAddress?**: `string` \| `null`

Defined in: [modules/bitview-client/index.js:121](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L121)

Primary coinbase output address

***

### coinbaseAddresses

> **coinbaseAddresses**: `string`[]

Defined in: [modules/bitview-client/index.js:122](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L122)

All coinbase output addresses

***

### coinbaseBip54?

> `optional` **coinbaseBip54?**: `boolean` \| `null`

Defined in: [modules/bitview-client/index.js:141](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L141)

Whether the coinbase follows BIP54 (its lock time is the height minus one and its input's
sequence isn't final), from timestamp 1,771,507,776 on; null before

***

### coinbaseRaw

> **coinbaseRaw**: `string`

Defined in: [modules/bitview-client/index.js:120](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L120)

Raw coinbase transaction scriptsig as hex

***

### coinbaseSignature

> **coinbaseSignature**: `string`

Defined in: [modules/bitview-client/index.js:123](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L123)

Coinbase output script in ASM format

***

### coinbaseSignatureAscii

> **coinbaseSignatureAscii**: `string`

Defined in: [modules/bitview-client/index.js:124](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L124)

Coinbase scriptsig with each byte read as the Unicode code point of the same value

***

### expectedFees?

> `optional` **expectedFees?**: `number` \| `null`

Defined in: [modules/bitview-client/index.js:147](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L147)

Fees of the expected block template (always null: it needs the mempool as it was)

***

### expectedWeight?

> `optional` **expectedWeight?**: `number` \| `null`

Defined in: [modules/bitview-client/index.js:148](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L148)

Weight of the expected block template (always null: it needs the mempool as it was)

***

### feePercentiles

> **feePercentiles**: `number`[]

Defined in: [modules/bitview-client/index.js:131](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L131)

Fee amount percentiles in satoshis: [min, 10%, 25%, 50%, 75%, 90%, max]

***

### feeRange

> **feeRange**: `number`[]

Defined in: [modules/bitview-client/index.js:115](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L115)

Fee rate range: [min, 10%, 25%, 50%, 75%, 90%, max]

***

### firstSeen?

> `optional` **firstSeen?**: `number` \| `null`

Defined in: [modules/bitview-client/index.js:143](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L143)

Timestamp when the block was first seen (always null, not yet supported)

***

### header

> **header**: `string`

Defined in: [modules/bitview-client/index.js:135](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L135)

Raw 80-byte block header as hex

***

### matchRate?

> `optional` **matchRate?**: `number` \| `null`

Defined in: [modules/bitview-client/index.js:145](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L145)

Share of the expected block template the block matched, from mempool.space's own audits
(always null: it needs the mempool as it was)

***

### medianFee

> **medianFee**: `number`

Defined in: [modules/bitview-client/index.js:114](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L114)

Median fee rate in sat/vB

***

### medianFeeAmt

> **medianFeeAmt**: `number`

Defined in: [modules/bitview-client/index.js:130](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L130)

Median fee amount in satoshis

***

### orphans

> **orphans**: [`BlockOrphan`](BlockOrphan.md)[]

Defined in: [modules/bitview-client/index.js:144](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L144)

Stale blocks at this height (always empty, not yet supported)

***

### pool

> **pool**: [`BlockPool`](BlockPool.md)

Defined in: [modules/bitview-client/index.js:117](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L117)

***

### price

> **price**: `number`

Defined in: [modules/bitview-client/index.js:149](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L149)

USD price at block height

***

### reward

> **reward**: `number`

Defined in: [modules/bitview-client/index.js:116](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L116)

Total block reward (subsidy + fees) in satoshis

***

### segwitTotalSize

> **segwitTotalSize**: `number`

Defined in: [modules/bitview-client/index.js:133](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L133)

Total size of segwit transactions in bytes

***

### segwitTotalTxs

> **segwitTotalTxs**: `number`

Defined in: [modules/bitview-client/index.js:132](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L132)

***

### segwitTotalWeight

> **segwitTotalWeight**: `number`

Defined in: [modules/bitview-client/index.js:134](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L134)

***

### totalFees

> **totalFees**: `number`

Defined in: [modules/bitview-client/index.js:113](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L113)

Total fees in satoshis

***

### totalInputAmt

> **totalInputAmt**: `number`

Defined in: [modules/bitview-client/index.js:139](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L139)

Total input amount in satoshis

***

### totalInputs

> **totalInputs**: `number`

Defined in: [modules/bitview-client/index.js:127](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L127)

Total number of inputs (excluding coinbase)

***

### totalOutputAmt

> **totalOutputAmt**: `number`

Defined in: [modules/bitview-client/index.js:129](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L129)

Total output amount in satoshis, coinbase excluded

***

### totalOutputs

> **totalOutputs**: `number`

Defined in: [modules/bitview-client/index.js:128](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L128)

***

### utxoSetChange

> **utxoSetChange**: `number`

Defined in: [modules/bitview-client/index.js:136](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L136)

UTXO set change as mempool.space and bitcoin-cli report it: total outputs minus total
inputs, unspendable outputs (OP_RETURN) included, unlike `utxoSetSize`

***

### utxoSetSize

> **utxoSetSize**: `number`

Defined in: [modules/bitview-client/index.js:138](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L138)

Total spendable UTXO set size at this height (excludes OP_RETURN and other unspendable outputs)

***

### virtualSize

> **virtualSize**: `number`

Defined in: [modules/bitview-client/index.js:140](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L140)

Virtual size in vbytes: weight / 4, unrounded
