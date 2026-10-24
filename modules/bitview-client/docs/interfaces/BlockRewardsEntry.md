[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / BlockRewardsEntry

# Interface: BlockRewardsEntry

Defined in: [modules/bitview-client/index.js:292](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L292)

## Properties

### avgHeight

> **avgHeight**: `number`

Defined in: [modules/bitview-client/index.js:293](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L293)

Average block height in this bucket

***

### avgRewards

> **avgRewards**: `number`

Defined in: [modules/bitview-client/index.js:295](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L295)

Mean coinbase reward (subsidy + fees) per block in this bucket, rounded half up (sats)

***

### timestamp

> **timestamp**: `number`

Defined in: [modules/bitview-client/index.js:294](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L294)

Average block timestamp in this bucket

***

### USD

> **USD**: `number`

Defined in: [modules/bitview-client/index.js:296](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L296)

Mean BTC/USD spot price over the bucket's blocks (the price, not the rewards in USD)
