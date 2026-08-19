[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / BlockExtras

# Interface: BlockExtras

Defined in: [Developer/brk/modules/bitview-client/index.js:108](https://github.com/bitcoinresearchkit/brk/blob/70a3e90b82b397fa1d5ca46c4ed613066ed3d195/modules/bitview-client/index.js#L108)

## Properties

### avgFee

> **avgFee**: `number`

Defined in: [Developer/brk/modules/bitview-client/index.js:114](https://github.com/bitcoinresearchkit/brk/blob/70a3e90b82b397fa1d5ca46c4ed613066ed3d195/modules/bitview-client/index.js#L114)

Average fee per transaction in satoshis

***

### avgFeeRate

> **avgFeeRate**: `number`

Defined in: [Developer/brk/modules/bitview-client/index.js:115](https://github.com/bitcoinresearchkit/brk/blob/70a3e90b82b397fa1d5ca46c4ed613066ed3d195/modules/bitview-client/index.js#L115)

Average fee rate in sat/vB

***

### avgTxSize

> **avgTxSize**: `number`

Defined in: [Developer/brk/modules/bitview-client/index.js:121](https://github.com/bitcoinresearchkit/brk/blob/70a3e90b82b397fa1d5ca46c4ed613066ed3d195/modules/bitview-client/index.js#L121)

Average transaction size in bytes

***

### coinbaseAddress?

> `optional` **coinbaseAddress?**: `string` \| `null`

Defined in: [Developer/brk/modules/bitview-client/index.js:117](https://github.com/bitcoinresearchkit/brk/blob/70a3e90b82b397fa1d5ca46c4ed613066ed3d195/modules/bitview-client/index.js#L117)

Primary coinbase output address

***

### coinbaseAddresses

> **coinbaseAddresses**: `string`[]

Defined in: [Developer/brk/modules/bitview-client/index.js:118](https://github.com/bitcoinresearchkit/brk/blob/70a3e90b82b397fa1d5ca46c4ed613066ed3d195/modules/bitview-client/index.js#L118)

All coinbase output addresses

***

### coinbaseRaw

> **coinbaseRaw**: `string`

Defined in: [Developer/brk/modules/bitview-client/index.js:116](https://github.com/bitcoinresearchkit/brk/blob/70a3e90b82b397fa1d5ca46c4ed613066ed3d195/modules/bitview-client/index.js#L116)

Raw coinbase transaction scriptsig as hex

***

### coinbaseSignature

> **coinbaseSignature**: `string`

Defined in: [Developer/brk/modules/bitview-client/index.js:119](https://github.com/bitcoinresearchkit/brk/blob/70a3e90b82b397fa1d5ca46c4ed613066ed3d195/modules/bitview-client/index.js#L119)

Coinbase output script in ASM format

***

### coinbaseSignatureAscii

> **coinbaseSignatureAscii**: `string`

Defined in: [Developer/brk/modules/bitview-client/index.js:120](https://github.com/bitcoinresearchkit/brk/blob/70a3e90b82b397fa1d5ca46c4ed613066ed3d195/modules/bitview-client/index.js#L120)

Coinbase scriptsig decoded as ASCII

***

### feePercentiles

> **feePercentiles**: `number`[]

Defined in: [Developer/brk/modules/bitview-client/index.js:126](https://github.com/bitcoinresearchkit/brk/blob/70a3e90b82b397fa1d5ca46c4ed613066ed3d195/modules/bitview-client/index.js#L126)

Fee amount percentiles in satoshis: [min, 10%, 25%, 50%, 75%, 90%, max]

***

### feeRange

> **feeRange**: `number`[]

Defined in: [Developer/brk/modules/bitview-client/index.js:111](https://github.com/bitcoinresearchkit/brk/blob/70a3e90b82b397fa1d5ca46c4ed613066ed3d195/modules/bitview-client/index.js#L111)

Fee rate range: [min, 10%, 25%, 50%, 75%, 90%, max]

***

### firstSeen?

> `optional` **firstSeen?**: `number` \| `null`

Defined in: [Developer/brk/modules/bitview-client/index.js:137](https://github.com/bitcoinresearchkit/brk/blob/70a3e90b82b397fa1d5ca46c4ed613066ed3d195/modules/bitview-client/index.js#L137)

Timestamp when the block was first seen (always null, not yet supported)

***

### header

> **header**: `string`

Defined in: [Developer/brk/modules/bitview-client/index.js:130](https://github.com/bitcoinresearchkit/brk/blob/70a3e90b82b397fa1d5ca46c4ed613066ed3d195/modules/bitview-client/index.js#L130)

Raw 80-byte block header as hex

***

### medianFee

> **medianFee**: `number`

Defined in: [Developer/brk/modules/bitview-client/index.js:110](https://github.com/bitcoinresearchkit/brk/blob/70a3e90b82b397fa1d5ca46c4ed613066ed3d195/modules/bitview-client/index.js#L110)

Median fee rate in sat/vB

***

### medianFeeAmt

> **medianFeeAmt**: `number`

Defined in: [Developer/brk/modules/bitview-client/index.js:125](https://github.com/bitcoinresearchkit/brk/blob/70a3e90b82b397fa1d5ca46c4ed613066ed3d195/modules/bitview-client/index.js#L125)

Median fee amount in satoshis

***

### orphans

> **orphans**: `string`[]

Defined in: [Developer/brk/modules/bitview-client/index.js:138](https://github.com/bitcoinresearchkit/brk/blob/70a3e90b82b397fa1d5ca46c4ed613066ed3d195/modules/bitview-client/index.js#L138)

Orphaned blocks (always empty)

***

### pool

> **pool**: [`BlockPool`](BlockPool.md)

Defined in: [Developer/brk/modules/bitview-client/index.js:113](https://github.com/bitcoinresearchkit/brk/blob/70a3e90b82b397fa1d5ca46c4ed613066ed3d195/modules/bitview-client/index.js#L113)

Mining pool that mined this block

***

### price

> **price**: `number`

Defined in: [Developer/brk/modules/bitview-client/index.js:139](https://github.com/bitcoinresearchkit/brk/blob/70a3e90b82b397fa1d5ca46c4ed613066ed3d195/modules/bitview-client/index.js#L139)

USD price at block height

***

### reward

> **reward**: `number`

Defined in: [Developer/brk/modules/bitview-client/index.js:112](https://github.com/bitcoinresearchkit/brk/blob/70a3e90b82b397fa1d5ca46c4ed613066ed3d195/modules/bitview-client/index.js#L112)

Total block reward (subsidy + fees) in satoshis

***

### segwitTotalSize

> **segwitTotalSize**: `number`

Defined in: [Developer/brk/modules/bitview-client/index.js:128](https://github.com/bitcoinresearchkit/brk/blob/70a3e90b82b397fa1d5ca46c4ed613066ed3d195/modules/bitview-client/index.js#L128)

Total size of segwit transactions in bytes

***

### segwitTotalTxs

> **segwitTotalTxs**: `number`

Defined in: [Developer/brk/modules/bitview-client/index.js:127](https://github.com/bitcoinresearchkit/brk/blob/70a3e90b82b397fa1d5ca46c4ed613066ed3d195/modules/bitview-client/index.js#L127)

Number of segwit transactions

***

### segwitTotalWeight

> **segwitTotalWeight**: `number`

Defined in: [Developer/brk/modules/bitview-client/index.js:129](https://github.com/bitcoinresearchkit/brk/blob/70a3e90b82b397fa1d5ca46c4ed613066ed3d195/modules/bitview-client/index.js#L129)

Total weight of segwit transactions

***

### totalFees

> **totalFees**: `number`

Defined in: [Developer/brk/modules/bitview-client/index.js:109](https://github.com/bitcoinresearchkit/brk/blob/70a3e90b82b397fa1d5ca46c4ed613066ed3d195/modules/bitview-client/index.js#L109)

Total fees in satoshis

***

### totalInputAmt

> **totalInputAmt**: `number`

Defined in: [Developer/brk/modules/bitview-client/index.js:135](https://github.com/bitcoinresearchkit/brk/blob/70a3e90b82b397fa1d5ca46c4ed613066ed3d195/modules/bitview-client/index.js#L135)

Total input amount in satoshis

***

### totalInputs

> **totalInputs**: `number`

Defined in: [Developer/brk/modules/bitview-client/index.js:122](https://github.com/bitcoinresearchkit/brk/blob/70a3e90b82b397fa1d5ca46c4ed613066ed3d195/modules/bitview-client/index.js#L122)

Total number of inputs (excluding coinbase)

***

### totalOutputAmt

> **totalOutputAmt**: `number`

Defined in: [Developer/brk/modules/bitview-client/index.js:124](https://github.com/bitcoinresearchkit/brk/blob/70a3e90b82b397fa1d5ca46c4ed613066ed3d195/modules/bitview-client/index.js#L124)

Total output amount in satoshis

***

### totalOutputs

> **totalOutputs**: `number`

Defined in: [Developer/brk/modules/bitview-client/index.js:123](https://github.com/bitcoinresearchkit/brk/blob/70a3e90b82b397fa1d5ca46c4ed613066ed3d195/modules/bitview-client/index.js#L123)

Total number of outputs

***

### utxoSetChange

> **utxoSetChange**: `number`

Defined in: [Developer/brk/modules/bitview-client/index.js:131](https://github.com/bitcoinresearchkit/brk/blob/70a3e90b82b397fa1d5ca46c4ed613066ed3d195/modules/bitview-client/index.js#L131)

UTXO set change (total outputs - total inputs, includes unspendable like OP_RETURN).
Note: intentionally differs from utxo_set_size diff which excludes unspendable outputs.
Matches mempool.space/bitcoin-cli behavior.

***

### utxoSetSize

> **utxoSetSize**: `number`

Defined in: [Developer/brk/modules/bitview-client/index.js:134](https://github.com/bitcoinresearchkit/brk/blob/70a3e90b82b397fa1d5ca46c4ed613066ed3d195/modules/bitview-client/index.js#L134)

Total spendable UTXO set size at this height (excludes OP_RETURN and other unspendable outputs)

***

### virtualSize

> **virtualSize**: `number`

Defined in: [Developer/brk/modules/bitview-client/index.js:136](https://github.com/bitcoinresearchkit/brk/blob/70a3e90b82b397fa1d5ca46c4ed613066ed3d195/modules/bitview-client/index.js#L136)

Virtual size in vbytes
