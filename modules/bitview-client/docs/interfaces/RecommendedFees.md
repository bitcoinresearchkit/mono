[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / RecommendedFees

# Interface: RecommendedFees

Defined in: [modules/bitview-client/index.js:1263](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1263)

## Properties

### economyFee

> **economyFee**: `number`

Defined in: [modules/bitview-client/index.js:1267](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1267)

Smoothed, fullness-adjusted third-block fee estimate, bounded by `minimumFee` and twice
it; `minimumFee` when no third projected block exists

***

### fastestFee

> **fastestFee**: `number`

Defined in: [modules/bitview-client/index.js:1264](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1264)

Fee rate for fastest confirmation (next block)

***

### halfHourFee

> **halfHourFee**: `number`

Defined in: [modules/bitview-client/index.js:1265](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1265)

Fee rate for confirmation within ~30 minutes (second projected block)

***

### hourFee

> **hourFee**: `number`

Defined in: [modules/bitview-client/index.js:1266](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1266)

Fee rate for confirmation within ~1 hour (third projected block)

***

### minimumFee

> **minimumFee**: `number`

Defined in: [modules/bitview-client/index.js:1269](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1269)

The node's current mempool minimum fee rate, rounded up to a whole sat/vB (at least 1)
