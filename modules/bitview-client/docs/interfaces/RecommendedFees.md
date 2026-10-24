[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / RecommendedFees

# Interface: RecommendedFees

Defined in: [modules/bitview-client/index.js:1227](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1227)

## Properties

### economyFee

> **economyFee**: `number`

Defined in: [modules/bitview-client/index.js:1231](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1231)

Smoothed, fullness-adjusted third-block fee estimate, bounded by `minimumFee` and twice
it; `minimumFee` when no third projected block exists

***

### fastestFee

> **fastestFee**: `number`

Defined in: [modules/bitview-client/index.js:1228](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1228)

Fee rate for fastest confirmation (next block)

***

### halfHourFee

> **halfHourFee**: `number`

Defined in: [modules/bitview-client/index.js:1229](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1229)

Fee rate for confirmation within ~30 minutes (second projected block)

***

### hourFee

> **hourFee**: `number`

Defined in: [modules/bitview-client/index.js:1230](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1230)

Fee rate for confirmation within ~1 hour (third projected block)

***

### minimumFee

> **minimumFee**: `number`

Defined in: [modules/bitview-client/index.js:1233](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1233)

The node's current mempool minimum fee rate, rounded up to a whole sat/vB (at least 1)
