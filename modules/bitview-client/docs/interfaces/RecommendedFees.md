[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / RecommendedFees

# Interface: RecommendedFees

Defined in: [modules/bitview-client/index.js:1177](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1177)

## Properties

### economyFee

> **economyFee**: `number`

Defined in: [modules/bitview-client/index.js:1181](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1181)

Smoothed, fullness-adjusted third-block fee estimate, bounded by `minimumFee` and twice
it; `minimumFee` when no third projected block exists

***

### fastestFee

> **fastestFee**: `number`

Defined in: [modules/bitview-client/index.js:1178](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1178)

Fee rate for fastest confirmation (next block)

***

### halfHourFee

> **halfHourFee**: `number`

Defined in: [modules/bitview-client/index.js:1179](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1179)

Fee rate for confirmation within ~30 minutes (second projected block)

***

### hourFee

> **hourFee**: `number`

Defined in: [modules/bitview-client/index.js:1180](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1180)

Fee rate for confirmation within ~1 hour (third projected block)

***

### minimumFee

> **minimumFee**: `number`

Defined in: [modules/bitview-client/index.js:1183](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1183)

The node's current mempool minimum fee rate, rounded up to a whole sat/vB (at least 1)
