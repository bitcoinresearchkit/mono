[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / MempoolInfo

# Interface: MempoolInfo

Defined in: [modules/bitview-client/index.js:843](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L843)

## Properties

### count

> **count**: `number`

Defined in: [modules/bitview-client/index.js:844](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L844)

Number of transactions in the mempool

***

### feeHistogram

> **feeHistogram**: `number`[][]

Defined in: [modules/bitview-client/index.js:847](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L847)

Fee histogram as Esplora bins it: `[[fee_rate, vsize], ...]` by descending fee rate, each
entry the vsize of the transactions paying from its fee rate up to the previous entry's,
a bin closing at the first fee rate change past 50,000 vB

***

### totalFee

> **totalFee**: `number`

Defined in: [modules/bitview-client/index.js:846](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L846)

Total fees of all transactions in the mempool (satoshis)

***

### vsize

> **vsize**: `number`

Defined in: [modules/bitview-client/index.js:845](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L845)

Total virtual size of all transactions in the mempool (vbytes)
