[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / MempoolBlock

# Interface: MempoolBlock

Defined in: [modules/bitview-client/index.js:729](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L729)

## Properties

### blockSize

> **blockSize**: `number`

Defined in: [modules/bitview-client/index.js:730](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L730)

Sum of the transactions' serialized sizes in bytes (witness + non-witness), without header
or coinbase.

***

### blockVSize

> **blockVSize**: `number`

Defined in: [modules/bitview-client/index.js:732](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L732)

Sum of the transactions' virtual sizes in vbytes

***

### feeRange

> **feeRange**: `number`[]

Defined in: [modules/bitview-client/index.js:736](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L736)

Vsize-weighted percentiles of the transactions' CPFP chunk fee rates, in sat/vB:
[0, 10, 25, 50, 75, 90, 100] for the first block, [5, 10, 25, 50, 75, 90, 95] for the
later projected ones

***

### medianFee

> **medianFee**: `number`

Defined in: [modules/bitview-client/index.js:735](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L735)

Median fee rate in sat/vB

***

### nTx

> **nTx**: `number`

Defined in: [modules/bitview-client/index.js:733](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L733)

Number of transactions in the projected block

***

### totalFees

> **totalFees**: `number`

Defined in: [modules/bitview-client/index.js:734](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L734)

Total fees in satoshis
