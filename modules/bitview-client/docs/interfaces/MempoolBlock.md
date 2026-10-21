[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / MempoolBlock

# Interface: MempoolBlock

Defined in: [modules/bitview-client/index.js:698](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L698)

## Properties

### blockSize

> **blockSize**: `number`

Defined in: [modules/bitview-client/index.js:699](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L699)

Sum of the transactions' serialized sizes in bytes (witness + non-witness), without header
or coinbase.

***

### blockVSize

> **blockVSize**: `number`

Defined in: [modules/bitview-client/index.js:701](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L701)

Sum of the transactions' virtual sizes in vbytes

***

### feeRange

> **feeRange**: `number`[]

Defined in: [modules/bitview-client/index.js:705](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L705)

Vsize-weighted percentiles of the transactions' CPFP chunk fee rates, in sat/vB:
[0, 10, 25, 50, 75, 90, 100] for the first block, [5, 10, 25, 50, 75, 90, 95] for the
later projected ones

***

### medianFee

> **medianFee**: `number`

Defined in: [modules/bitview-client/index.js:704](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L704)

Median fee rate in sat/vB

***

### nTx

> **nTx**: `number`

Defined in: [modules/bitview-client/index.js:702](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L702)

Number of transactions in the projected block

***

### totalFees

> **totalFees**: `number`

Defined in: [modules/bitview-client/index.js:703](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L703)

Total fees in satoshis
