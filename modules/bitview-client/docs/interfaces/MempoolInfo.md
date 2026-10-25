[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / MempoolInfo

# Interface: MempoolInfo

Defined in: [modules/bitview-client/index.js:833](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L833)

## Properties

### count

> **count**: `number`

Defined in: [modules/bitview-client/index.js:834](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L834)

Number of transactions in the mempool

***

### feeHistogram

> **feeHistogram**: `number`[][]

Defined in: [modules/bitview-client/index.js:837](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L837)

Fee histogram as Esplora bins it: `[[fee_rate, vsize], ...]` by descending fee rate, each
entry the vsize of the transactions paying from its fee rate up to the previous entry's,
a bin closing at the first fee rate change past 50,000 vB

***

### totalFee

> **totalFee**: `number`

Defined in: [modules/bitview-client/index.js:836](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L836)

Total fees of all transactions in the mempool (satoshis)

***

### vsize

> **vsize**: `number`

Defined in: [modules/bitview-client/index.js:835](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L835)

Total virtual size of all transactions in the mempool (vbytes)
