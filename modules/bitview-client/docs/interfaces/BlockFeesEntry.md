[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / BlockFeesEntry

# Interface: BlockFeesEntry

Defined in: [modules/bitview-client/index.js:190](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L190)

## Properties

### avgFees

> **avgFees**: `number`

Defined in: [modules/bitview-client/index.js:193](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L193)

Mean fees per block in this bucket, rounded half up (sats)

***

### avgHeight

> **avgHeight**: `number`

Defined in: [modules/bitview-client/index.js:191](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L191)

Average block height in this bucket

***

### timestamp

> **timestamp**: `number`

Defined in: [modules/bitview-client/index.js:192](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L192)

Average block timestamp in this bucket

***

### USD

> **USD**: `number`

Defined in: [modules/bitview-client/index.js:194](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L194)

Mean BTC/USD spot price over the bucket's blocks (the price, not the fees in USD)
