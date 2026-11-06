[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / MempoolBlock

# Interface: MempoolBlock

Defined in: [modules/bitview-client/index.js:766](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L766)

## Properties

### blockSize

> **blockSize**: `number`

Defined in: [modules/bitview-client/index.js:767](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L767)

Sum of the transactions' serialized sizes in bytes (witness + non-witness), without header
  or coinbase.

***

### blockVSize

> **blockVSize**: `number`

Defined in: [modules/bitview-client/index.js:769](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L769)

The transactions' weight over 4, unrounded (mempool.space's): the block's virtual size

***

### feeRange

> **feeRange**: `number`[]

Defined in: [modules/bitview-client/index.js:773](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L773)

Vsize-weighted percentiles of the transactions' CPFP chunk fee rates, in sat/vB:
  [0, 10, 25, 50, 75, 90, 100] for the first block, [5, 10, 25, 50, 75, 90, 95] for the
  later projected ones

***

### medianFee

> **medianFee**: `number`

Defined in: [modules/bitview-client/index.js:772](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L772)

Median fee rate in sat/vB

***

### nTx

> **nTx**: `number`

Defined in: [modules/bitview-client/index.js:770](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L770)

Number of transactions in the projected block

***

### totalFees

> **totalFees**: `number`

Defined in: [modules/bitview-client/index.js:771](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L771)

Total fees in satoshis
