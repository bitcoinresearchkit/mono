[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / BlockExtras

# Interface: BlockExtras

Defined in: [modules/bitview-client/index.js:79](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L79)

## Properties

### avgFee

> **avgFee**: `number`

Defined in: [modules/bitview-client/index.js:90](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L90)

Mean fee per non-coinbase transaction in satoshis, rounded down (0 without any)

***

### avgFeeRate

> **avgFeeRate**: `number`

Defined in: [modules/bitview-client/index.js:91](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L91)

Average fee rate in sat/vB

***

### avgTxSize

> **avgTxSize**: `number`

Defined in: [modules/bitview-client/index.js:97](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L97)

Non-coinbase transaction bytes over the transaction count (coinbase included), rounded to
  2 decimals

***

### coinbaseAddress?

> `optional` **coinbaseAddress?**: `string` \| `null`

Defined in: [modules/bitview-client/index.js:93](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L93)

Primary coinbase output address

***

### coinbaseAddresses

> **coinbaseAddresses**: `string`[]

Defined in: [modules/bitview-client/index.js:94](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L94)

All coinbase output addresses

***

### coinbaseBip54?

> `optional` **coinbaseBip54?**: `boolean` \| `null`

Defined in: [modules/bitview-client/index.js:113](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L113)

Whether the coinbase follows BIP54 (its lock time is the height minus one and its input's
  sequence isn't final), from timestamp 1,771,507,776 on; null before

***

### coinbaseRaw

> **coinbaseRaw**: `string`

Defined in: [modules/bitview-client/index.js:92](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L92)

Raw coinbase transaction scriptsig as hex

***

### coinbaseSignature

> **coinbaseSignature**: `string`

Defined in: [modules/bitview-client/index.js:95](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L95)

Coinbase output script in ASM format

***

### coinbaseSignatureAscii

> **coinbaseSignatureAscii**: `string`

Defined in: [modules/bitview-client/index.js:96](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L96)

Coinbase scriptsig with each byte read as the Unicode code point of the same value

***

### expectedFees?

> `optional` **expectedFees?**: `number` \| `null`

Defined in: [modules/bitview-client/index.js:119](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L119)

Fees of the expected block template (always null: it needs the mempool as it was)

***

### expectedWeight?

> `optional` **expectedWeight?**: `number` \| `null`

Defined in: [modules/bitview-client/index.js:120](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L120)

Weight of the expected block template (always null: it needs the mempool as it was)

***

### feePercentiles

> **feePercentiles**: `number`[]

Defined in: [modules/bitview-client/index.js:103](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L103)

Fee amount percentiles in satoshis: [min, 10%, 25%, 50%, 75%, 90%, max]

***

### feeRange

> **feeRange**: `number`[]

Defined in: [modules/bitview-client/index.js:84](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L84)

Effective (CPFP) fee rate range in sat/vB: [min, 10%, 25%, 50%, 75%, 90%, max], the
  percentiles of block space (vsize-weighted, as `medianFee`; mempool.space ranks
  transactions by count), min and max the lower/higher of the 1st/99th percentile and the
  extreme rate in the block's last/first 2% of vsize

***

### firstSeen?

> `optional` **firstSeen?**: `number` \| `null`

Defined in: [modules/bitview-client/index.js:115](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L115)

Timestamp when the block was first seen (always null, not yet supported)

***

### header

> **header**: `string`

Defined in: [modules/bitview-client/index.js:107](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L107)

Raw 80-byte block header as hex

***

### matchRate?

> `optional` **matchRate?**: `number` \| `null`

Defined in: [modules/bitview-client/index.js:117](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L117)

Share of the expected block template the block matched, from mempool.space's own audits
  (always null: it needs the mempool as it was)

***

### medianFee

> **medianFee**: `number`

Defined in: [modules/bitview-client/index.js:81](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L81)

Median effective (CPFP) fee rate in sat/vB, by vsize: the rate half the block's space
  paid at most (Bitcoin Core's `getblockstats` rule; mempool.space averages the middle
  0.25% of a 4M WU block instead)

***

### medianFeeAmt

> **medianFeeAmt**: `number`

Defined in: [modules/bitview-client/index.js:102](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L102)

Median fee amount in satoshis

***

### orphans

> **orphans**: [`BlockOrphan`](BlockOrphan.md)[]

Defined in: [modules/bitview-client/index.js:116](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L116)

Stale blocks at this height (always empty, not yet supported)

***

### pool

> **pool**: [`BlockPool`](BlockPool.md)

Defined in: [modules/bitview-client/index.js:89](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L89)

***

### price

> **price**: `number`

Defined in: [modules/bitview-client/index.js:121](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L121)

USD price at block height

***

### reward

> **reward**: `number`

Defined in: [modules/bitview-client/index.js:88](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L88)

Total block reward (subsidy + fees) in satoshis

***

### segwitTotalSize

> **segwitTotalSize**: `number`

Defined in: [modules/bitview-client/index.js:105](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L105)

Total size of segwit transactions in bytes

***

### segwitTotalTxs

> **segwitTotalTxs**: `number`

Defined in: [modules/bitview-client/index.js:104](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L104)

***

### segwitTotalWeight

> **segwitTotalWeight**: `number`

Defined in: [modules/bitview-client/index.js:106](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L106)

***

### totalFees

> **totalFees**: `number`

Defined in: [modules/bitview-client/index.js:80](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L80)

Total fees in satoshis

***

### totalInputAmt

> **totalInputAmt**: `number`

Defined in: [modules/bitview-client/index.js:111](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L111)

Total input amount in satoshis

***

### totalInputs

> **totalInputs**: `number`

Defined in: [modules/bitview-client/index.js:99](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L99)

Total number of inputs (excluding coinbase)

***

### totalOutputAmt

> **totalOutputAmt**: `number`

Defined in: [modules/bitview-client/index.js:101](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L101)

Total output amount in satoshis, coinbase excluded

***

### totalOutputs

> **totalOutputs**: `number`

Defined in: [modules/bitview-client/index.js:100](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L100)

***

### utxoSetChange

> **utxoSetChange**: `number`

Defined in: [modules/bitview-client/index.js:108](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L108)

UTXO set change as mempool.space and bitcoin-cli report it: total outputs minus total
  inputs, unspendable outputs (OP_RETURN) included, unlike `utxoSetSize`

***

### utxoSetSize

> **utxoSetSize**: `number`

Defined in: [modules/bitview-client/index.js:110](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L110)

Total spendable UTXO set size at this height (excludes OP_RETURN and other unspendable outputs)

***

### virtualSize

> **virtualSize**: `number`

Defined in: [modules/bitview-client/index.js:112](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L112)

Virtual size in vbytes: weight / 4, unrounded
