[**brk-client**](../README.md)

***

[brk-client](../globals.md) / MempoolInfo

# Interface: MempoolInfo

Defined in: [Developer/brk/modules/brk-client/index.js:755](https://github.com/bitcoinresearchkit/brk/blob/70a3e90b82b397fa1d5ca46c4ed613066ed3d195/modules/brk-client/index.js#L755)

## Properties

### count

> **count**: `number`

Defined in: [Developer/brk/modules/brk-client/index.js:756](https://github.com/bitcoinresearchkit/brk/blob/70a3e90b82b397fa1d5ca46c4ed613066ed3d195/modules/brk-client/index.js#L756)

Number of transactions in the mempool

***

### feeHistogram

> **feeHistogram**: `number`[][]

Defined in: [Developer/brk/modules/brk-client/index.js:759](https://github.com/bitcoinresearchkit/brk/blob/70a3e90b82b397fa1d5ca46c4ed613066ed3d195/modules/brk-client/index.js#L759)

Fee histogram: `[[fee_rate, vsize], ...]` sorted by descending fee rate

***

### totalFee

> **totalFee**: `number`

Defined in: [Developer/brk/modules/brk-client/index.js:758](https://github.com/bitcoinresearchkit/brk/blob/70a3e90b82b397fa1d5ca46c4ed613066ed3d195/modules/brk-client/index.js#L758)

Total fees of all transactions in the mempool (satoshis)

***

### vsize

> **vsize**: `number`

Defined in: [Developer/brk/modules/brk-client/index.js:757](https://github.com/bitcoinresearchkit/brk/blob/70a3e90b82b397fa1d5ca46c4ed613066ed3d195/modules/brk-client/index.js#L757)

Total virtual size of all transactions in the mempool (vbytes)
