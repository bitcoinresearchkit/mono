[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / MempoolInfo

# Interface: MempoolInfo

Defined in: [modules/bitview-client/index.js:858](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L858)

## Properties

### count

> **count**: `number`

Defined in: [modules/bitview-client/index.js:859](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L859)

Number of transactions in the mempool

***

### feeHistogram

> **feeHistogram**: `number`[][]

Defined in: [modules/bitview-client/index.js:862](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L862)

Fee histogram as Esplora bins it: `[[fee_rate, vsize], ...]` by descending fee rate, each
entry the vsize of the transactions paying from its fee rate up to the previous entry's,
a bin closing at the first fee rate change past 50,000 vB

***

### totalFee

> **totalFee**: `number`

Defined in: [modules/bitview-client/index.js:861](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L861)

Total fees of all transactions in the mempool (satoshis)

***

### vsize

> **vsize**: `number`

Defined in: [modules/bitview-client/index.js:860](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L860)

Total virtual size of all transactions in the mempool (vbytes)
