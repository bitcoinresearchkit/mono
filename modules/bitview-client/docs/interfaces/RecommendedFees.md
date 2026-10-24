[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / RecommendedFees

# Interface: RecommendedFees

Defined in: [modules/bitview-client/index.js:1166](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1166)

## Properties

### economyFee

> **economyFee**: `number`

Defined in: [modules/bitview-client/index.js:1170](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1170)

Smoothed, fullness-adjusted third-block fee estimate, bounded by `minimumFee` and twice
it; `minimumFee` when no third projected block exists

***

### fastestFee

> **fastestFee**: `number`

Defined in: [modules/bitview-client/index.js:1167](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1167)

Fee rate for fastest confirmation (next block)

***

### halfHourFee

> **halfHourFee**: `number`

Defined in: [modules/bitview-client/index.js:1168](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1168)

Fee rate for confirmation within ~30 minutes (second projected block)

***

### hourFee

> **hourFee**: `number`

Defined in: [modules/bitview-client/index.js:1169](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1169)

Fee rate for confirmation within ~1 hour (third projected block)

***

### minimumFee

> **minimumFee**: `number`

Defined in: [modules/bitview-client/index.js:1172](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1172)

The node's current mempool minimum fee rate, rounded up to a whole sat/vB (at least 1)
