[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / SeriesCount

# Interface: SeriesCount

Defined in: [modules/bitview-client/index.js:1315](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1315)

## Properties

### distinct

> **distinct**: `number`

Defined in: [modules/bitview-client/index.js:1316](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1316)

Number of unique series available (e.g., realized_price, market_cap)

***

### lazy

> **lazy**: `number`

Defined in: [modules/bitview-client/index.js:1318](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1318)

Number of lazy (computed on-the-fly) series-index combinations

***

### stored

> **stored**: `number`

Defined in: [modules/bitview-client/index.js:1319](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1319)

Number of eager (stored on disk) series-index combinations

***

### total

> **total**: `number`

Defined in: [modules/bitview-client/index.js:1317](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1317)

Total number of series-index combinations across all timeframes
