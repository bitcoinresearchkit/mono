[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / RecommendedFees

# Interface: RecommendedFees

Defined in: [modules/bitview-client/index.js:1171](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1171)

## Properties

### economyFee

> **economyFee**: `number`

Defined in: [modules/bitview-client/index.js:1175](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1175)

Smoothed, fullness-adjusted third-block fee estimate, bounded by `minimumFee` and twice
it; `minimumFee` when no third projected block exists

***

### fastestFee

> **fastestFee**: `number`

Defined in: [modules/bitview-client/index.js:1172](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1172)

Fee rate for fastest confirmation (next block)

***

### halfHourFee

> **halfHourFee**: `number`

Defined in: [modules/bitview-client/index.js:1173](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1173)

Fee rate for confirmation within ~30 minutes (second projected block)

***

### hourFee

> **hourFee**: `number`

Defined in: [modules/bitview-client/index.js:1174](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1174)

Fee rate for confirmation within ~1 hour (third projected block)

***

### minimumFee

> **minimumFee**: `number`

Defined in: [modules/bitview-client/index.js:1177](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1177)

The node's current mempool minimum fee rate, rounded up to a whole sat/vB (at least 1)
