[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / PreciseRecommendedFees

# Interface: PreciseRecommendedFees

Defined in: [modules/bitview-client/index.js:1091](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1091)

## Properties

### economyFee

> **economyFee**: `number`

Defined in: [modules/bitview-client/index.js:1095](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1095)

Smoothed, fullness-adjusted third-block fee estimate, bounded by `minimumFee` and twice
it; `minimumFee` when no third projected block exists

***

### fastestFee

> **fastestFee**: `number`

Defined in: [modules/bitview-client/index.js:1092](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1092)

Fee rate for fastest confirmation (next block)

***

### halfHourFee

> **halfHourFee**: `number`

Defined in: [modules/bitview-client/index.js:1093](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1093)

Fee rate for confirmation within ~30 minutes (second projected block)

***

### hourFee

> **hourFee**: `number`

Defined in: [modules/bitview-client/index.js:1094](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1094)

Fee rate for confirmation within ~1 hour (third projected block)

***

### minimumFee

> **minimumFee**: `number`

Defined in: [modules/bitview-client/index.js:1097](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1097)

The node's current mempool minimum fee rate, rounded up to 0.001 sat/vB (at least 0.001)
