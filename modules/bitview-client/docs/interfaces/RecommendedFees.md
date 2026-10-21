[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / RecommendedFees

# Interface: RecommendedFees

Defined in: [modules/bitview-client/index.js:1082](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1082)

## Properties

### economyFee

> **economyFee**: `number`

Defined in: [modules/bitview-client/index.js:1086](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1086)

Smoothed, fullness-adjusted third-block fee estimate, bounded by `minimumFee` and twice
it; `minimumFee` when no third projected block exists

***

### fastestFee

> **fastestFee**: `number`

Defined in: [modules/bitview-client/index.js:1083](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1083)

Fee rate for fastest confirmation (next block)

***

### halfHourFee

> **halfHourFee**: `number`

Defined in: [modules/bitview-client/index.js:1084](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1084)

Fee rate for confirmation within ~30 minutes (second projected block)

***

### hourFee

> **hourFee**: `number`

Defined in: [modules/bitview-client/index.js:1085](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1085)

Fee rate for confirmation within ~1 hour (third projected block)

***

### minimumFee

> **minimumFee**: `number`

Defined in: [modules/bitview-client/index.js:1088](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1088)

The node's current mempool minimum fee rate, rounded up to a whole sat/vB (at least 1)
