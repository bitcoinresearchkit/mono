[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / SeriesCount

# Interface: SeriesCount

Defined in: [modules/bitview-client/index.js:1343](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1343)

## Properties

### distinct

> **distinct**: `number`

Defined in: [modules/bitview-client/index.js:1344](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1344)

Number of unique series available (e.g., realized_price, market_cap)

***

### lazy

> **lazy**: `number`

Defined in: [modules/bitview-client/index.js:1346](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1346)

Number of lazy (computed on-the-fly) series-index combinations

***

### stored

> **stored**: `number`

Defined in: [modules/bitview-client/index.js:1347](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1347)

Number of eager (stored on disk) series-index combinations

***

### total

> **total**: `number`

Defined in: [modules/bitview-client/index.js:1345](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1345)

Total number of series-index combinations across all timeframes
