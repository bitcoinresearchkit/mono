[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / MempoolInfo

# Interface: MempoolInfo

Defined in: [Developer/mono/modules/bitview-client/index.js:727](https://github.com/bitcoinresearchkit/mono/blob/ea62dc574077b34b3299e636a16f81ea96849ead/modules/bitview-client/index.js#L727)

## Properties

### count

> **count**: `number`

Defined in: [Developer/mono/modules/bitview-client/index.js:728](https://github.com/bitcoinresearchkit/mono/blob/ea62dc574077b34b3299e636a16f81ea96849ead/modules/bitview-client/index.js#L728)

Number of transactions in the mempool

***

### feeHistogram

> **feeHistogram**: `number`[][]

Defined in: [Developer/mono/modules/bitview-client/index.js:731](https://github.com/bitcoinresearchkit/mono/blob/ea62dc574077b34b3299e636a16f81ea96849ead/modules/bitview-client/index.js#L731)

Fee histogram: `[[fee_rate, vsize], ...]` sorted by descending fee rate

***

### totalFee

> **totalFee**: `number`

Defined in: [Developer/mono/modules/bitview-client/index.js:730](https://github.com/bitcoinresearchkit/mono/blob/ea62dc574077b34b3299e636a16f81ea96849ead/modules/bitview-client/index.js#L730)

Total fees of all transactions in the mempool (satoshis)

***

### vsize

> **vsize**: `number`

Defined in: [Developer/mono/modules/bitview-client/index.js:729](https://github.com/bitcoinresearchkit/mono/blob/ea62dc574077b34b3299e636a16f81ea96849ead/modules/bitview-client/index.js#L729)

Total virtual size of all transactions in the mempool (vbytes)
