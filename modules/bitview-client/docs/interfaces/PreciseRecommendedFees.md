[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / PreciseRecommendedFees

# Interface: PreciseRecommendedFees

Defined in: [modules/bitview-client/index.js:1024](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1024)

## Properties

### economyFee

> **economyFee**: `number`

Defined in: [modules/bitview-client/index.js:1028](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1028)

Smoothed, fullness-adjusted third-block fee estimate, bounded by `minimumFee` and twice
it; `minimumFee` when no third projected block exists

***

### fastestFee

> **fastestFee**: `number`

Defined in: [modules/bitview-client/index.js:1025](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1025)

Fee rate for fastest confirmation (next block)

***

### halfHourFee

> **halfHourFee**: `number`

Defined in: [modules/bitview-client/index.js:1026](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1026)

Fee rate for confirmation within ~30 minutes (second projected block)

***

### hourFee

> **hourFee**: `number`

Defined in: [modules/bitview-client/index.js:1027](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1027)

Fee rate for confirmation within ~1 hour (third projected block)

***

### minimumFee

> **minimumFee**: `number`

Defined in: [modules/bitview-client/index.js:1030](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1030)

The node's current mempool minimum fee rate, rounded up to 0.001 sat/vB (at least 0.001)
