[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / MempoolBlock

# Interface: MempoolBlock

Defined in: [modules/bitview-client/index.js:749](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L749)

## Properties

### blockSize

> **blockSize**: `number`

Defined in: [modules/bitview-client/index.js:750](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L750)

Sum of the transactions' serialized sizes in bytes (witness + non-witness), without header
  or coinbase.

***

### blockVSize

> **blockVSize**: `number`

Defined in: [modules/bitview-client/index.js:752](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L752)

The transactions' weight over 4, unrounded (mempool.space's): the block's virtual size

***

### feeRange

> **feeRange**: `number`[]

Defined in: [modules/bitview-client/index.js:756](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L756)

Vsize-weighted percentiles of the transactions' CPFP chunk fee rates, in sat/vB:
  [0, 10, 25, 50, 75, 90, 100] for the first block, [5, 10, 25, 50, 75, 90, 95] for the
  later projected ones

***

### medianFee

> **medianFee**: `number`

Defined in: [modules/bitview-client/index.js:755](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L755)

Median fee rate in sat/vB

***

### nTx

> **nTx**: `number`

Defined in: [modules/bitview-client/index.js:753](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L753)

Number of transactions in the projected block

***

### totalFees

> **totalFees**: `number`

Defined in: [modules/bitview-client/index.js:754](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L754)

Total fees in satoshis
