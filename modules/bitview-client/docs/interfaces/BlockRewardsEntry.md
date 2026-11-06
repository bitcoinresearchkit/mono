[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / BlockRewardsEntry

# Interface: BlockRewardsEntry

Defined in: [modules/bitview-client/index.js:288](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L288)

## Properties

### avgHeight

> **avgHeight**: `number`

Defined in: [modules/bitview-client/index.js:289](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L289)

Average block height in this bucket

***

### avgRewards

> **avgRewards**: `number`

Defined in: [modules/bitview-client/index.js:291](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L291)

Mean coinbase reward (subsidy + fees) per block in this bucket, rounded half up (sats)

***

### timestamp

> **timestamp**: `number`

Defined in: [modules/bitview-client/index.js:290](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L290)

Average block timestamp in this bucket

***

### USD

> **USD**: `number`

Defined in: [modules/bitview-client/index.js:292](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L292)

Mean BTC/USD spot price over the bucket's blocks (the price, not the rewards in USD)
