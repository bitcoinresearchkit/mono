[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / MempoolInfo

# Interface: MempoolInfo

Defined in: [Developer/mono/modules/bitview-client/index.js:766](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L766)

## Properties

### count

> **count**: `number`

Defined in: [Developer/mono/modules/bitview-client/index.js:767](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L767)

Number of transactions in the mempool

***

### feeHistogram

> **feeHistogram**: `number`[][]

Defined in: [Developer/mono/modules/bitview-client/index.js:770](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L770)

Fee histogram: `[[fee_rate, vsize], ...]` sorted by descending fee rate

***

### totalFee

> **totalFee**: `number`

Defined in: [Developer/mono/modules/bitview-client/index.js:769](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L769)

Total fees of all transactions in the mempool (satoshis)

***

### vsize

> **vsize**: `number`

Defined in: [Developer/mono/modules/bitview-client/index.js:768](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L768)

Total virtual size of all transactions in the mempool (vbytes)
