[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / RecommendedFees

# Interface: RecommendedFees

Defined in: [modules/bitview-client/index.js:1234](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1234)

## Properties

### economyFee

> **economyFee**: `number`

Defined in: [modules/bitview-client/index.js:1238](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1238)

Smoothed, fullness-adjusted third-block fee estimate, bounded by `minimumFee` and twice
  it; `minimumFee` when no third projected block exists

***

### fastestFee

> **fastestFee**: `number`

Defined in: [modules/bitview-client/index.js:1235](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1235)

Fee rate for fastest confirmation (next block)

***

### halfHourFee

> **halfHourFee**: `number`

Defined in: [modules/bitview-client/index.js:1236](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1236)

Fee rate for confirmation within ~30 minutes (second projected block)

***

### hourFee

> **hourFee**: `number`

Defined in: [modules/bitview-client/index.js:1237](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1237)

Fee rate for confirmation within ~1 hour (third projected block)

***

### minimumFee

> **minimumFee**: `number`

Defined in: [modules/bitview-client/index.js:1240](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1240)

The node's current mempool minimum fee rate, rounded up to a whole sat/vB (at least 1)
