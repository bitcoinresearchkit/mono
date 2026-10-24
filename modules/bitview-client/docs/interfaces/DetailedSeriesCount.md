[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / DetailedSeriesCount

# Interface: DetailedSeriesCount

Defined in: [modules/bitview-client/index.js:488](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L488)

## Properties

### byDb

> **byDb**: `object`

Defined in: [modules/bitview-client/index.js:493](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L493)

#### Index Signature

\[`key`: `string`\]: [`SeriesCount`](SeriesCount.md)

***

### distinct

> **distinct**: `number`

Defined in: [modules/bitview-client/index.js:489](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L489)

Number of unique series available (e.g., realized_price, market_cap)

***

### lazy

> **lazy**: `number`

Defined in: [modules/bitview-client/index.js:491](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L491)

Number of lazy (computed on-the-fly) series-index combinations

***

### stored

> **stored**: `number`

Defined in: [modules/bitview-client/index.js:492](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L492)

Number of eager (stored on disk) series-index combinations

***

### total

> **total**: `number`

Defined in: [modules/bitview-client/index.js:490](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L490)

Total number of series-index combinations across all timeframes
