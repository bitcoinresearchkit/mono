[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / MempoolInfo

# Interface: MempoolInfo

Defined in: [modules/bitview-client/index.js:751](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L751)

## Properties

### count

> **count**: `number`

Defined in: [modules/bitview-client/index.js:752](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L752)

Number of transactions in the mempool

***

### feeHistogram

> **feeHistogram**: `number`[][]

Defined in: [modules/bitview-client/index.js:755](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L755)

Fee histogram as Esplora bins it: `[[fee_rate, vsize], ...]` by descending fee rate, each
  entry the vsize of the transactions paying from its fee rate up to the previous entry's,
  a bin closing at the first fee rate change past 50,000 vB

***

### totalFee

> **totalFee**: `number`

Defined in: [modules/bitview-client/index.js:754](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L754)

Total fees of all transactions in the mempool (satoshis)

***

### vsize

> **vsize**: `number`

Defined in: [modules/bitview-client/index.js:753](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L753)

Total virtual size of all transactions in the mempool (vbytes)
