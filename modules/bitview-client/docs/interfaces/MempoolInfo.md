[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / MempoolInfo

# Interface: MempoolInfo

Defined in: [modules/bitview-client/index.js:712](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L712)

## Properties

### count

> **count**: `number`

Defined in: [modules/bitview-client/index.js:713](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L713)

Number of transactions in the mempool

***

### feeHistogram

> **feeHistogram**: `number`[][]

Defined in: [modules/bitview-client/index.js:716](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L716)

Fee histogram as Esplora bins it: `[[fee_rate, vsize], ...]` by descending fee rate, each
entry the vsize of the transactions paying from its fee rate up to the previous entry's,
a bin closing at the first fee rate change past 50,000 vB

***

### totalFee

> **totalFee**: `number`

Defined in: [modules/bitview-client/index.js:715](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L715)

Total fees of all transactions in the mempool (satoshis)

***

### vsize

> **vsize**: `number`

Defined in: [modules/bitview-client/index.js:714](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L714)

Total virtual size of all transactions in the mempool (vbytes)
