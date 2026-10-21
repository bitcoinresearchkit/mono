[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / BlockRewardsEntry

# Interface: BlockRewardsEntry

Defined in: [modules/bitview-client/index.js:219](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L219)

## Properties

### avgHeight

> **avgHeight**: `number`

Defined in: [modules/bitview-client/index.js:220](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L220)

Average block height in this bucket

***

### avgRewards

> **avgRewards**: `number`

Defined in: [modules/bitview-client/index.js:222](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L222)

Mean coinbase reward (subsidy + fees) per block in this bucket, rounded half up (sats)

***

### timestamp

> **timestamp**: `number`

Defined in: [modules/bitview-client/index.js:221](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L221)

Average block timestamp in this bucket

***

### USD

> **USD**: `number`

Defined in: [modules/bitview-client/index.js:223](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L223)

Mean BTC/USD spot price over the bucket's blocks (the price, not the rewards in USD)
