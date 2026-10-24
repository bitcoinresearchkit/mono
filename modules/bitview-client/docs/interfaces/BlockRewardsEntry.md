[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / BlockRewardsEntry

# Interface: BlockRewardsEntry

Defined in: [modules/bitview-client/index.js:277](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L277)

## Properties

### avgHeight

> **avgHeight**: `number`

Defined in: [modules/bitview-client/index.js:278](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L278)

Average block height in this bucket

***

### avgRewards

> **avgRewards**: `number`

Defined in: [modules/bitview-client/index.js:280](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L280)

Mean coinbase reward (subsidy + fees) per block in this bucket, rounded half up (sats)

***

### timestamp

> **timestamp**: `number`

Defined in: [modules/bitview-client/index.js:279](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L279)

Average block timestamp in this bucket

***

### USD

> **USD**: `number`

Defined in: [modules/bitview-client/index.js:281](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L281)

Mean BTC/USD spot price over the bucket's blocks (the price, not the rewards in USD)
