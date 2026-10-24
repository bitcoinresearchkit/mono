[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / PreciseRecommendedFees

# Interface: PreciseRecommendedFees

Defined in: [modules/bitview-client/index.js:1119](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1119)

## Properties

### economyFee

> **economyFee**: `number`

Defined in: [modules/bitview-client/index.js:1123](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1123)

Smoothed, fullness-adjusted third-block fee estimate, bounded by `minimumFee` and twice
it; `minimumFee` when no third projected block exists

***

### fastestFee

> **fastestFee**: `number`

Defined in: [modules/bitview-client/index.js:1120](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1120)

Fee rate for fastest confirmation (next block)

***

### halfHourFee

> **halfHourFee**: `number`

Defined in: [modules/bitview-client/index.js:1121](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1121)

Fee rate for confirmation within ~30 minutes (second projected block)

***

### hourFee

> **hourFee**: `number`

Defined in: [modules/bitview-client/index.js:1122](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1122)

Fee rate for confirmation within ~1 hour (third projected block)

***

### minimumFee

> **minimumFee**: `number`

Defined in: [modules/bitview-client/index.js:1125](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1125)

The node's current mempool minimum fee rate, rounded up to 0.001 sat/vB (at least 0.001)
