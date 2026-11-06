[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / RecommendedFees

# Interface: RecommendedFees

Defined in: [modules/bitview-client/index.js:1273](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1273)

## Properties

### economyFee

> **economyFee**: `number`

Defined in: [modules/bitview-client/index.js:1277](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1277)

Smoothed, fullness-adjusted third-block fee estimate, bounded by `minimumFee` and twice
  it; `minimumFee` when no third projected block exists

***

### fastestFee

> **fastestFee**: `number`

Defined in: [modules/bitview-client/index.js:1274](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1274)

Fee rate for fastest confirmation (next block)

***

### halfHourFee

> **halfHourFee**: `number`

Defined in: [modules/bitview-client/index.js:1275](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1275)

Fee rate for confirmation within ~30 minutes (second projected block)

***

### hourFee

> **hourFee**: `number`

Defined in: [modules/bitview-client/index.js:1276](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1276)

Fee rate for confirmation within ~1 hour (third projected block)

***

### minimumFee

> **minimumFee**: `number`

Defined in: [modules/bitview-client/index.js:1279](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1279)

The node's current mempool minimum fee rate, rounded up to a whole sat/vB (at least 1)
