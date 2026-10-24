[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / RecommendedFees

# Interface: RecommendedFees

Defined in: [modules/bitview-client/index.js:1195](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1195)

## Properties

### economyFee

> **economyFee**: `number`

Defined in: [modules/bitview-client/index.js:1199](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1199)

Smoothed, fullness-adjusted third-block fee estimate, bounded by `minimumFee` and twice
it; `minimumFee` when no third projected block exists

***

### fastestFee

> **fastestFee**: `number`

Defined in: [modules/bitview-client/index.js:1196](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1196)

Fee rate for fastest confirmation (next block)

***

### halfHourFee

> **halfHourFee**: `number`

Defined in: [modules/bitview-client/index.js:1197](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1197)

Fee rate for confirmation within ~30 minutes (second projected block)

***

### hourFee

> **hourFee**: `number`

Defined in: [modules/bitview-client/index.js:1198](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1198)

Fee rate for confirmation within ~1 hour (third projected block)

***

### minimumFee

> **minimumFee**: `number`

Defined in: [modules/bitview-client/index.js:1201](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1201)

The node's current mempool minimum fee rate, rounded up to a whole sat/vB (at least 1)
