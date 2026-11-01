[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / PreciseRecommendedFees

# Interface: PreciseRecommendedFees

Defined in: modules/bitview-client/index.js:1207

## Properties

### economyFee

> **economyFee**: `number`

Defined in: modules/bitview-client/index.js:1211

Smoothed, fullness-adjusted third-block fee estimate, bounded by `minimumFee` and twice
it; `minimumFee` when no third projected block exists

***

### fastestFee

> **fastestFee**: `number`

Defined in: modules/bitview-client/index.js:1208

Fee rate for fastest confirmation (next block)

***

### halfHourFee

> **halfHourFee**: `number`

Defined in: modules/bitview-client/index.js:1209

Fee rate for confirmation within ~30 minutes (second projected block)

***

### hourFee

> **hourFee**: `number`

Defined in: modules/bitview-client/index.js:1210

Fee rate for confirmation within ~1 hour (third projected block)

***

### minimumFee

> **minimumFee**: `number`

Defined in: modules/bitview-client/index.js:1213

The node's current mempool minimum fee rate, rounded up to 0.001 sat/vB (at least 0.001)
