[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / MempoolBlock

# Interface: MempoolBlock

Defined in: [modules/bitview-client/index.js:734](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L734)

## Properties

### blockSize

> **blockSize**: `number`

Defined in: [modules/bitview-client/index.js:735](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L735)

Sum of the transactions' serialized sizes in bytes (witness + non-witness), without header
or coinbase.

***

### blockVSize

> **blockVSize**: `number`

Defined in: [modules/bitview-client/index.js:737](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L737)

Sum of the transactions' virtual sizes in vbytes

***

### feeRange

> **feeRange**: `number`[]

Defined in: [modules/bitview-client/index.js:741](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L741)

Vsize-weighted percentiles of the transactions' CPFP chunk fee rates, in sat/vB:
[0, 10, 25, 50, 75, 90, 100] for the first block, [5, 10, 25, 50, 75, 90, 95] for the
later projected ones

***

### medianFee

> **medianFee**: `number`

Defined in: [modules/bitview-client/index.js:740](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L740)

Median fee rate in sat/vB

***

### nTx

> **nTx**: `number`

Defined in: [modules/bitview-client/index.js:738](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L738)

Number of transactions in the projected block

***

### totalFees

> **totalFees**: `number`

Defined in: [modules/bitview-client/index.js:739](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L739)

Total fees in satoshis
