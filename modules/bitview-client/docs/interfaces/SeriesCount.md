[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / SeriesCount

# Interface: SeriesCount

Defined in: [modules/bitview-client/index.js:1213](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1213)

## Properties

### distinct

> **distinct**: `number`

Defined in: [modules/bitview-client/index.js:1214](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1214)

Number of unique series available (e.g., realized_price, market_cap)

***

### lazy

> **lazy**: `number`

Defined in: [modules/bitview-client/index.js:1216](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1216)

Number of lazy (computed on-the-fly) series-index combinations

***

### stored

> **stored**: `number`

Defined in: [modules/bitview-client/index.js:1217](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1217)

Number of eager (stored on disk) series-index combinations

***

### total

> **total**: `number`

Defined in: [modules/bitview-client/index.js:1215](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1215)

Total number of series-index combinations across all timeframes
