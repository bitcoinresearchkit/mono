[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / MempoolBlock

# Interface: MempoolBlock

Defined in: modules/bitview-client/index.js:760

## Properties

### blockSize

> **blockSize**: `number`

Defined in: modules/bitview-client/index.js:761

Sum of the transactions' serialized sizes in bytes (witness + non-witness), without header
or coinbase.

***

### blockVSize

> **blockVSize**: `number`

Defined in: modules/bitview-client/index.js:763

Sum of the transactions' virtual sizes in vbytes

***

### feeRange

> **feeRange**: `number`[]

Defined in: modules/bitview-client/index.js:767

Vsize-weighted percentiles of the transactions' CPFP chunk fee rates, in sat/vB:
[0, 10, 25, 50, 75, 90, 100] for the first block, [5, 10, 25, 50, 75, 90, 95] for the
later projected ones

***

### medianFee

> **medianFee**: `number`

Defined in: modules/bitview-client/index.js:766

Median fee rate in sat/vB

***

### nTx

> **nTx**: `number`

Defined in: modules/bitview-client/index.js:764

Number of transactions in the projected block

***

### totalFees

> **totalFees**: `number`

Defined in: modules/bitview-client/index.js:765

Total fees in satoshis
