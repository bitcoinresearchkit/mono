[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / BlockFeesEntry

# Interface: BlockFeesEntry

Defined in: [modules/bitview-client/index.js:215](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L215)

## Properties

### avgFees

> **avgFees**: `number`

Defined in: [modules/bitview-client/index.js:218](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L218)

Mean fees per block in this bucket, rounded half up (sats)

***

### avgHeight

> **avgHeight**: `number`

Defined in: [modules/bitview-client/index.js:216](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L216)

Average block height in this bucket

***

### timestamp

> **timestamp**: `number`

Defined in: [modules/bitview-client/index.js:217](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L217)

Average block timestamp in this bucket

***

### USD

> **USD**: `number`

Defined in: [modules/bitview-client/index.js:219](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L219)

Mean BTC/USD spot price over the bucket's blocks (the price, not the fees in USD)
