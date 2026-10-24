[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / DetailedSeriesCount

# Interface: DetailedSeriesCount

Defined in: [modules/bitview-client/index.js:503](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L503)

## Properties

### byDb

> **byDb**: `object`

Defined in: [modules/bitview-client/index.js:508](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L508)

#### Index Signature

\[`key`: `string`\]: [`SeriesCount`](SeriesCount.md)

***

### distinct

> **distinct**: `number`

Defined in: [modules/bitview-client/index.js:504](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L504)

Number of unique series available (e.g., realized_price, market_cap)

***

### lazy

> **lazy**: `number`

Defined in: [modules/bitview-client/index.js:506](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L506)

Number of lazy (computed on-the-fly) series-index combinations

***

### stored

> **stored**: `number`

Defined in: [modules/bitview-client/index.js:507](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L507)

Number of eager (stored on disk) series-index combinations

***

### total

> **total**: `number`

Defined in: [modules/bitview-client/index.js:505](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L505)

Total number of series-index combinations across all timeframes
