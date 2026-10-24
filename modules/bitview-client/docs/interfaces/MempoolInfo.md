[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / MempoolInfo

# Interface: MempoolInfo

Defined in: [modules/bitview-client/index.js:809](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L809)

## Properties

### count

> **count**: `number`

Defined in: [modules/bitview-client/index.js:810](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L810)

Number of transactions in the mempool

***

### feeHistogram

> **feeHistogram**: `number`[][]

Defined in: [modules/bitview-client/index.js:813](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L813)

Fee histogram as Esplora bins it: `[[fee_rate, vsize], ...]` by descending fee rate, each
entry the vsize of the transactions paying from its fee rate up to the previous entry's,
a bin closing at the first fee rate change past 50,000 vB

***

### totalFee

> **totalFee**: `number`

Defined in: [modules/bitview-client/index.js:812](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L812)

Total fees of all transactions in the mempool (satoshis)

***

### vsize

> **vsize**: `number`

Defined in: [modules/bitview-client/index.js:811](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L811)

Total virtual size of all transactions in the mempool (vbytes)
