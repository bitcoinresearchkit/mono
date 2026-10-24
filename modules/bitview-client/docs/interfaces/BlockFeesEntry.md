[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / BlockFeesEntry

# Interface: BlockFeesEntry

Defined in: [modules/bitview-client/index.js:170](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L170)

## Properties

### avgFees

> **avgFees**: `number`

Defined in: [modules/bitview-client/index.js:173](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L173)

Mean fees per block in this bucket, rounded half up (sats)

***

### avgHeight

> **avgHeight**: `number`

Defined in: [modules/bitview-client/index.js:171](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L171)

Average block height in this bucket

***

### timestamp

> **timestamp**: `number`

Defined in: [modules/bitview-client/index.js:172](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L172)

Average block timestamp in this bucket

***

### USD

> **USD**: `number`

Defined in: [modules/bitview-client/index.js:174](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L174)

Mean BTC/USD spot price over the bucket's blocks (the price, not the fees in USD)
