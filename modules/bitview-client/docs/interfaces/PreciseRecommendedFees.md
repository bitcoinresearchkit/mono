[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / PreciseRecommendedFees

# Interface: PreciseRecommendedFees

Defined in: [modules/bitview-client/index.js:1205](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1205)

## Properties

### economyFee

> **economyFee**: `number`

Defined in: [modules/bitview-client/index.js:1209](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1209)

Smoothed, fullness-adjusted third-block fee estimate, bounded by `minimumFee` and twice
it; `minimumFee` when no third projected block exists

***

### fastestFee

> **fastestFee**: `number`

Defined in: [modules/bitview-client/index.js:1206](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1206)

Fee rate for fastest confirmation (next block)

***

### halfHourFee

> **halfHourFee**: `number`

Defined in: [modules/bitview-client/index.js:1207](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1207)

Fee rate for confirmation within ~30 minutes (second projected block)

***

### hourFee

> **hourFee**: `number`

Defined in: [modules/bitview-client/index.js:1208](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1208)

Fee rate for confirmation within ~1 hour (third projected block)

***

### minimumFee

> **minimumFee**: `number`

Defined in: [modules/bitview-client/index.js:1211](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1211)

The node's current mempool minimum fee rate, rounded up to 0.001 sat/vB (at least 0.001)
