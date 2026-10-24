[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / BlockRewardsEntry

# Interface: BlockRewardsEntry

Defined in: [modules/bitview-client/index.js:262](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L262)

## Properties

### avgHeight

> **avgHeight**: `number`

Defined in: [modules/bitview-client/index.js:263](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L263)

Average block height in this bucket

***

### avgRewards

> **avgRewards**: `number`

Defined in: [modules/bitview-client/index.js:265](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L265)

Mean coinbase reward (subsidy + fees) per block in this bucket, rounded half up (sats)

***

### timestamp

> **timestamp**: `number`

Defined in: [modules/bitview-client/index.js:264](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L264)

Average block timestamp in this bucket

***

### USD

> **USD**: `number`

Defined in: [modules/bitview-client/index.js:266](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L266)

Mean BTC/USD spot price over the bucket's blocks (the price, not the rewards in USD)
