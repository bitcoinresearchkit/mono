[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / PreciseRecommendedFees

# Interface: PreciseRecommendedFees

Defined in: [modules/bitview-client/index.js:1215](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1215)

## Properties

### economyFee

> **economyFee**: `number`

Defined in: [modules/bitview-client/index.js:1219](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1219)

Smoothed, fullness-adjusted third-block fee estimate, bounded by `minimumFee` and twice
  it; `minimumFee` when no third projected block exists

***

### fastestFee

> **fastestFee**: `number`

Defined in: [modules/bitview-client/index.js:1216](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1216)

Fee rate for fastest confirmation (next block)

***

### halfHourFee

> **halfHourFee**: `number`

Defined in: [modules/bitview-client/index.js:1217](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1217)

Fee rate for confirmation within ~30 minutes (second projected block)

***

### hourFee

> **hourFee**: `number`

Defined in: [modules/bitview-client/index.js:1218](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1218)

Fee rate for confirmation within ~1 hour (third projected block)

***

### minimumFee

> **minimumFee**: `number`

Defined in: [modules/bitview-client/index.js:1221](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1221)

The node's current mempool minimum fee rate, rounded up to 0.001 sat/vB (at least 0.001)
