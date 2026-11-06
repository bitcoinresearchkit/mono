[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / BlockRewardsEntry

# Interface: BlockRewardsEntry

Defined in: [modules/bitview-client/index.js:235](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L235)

## Properties

### avgHeight

> **avgHeight**: `number`

Defined in: [modules/bitview-client/index.js:236](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L236)

Average block height in this bucket

***

### avgRewards

> **avgRewards**: `number`

Defined in: [modules/bitview-client/index.js:238](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L238)

Mean coinbase reward (subsidy + fees) per block in this bucket, rounded half up (sats)

***

### timestamp

> **timestamp**: `number`

Defined in: [modules/bitview-client/index.js:237](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L237)

Average block timestamp in this bucket

***

### USD

> **USD**: `number`

Defined in: [modules/bitview-client/index.js:239](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L239)

Mean BTC/USD spot price over the bucket's blocks (the price, not the rewards in USD)
