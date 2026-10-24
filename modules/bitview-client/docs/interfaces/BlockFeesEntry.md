[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / BlockFeesEntry

# Interface: BlockFeesEntry

Defined in: [modules/bitview-client/index.js:185](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L185)

## Properties

### avgFees

> **avgFees**: `number`

Defined in: [modules/bitview-client/index.js:188](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L188)

Mean fees per block in this bucket, rounded half up (sats)

***

### avgHeight

> **avgHeight**: `number`

Defined in: [modules/bitview-client/index.js:186](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L186)

Average block height in this bucket

***

### timestamp

> **timestamp**: `number`

Defined in: [modules/bitview-client/index.js:187](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L187)

Average block timestamp in this bucket

***

### USD

> **USD**: `number`

Defined in: [modules/bitview-client/index.js:189](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L189)

Mean BTC/USD spot price over the bucket's blocks (the price, not the fees in USD)
