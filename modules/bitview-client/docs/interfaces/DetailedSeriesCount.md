[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / DetailedSeriesCount

# Interface: DetailedSeriesCount

Defined in: [modules/bitview-client/index.js:499](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L499)

## Properties

### byDb

> **byDb**: `object`

Defined in: [modules/bitview-client/index.js:504](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L504)

#### Index Signature

\[`key`: `string`\]: [`SeriesCount`](SeriesCount.md)

***

### distinct

> **distinct**: `number`

Defined in: [modules/bitview-client/index.js:500](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L500)

Number of unique series available (e.g., realized_price, market_cap)

***

### lazy

> **lazy**: `number`

Defined in: [modules/bitview-client/index.js:502](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L502)

Number of lazy (computed on-the-fly) series-index combinations

***

### stored

> **stored**: `number`

Defined in: [modules/bitview-client/index.js:503](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L503)

Number of eager (stored on disk) series-index combinations

***

### total

> **total**: `number`

Defined in: [modules/bitview-client/index.js:501](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L501)

Total number of series-index combinations across all timeframes
