[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / DetailedSeriesCount

# Interface: DetailedSeriesCount

Defined in: [modules/bitview-client/index.js:497](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L497)

## Properties

### byDb

> **byDb**: `object`

Defined in: [modules/bitview-client/index.js:502](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L502)

#### Index Signature

\[`key`: `string`\]: [`SeriesCount`](SeriesCount.md)

***

### distinct

> **distinct**: `number`

Defined in: [modules/bitview-client/index.js:498](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L498)

Number of unique series available (e.g., realized_price, market_cap)

***

### lazy

> **lazy**: `number`

Defined in: [modules/bitview-client/index.js:500](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L500)

Number of lazy (computed on-the-fly) series-index combinations

***

### stored

> **stored**: `number`

Defined in: [modules/bitview-client/index.js:501](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L501)

Number of eager (stored on disk) series-index combinations

***

### total

> **total**: `number`

Defined in: [modules/bitview-client/index.js:499](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L499)

Total number of series-index combinations across all timeframes
