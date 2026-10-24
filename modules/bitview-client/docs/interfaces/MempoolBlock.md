[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / MempoolBlock

# Interface: MempoolBlock

Defined in: [modules/bitview-client/index.js:780](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L780)

## Properties

### blockSize

> **blockSize**: `number`

Defined in: [modules/bitview-client/index.js:781](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L781)

Sum of the transactions' serialized sizes in bytes (witness + non-witness), without header
or coinbase.

***

### blockVSize

> **blockVSize**: `number`

Defined in: [modules/bitview-client/index.js:783](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L783)

Sum of the transactions' virtual sizes in vbytes

***

### feeRange

> **feeRange**: `number`[]

Defined in: [modules/bitview-client/index.js:787](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L787)

Vsize-weighted percentiles of the transactions' CPFP chunk fee rates, in sat/vB:
[0, 10, 25, 50, 75, 90, 100] for the first block, [5, 10, 25, 50, 75, 90, 95] for the
later projected ones

***

### medianFee

> **medianFee**: `number`

Defined in: [modules/bitview-client/index.js:786](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L786)

Median fee rate in sat/vB

***

### nTx

> **nTx**: `number`

Defined in: [modules/bitview-client/index.js:784](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L784)

Number of transactions in the projected block

***

### totalFees

> **totalFees**: `number`

Defined in: [modules/bitview-client/index.js:785](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L785)

Total fees in satoshis
