[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / BlockRewardsEntry

# Interface: BlockRewardsEntry

Defined in: [modules/bitview-client/index.js:282](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L282)

## Properties

### avgHeight

> **avgHeight**: `number`

Defined in: [modules/bitview-client/index.js:283](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L283)

Average block height in this bucket

***

### avgRewards

> **avgRewards**: `number`

Defined in: [modules/bitview-client/index.js:285](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L285)

Mean coinbase reward (subsidy + fees) per block in this bucket, rounded half up (sats)

***

### timestamp

> **timestamp**: `number`

Defined in: [modules/bitview-client/index.js:284](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L284)

Average block timestamp in this bucket

***

### USD

> **USD**: `number`

Defined in: [modules/bitview-client/index.js:286](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L286)

Mean BTC/USD spot price over the bucket's blocks (the price, not the rewards in USD)
