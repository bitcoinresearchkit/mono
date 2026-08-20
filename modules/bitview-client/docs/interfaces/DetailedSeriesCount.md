[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / DetailedSeriesCount

# Interface: DetailedSeriesCount

Defined in: [Developer/brk/modules/bitview-client/index.js:501](https://github.com/bitcoinresearchkit/brk/blob/b971f8d1b413e122481dddc3c9c980b7866ab1d3/modules/bitview-client/index.js#L501)

## Properties

### byDb

> **byDb**: `object`

Defined in: [Developer/brk/modules/bitview-client/index.js:506](https://github.com/bitcoinresearchkit/brk/blob/b971f8d1b413e122481dddc3c9c980b7866ab1d3/modules/bitview-client/index.js#L506)

Per-database breakdown of counts.

#### Index Signature

\[`key`: `string`\]: [`SeriesCount`](SeriesCount.md)

***

### distinct

> **distinct**: `number`

Defined in: [Developer/brk/modules/bitview-client/index.js:502](https://github.com/bitcoinresearchkit/brk/blob/b971f8d1b413e122481dddc3c9c980b7866ab1d3/modules/bitview-client/index.js#L502)

Number of unique series available (e.g., realized_price, market_cap)

***

### lazy

> **lazy**: `number`

Defined in: [Developer/brk/modules/bitview-client/index.js:504](https://github.com/bitcoinresearchkit/brk/blob/b971f8d1b413e122481dddc3c9c980b7866ab1d3/modules/bitview-client/index.js#L504)

Number of lazy (computed on-the-fly) series-index combinations

***

### stored

> **stored**: `number`

Defined in: [Developer/brk/modules/bitview-client/index.js:505](https://github.com/bitcoinresearchkit/brk/blob/b971f8d1b413e122481dddc3c9c980b7866ab1d3/modules/bitview-client/index.js#L505)

Number of eager (stored on disk) series-index combinations

***

### total

> **total**: `number`

Defined in: [Developer/brk/modules/bitview-client/index.js:503](https://github.com/bitcoinresearchkit/brk/blob/b971f8d1b413e122481dddc3c9c980b7866ab1d3/modules/bitview-client/index.js#L503)

Total number of series-index combinations across all timeframes
