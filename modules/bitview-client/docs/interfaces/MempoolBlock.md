[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / MempoolBlock

# Interface: MempoolBlock

Defined in: [modules/bitview-client/index.js:765](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L765)

## Properties

### blockSize

> **blockSize**: `number`

Defined in: [modules/bitview-client/index.js:766](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L766)

Sum of the transactions' serialized sizes in bytes (witness + non-witness), without header
or coinbase.

***

### blockVSize

> **blockVSize**: `number`

Defined in: [modules/bitview-client/index.js:768](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L768)

Sum of the transactions' virtual sizes in vbytes

***

### feeRange

> **feeRange**: `number`[]

Defined in: [modules/bitview-client/index.js:772](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L772)

Vsize-weighted percentiles of the transactions' CPFP chunk fee rates, in sat/vB:
[0, 10, 25, 50, 75, 90, 100] for the first block, [5, 10, 25, 50, 75, 90, 95] for the
later projected ones

***

### medianFee

> **medianFee**: `number`

Defined in: [modules/bitview-client/index.js:771](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L771)

Median fee rate in sat/vB

***

### nTx

> **nTx**: `number`

Defined in: [modules/bitview-client/index.js:769](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L769)

Number of transactions in the projected block

***

### totalFees

> **totalFees**: `number`

Defined in: [modules/bitview-client/index.js:770](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L770)

Total fees in satoshis
