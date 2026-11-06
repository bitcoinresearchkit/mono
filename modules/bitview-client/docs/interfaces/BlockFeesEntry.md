[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / BlockFeesEntry

# Interface: BlockFeesEntry

Defined in: [modules/bitview-client/index.js:196](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L196)

## Properties

### avgFees

> **avgFees**: `number`

Defined in: [modules/bitview-client/index.js:199](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L199)

Mean fees per block in this bucket, rounded half up (sats)

***

### avgHeight

> **avgHeight**: `number`

Defined in: [modules/bitview-client/index.js:197](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L197)

Average block height in this bucket

***

### timestamp

> **timestamp**: `number`

Defined in: [modules/bitview-client/index.js:198](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L198)

Average block timestamp in this bucket

***

### USD

> **USD**: `number`

Defined in: [modules/bitview-client/index.js:200](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L200)

Mean BTC/USD spot price over the bucket's blocks (the price, not the fees in USD)
