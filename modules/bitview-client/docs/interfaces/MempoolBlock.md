[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / MempoolBlock

# Interface: MempoolBlock

Defined in: [Developer/mono/modules/bitview-client/index.js:755](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L755)

## Properties

### blockSize

> **blockSize**: `number`

Defined in: [Developer/mono/modules/bitview-client/index.js:756](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L756)

Total serialized block size in bytes (witness + non-witness).

***

### blockVSize

> **blockVSize**: `number`

Defined in: [Developer/mono/modules/bitview-client/index.js:757](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L757)

Total block virtual size in vbytes

***

### feeRange

> **feeRange**: `number`[]

Defined in: [Developer/mono/modules/bitview-client/index.js:761](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L761)

Fee rate range: [min, 10%, 25%, 50%, 75%, 90%, max]

***

### medianFee

> **medianFee**: `number`

Defined in: [Developer/mono/modules/bitview-client/index.js:760](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L760)

Median fee rate in sat/vB

***

### nTx

> **nTx**: `number`

Defined in: [Developer/mono/modules/bitview-client/index.js:758](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L758)

Number of transactions in the projected block

***

### totalFees

> **totalFees**: `number`

Defined in: [Developer/mono/modules/bitview-client/index.js:759](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L759)

Total fees in satoshis
