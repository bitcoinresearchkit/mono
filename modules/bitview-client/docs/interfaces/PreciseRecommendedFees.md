[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / PreciseRecommendedFees

# Interface: PreciseRecommendedFees

Defined in: [modules/bitview-client/index.js:1137](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1137)

## Properties

### economyFee

> **economyFee**: `number`

Defined in: [modules/bitview-client/index.js:1141](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1141)

Smoothed, fullness-adjusted third-block fee estimate, bounded by `minimumFee` and twice
it; `minimumFee` when no third projected block exists

***

### fastestFee

> **fastestFee**: `number`

Defined in: [modules/bitview-client/index.js:1138](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1138)

Fee rate for fastest confirmation (next block)

***

### halfHourFee

> **halfHourFee**: `number`

Defined in: [modules/bitview-client/index.js:1139](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1139)

Fee rate for confirmation within ~30 minutes (second projected block)

***

### hourFee

> **hourFee**: `number`

Defined in: [modules/bitview-client/index.js:1140](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1140)

Fee rate for confirmation within ~1 hour (third projected block)

***

### minimumFee

> **minimumFee**: `number`

Defined in: [modules/bitview-client/index.js:1143](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1143)

The node's current mempool minimum fee rate, rounded up to 0.001 sat/vB (at least 0.001)
