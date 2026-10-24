[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / RecommendedFees

# Interface: RecommendedFees

Defined in: [modules/bitview-client/index.js:1149](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1149)

## Properties

### economyFee

> **economyFee**: `number`

Defined in: [modules/bitview-client/index.js:1153](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1153)

Smoothed, fullness-adjusted third-block fee estimate, bounded by `minimumFee` and twice
it; `minimumFee` when no third projected block exists

***

### fastestFee

> **fastestFee**: `number`

Defined in: [modules/bitview-client/index.js:1150](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1150)

Fee rate for fastest confirmation (next block)

***

### halfHourFee

> **halfHourFee**: `number`

Defined in: [modules/bitview-client/index.js:1151](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1151)

Fee rate for confirmation within ~30 minutes (second projected block)

***

### hourFee

> **hourFee**: `number`

Defined in: [modules/bitview-client/index.js:1152](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1152)

Fee rate for confirmation within ~1 hour (third projected block)

***

### minimumFee

> **minimumFee**: `number`

Defined in: [modules/bitview-client/index.js:1155](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1155)

The node's current mempool minimum fee rate, rounded up to a whole sat/vB (at least 1)
