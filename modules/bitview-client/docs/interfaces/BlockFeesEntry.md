[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / BlockFeesEntry

# Interface: BlockFeesEntry

Defined in: [modules/bitview-client/index.js:157](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L157)

## Properties

### avgFees

> **avgFees**: `number`

Defined in: [modules/bitview-client/index.js:160](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L160)

Mean fees per block in this bucket, rounded half up (sats)

***

### avgHeight

> **avgHeight**: `number`

Defined in: [modules/bitview-client/index.js:158](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L158)

Average block height in this bucket

***

### timestamp

> **timestamp**: `number`

Defined in: [modules/bitview-client/index.js:159](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L159)

Average block timestamp in this bucket

***

### USD

> **USD**: `number`

Defined in: [modules/bitview-client/index.js:161](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L161)

Mean BTC/USD spot price over the bucket's blocks (the price, not the fees in USD)
