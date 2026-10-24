[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / PreciseRecommendedFees

# Interface: PreciseRecommendedFees

Defined in: [modules/bitview-client/index.js:1113](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1113)

## Properties

### economyFee

> **economyFee**: `number`

Defined in: [modules/bitview-client/index.js:1117](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1117)

Smoothed, fullness-adjusted third-block fee estimate, bounded by `minimumFee` and twice
it; `minimumFee` when no third projected block exists

***

### fastestFee

> **fastestFee**: `number`

Defined in: [modules/bitview-client/index.js:1114](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1114)

Fee rate for fastest confirmation (next block)

***

### halfHourFee

> **halfHourFee**: `number`

Defined in: [modules/bitview-client/index.js:1115](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1115)

Fee rate for confirmation within ~30 minutes (second projected block)

***

### hourFee

> **hourFee**: `number`

Defined in: [modules/bitview-client/index.js:1116](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1116)

Fee rate for confirmation within ~1 hour (third projected block)

***

### minimumFee

> **minimumFee**: `number`

Defined in: [modules/bitview-client/index.js:1119](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1119)

The node's current mempool minimum fee rate, rounded up to 0.001 sat/vB (at least 0.001)
