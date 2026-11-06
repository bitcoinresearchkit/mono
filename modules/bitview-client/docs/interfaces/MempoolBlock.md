[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / MempoolBlock

# Interface: MempoolBlock

Defined in: [modules/bitview-client/index.js:685](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L685)

## Properties

### blockSize

> **blockSize**: `number`

Defined in: [modules/bitview-client/index.js:686](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L686)

Sum of the transactions' serialized sizes in bytes (witness + non-witness), without header
  or coinbase.

***

### blockVSize

> **blockVSize**: `number`

Defined in: [modules/bitview-client/index.js:688](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L688)

The transactions' weight over 4, unrounded (mempool.space's): the block's virtual size

***

### feeRange

> **feeRange**: `number`[]

Defined in: [modules/bitview-client/index.js:692](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L692)

Vsize-weighted percentiles of the transactions' CPFP chunk fee rates, in sat/vB:
  [0, 10, 25, 50, 75, 90, 100] for the first block, [5, 10, 25, 50, 75, 90, 95] for the
  later projected ones

***

### medianFee

> **medianFee**: `number`

Defined in: [modules/bitview-client/index.js:691](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L691)

Median fee rate in sat/vB

***

### nTx

> **nTx**: `number`

Defined in: [modules/bitview-client/index.js:689](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L689)

Number of transactions in the projected block

***

### totalFees

> **totalFees**: `number`

Defined in: [modules/bitview-client/index.js:690](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L690)

Total fees in satoshis
