[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / BlockFeesEntry

# Interface: BlockFeesEntry

Defined in: [modules/bitview-client/index.js:144](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L144)

## Properties

### avgFees

> **avgFees**: `number`

Defined in: [modules/bitview-client/index.js:147](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L147)

Mean fees per block in this bucket, rounded half up (sats)

***

### avgHeight

> **avgHeight**: `number`

Defined in: [modules/bitview-client/index.js:145](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L145)

Average block height in this bucket

***

### timestamp

> **timestamp**: `number`

Defined in: [modules/bitview-client/index.js:146](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L146)

Average block timestamp in this bucket

***

### USD

> **USD**: `number`

Defined in: [modules/bitview-client/index.js:148](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L148)

Mean BTC/USD spot price over the bucket's blocks (the price, not the fees in USD)
