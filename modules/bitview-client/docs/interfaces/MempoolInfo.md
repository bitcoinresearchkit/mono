[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / MempoolInfo

# Interface: MempoolInfo

Defined in: [modules/bitview-client/index.js:707](https://github.com/bitcoinresearchkit/mono/blob/35af38a95a9d8eab77d0bc6900ec332bc53accaf/modules/bitview-client/index.js#L707)

## Properties

### count

> **count**: `number`

Defined in: [modules/bitview-client/index.js:708](https://github.com/bitcoinresearchkit/mono/blob/35af38a95a9d8eab77d0bc6900ec332bc53accaf/modules/bitview-client/index.js#L708)

Number of transactions in the mempool

***

### feeHistogram

> **feeHistogram**: `number`[][]

Defined in: [modules/bitview-client/index.js:711](https://github.com/bitcoinresearchkit/mono/blob/35af38a95a9d8eab77d0bc6900ec332bc53accaf/modules/bitview-client/index.js#L711)

Fee histogram as Esplora bins it: `[[fee_rate, vsize], ...]` by descending fee rate, each
entry the vsize of the transactions paying from its fee rate up to the previous entry's,
a bin closing at the first fee rate change past 50,000 vB

***

### totalFee

> **totalFee**: `number`

Defined in: [modules/bitview-client/index.js:710](https://github.com/bitcoinresearchkit/mono/blob/35af38a95a9d8eab77d0bc6900ec332bc53accaf/modules/bitview-client/index.js#L710)

Total fees of all transactions in the mempool (satoshis)

***

### vsize

> **vsize**: `number`

Defined in: [modules/bitview-client/index.js:709](https://github.com/bitcoinresearchkit/mono/blob/35af38a95a9d8eab77d0bc6900ec332bc53accaf/modules/bitview-client/index.js#L709)

Total virtual size of all transactions in the mempool (vbytes)
