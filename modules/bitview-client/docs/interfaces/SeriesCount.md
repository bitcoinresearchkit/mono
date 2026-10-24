[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / SeriesCount

# Interface: SeriesCount

Defined in: [modules/bitview-client/index.js:1265](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1265)

## Properties

### distinct

> **distinct**: `number`

Defined in: [modules/bitview-client/index.js:1266](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1266)

Number of unique series available (e.g., realized_price, market_cap)

***

### lazy

> **lazy**: `number`

Defined in: [modules/bitview-client/index.js:1268](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1268)

Number of lazy (computed on-the-fly) series-index combinations

***

### stored

> **stored**: `number`

Defined in: [modules/bitview-client/index.js:1269](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1269)

Number of eager (stored on disk) series-index combinations

***

### total

> **total**: `number`

Defined in: [modules/bitview-client/index.js:1267](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1267)

Total number of series-index combinations across all timeframes
