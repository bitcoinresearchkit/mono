[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / MempoolInfo

# Interface: MempoolInfo

Defined in: [modules/bitview-client/index.js:779](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L779)

## Properties

### count

> **count**: `number`

Defined in: [modules/bitview-client/index.js:780](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L780)

Number of transactions in the mempool

***

### feeHistogram

> **feeHistogram**: `number`[][]

Defined in: [modules/bitview-client/index.js:783](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L783)

Fee histogram as Esplora bins it: `[[fee_rate, vsize], ...]` by descending fee rate, each
entry the vsize of the transactions paying from its fee rate up to the previous entry's,
a bin closing at the first fee rate change past 50,000 vB

***

### totalFee

> **totalFee**: `number`

Defined in: [modules/bitview-client/index.js:782](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L782)

Total fees of all transactions in the mempool (satoshis)

***

### vsize

> **vsize**: `number`

Defined in: [modules/bitview-client/index.js:781](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L781)

Total virtual size of all transactions in the mempool (vbytes)
